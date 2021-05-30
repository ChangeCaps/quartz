use crate::project::*;
use clap::{crate_authors, crate_version, Clap};
use egui::Key;
use egui::*;
use quartz_engine::core::game_state;
use quartz_engine::{
    core::editor_bridge::*,
    prelude::{Vec2, *},
};
use quartz_framework::{prelude::*, render::wgpu, winit};
use std::any::TypeId;
use std::collections::HashMap;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use winit::event::{self, ElementState, MouseScrollDelta, VirtualKeyCode as VKey, WindowEvent};

#[cfg(debug_assertions)]
pub const LIB_PATH: &'static str = "target/debug";
#[cfg(not(debug_assertions))]
pub const LIB_PATH: &'static str = "target/release";

pub const TARGET_FORMAT: format::TargetFormat =
    format::TargetFormat(wgpu::TextureFormat::Rgba8UnormSrgb);

pub struct GameState {
    pub state: Option<game_state::GameState>,
    pub bridge: Option<Bridge>,
    pub running: bool,
}

impl GameState {
    pub fn load(path: impl AsRef<Path>, instance: &Instance) -> Self {
        let bridge = unsafe { Bridge::load(path.as_ref()) }.unwrap();
        let state = bridge.new(instance, TARGET_FORMAT).unwrap();

        Self {
            state: Some(state),
            bridge: Some(bridge),
            running: false,
        }
    }

    pub fn deserialize<'de, D: quartz_engine::core::serde::Deserializer<'de>>(
        deserializer: D,
        path: impl AsRef<Path>,
        instance: &Instance,
    ) -> Self {
        let bridge = unsafe { Bridge::load(path.as_ref()) }.unwrap();
        let state = bridge
            .deserialize(deserializer, instance, TARGET_FORMAT)
            .unwrap();

        Self {
            state: Some(state),
            bridge: Some(bridge),
            running: false,
        }
    }

    pub fn reload<'de, D: quartz_engine::core::serde::Deserializer<'de>>(
        &mut self,
        deserializer: D,
        instance: &Instance,
    ) {
        self.state = Some(
            self.bridge
                .as_ref()
                .unwrap()
                .deserialize(deserializer, instance, TARGET_FORMAT)
                .unwrap(),
        );
    }
}

impl Drop for GameState {
    fn drop(&mut self) {
        drop(self.state.take());

        self.bridge.take().unwrap().close().unwrap();
    }
}

pub struct Camera {
    pub projection: PerspectiveProjection,
    pub transform: Transform,
    pub euler: Vec2,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            projection: Default::default(),
            transform: Transform::IDENTITY,
            euler: Vec2::new(0.0, 0.0),
        }
    }

    pub fn view_proj(&self) -> Mat4 {
        self.projection.matrix() * self.transform.matrix().inverse()
    }
}

pub enum ViewportType {
    Game,
    Editor { camera: Camera },
}

pub struct Viewport {
    pub texture_id: u64,
    pub ty: ViewportType,
}

pub enum Selection {
    None,
    Node(NodeId),
    Plugin(TypeId),
}

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
pub struct Opts {
    /// The path to your project.
    #[clap(default_value = ".")]
    pub project_path: PathBuf,
}

pub struct EditorState {
    pub egui_pipeline: RenderPipeline,
    pub egui_ctx: CtxRef,
    pub egui_raw_input: RawInput,
    pub egui_texture_version: Option<u64>,
    pub egui_texture: Texture2d,
    pub egui_sampler: Sampler,
    pub egui_point_pos: Option<Vec2>,
    pub egui_textures: HashMap<u64, Texture2d>,
    pub pick_pipeline: RenderPipeline<format::R32Uint, format::Depth32Float>,
    pub pick_texture: Texture2d<format::R32Uint>,
    pub pick_depth_texture: Texture2d<format::Depth32Float>,
    pub game: Option<GameState>,
    pub project: Project,
    pub building: Option<std::process::Child>,
    pub selection: Selection,
    pub viewports: Vec<Viewport>,
}

impl EditorState {
    pub fn new(instance: &Instance, target_format: format::TargetFormat) -> Self {
        let opts = Opts::parse();

        log::info!("Starting editor at: {}", opts.project_path.display());

        log::debug!("loading egui shader");
        let egui_shader = Shader::from_glsl(
            include_str!("shaders/egui.vert"),
            include_str!("shaders/egui.frag"),
        )
        .unwrap();
        let egui_pipeline = RenderPipeline::new(
            PipelineDescriptor {
                depth_stencil: None,
                ..PipelineDescriptor::default_settings(egui_shader, target_format)
            },
            instance,
        )
        .unwrap();

        log::debug!("loading mouse picking shader");
        let pick_shader = Shader::from_glsl(
            include_str!("shaders/pick.vert"),
            include_str!("shaders/pick.frag"),
        )
        .unwrap();
        let pick_pipeline = RenderPipeline::new(
            PipelineDescriptor::default_settings(pick_shader, Default::default()),
            instance,
        )
        .unwrap();

        let pick_texture = Texture2d::new(
            &TextureDescriptor::default_settings(D2::new(1, 1)),
            instance,
        );
        let pick_depth_texture = Texture2d::new(
            &TextureDescriptor::default_settings(D2::new(1, 1)),
            instance,
        );

        let egui_texture = Texture2d::new(
            &TextureDescriptor::default_settings(D2::new(1, 1)),
            instance,
        );
        let egui_sampler = Sampler::new(&Default::default(), instance);

        let mut egui_textures = HashMap::new();

        egui_textures.insert(
            0,
            Texture2d::new(
                &TextureDescriptor::default_settings(D2::new(1, 1)),
                instance,
            ),
        );
        egui_textures.insert(
            1,
            Texture2d::new(
                &TextureDescriptor::default_settings(D2::new(1, 1)),
                instance,
            ),
        );

        Self {
            egui_pipeline,
            egui_ctx: CtxRef::default(),
            egui_raw_input: RawInput::default(),
            egui_texture_version: None,
            egui_texture,
            egui_sampler,
            egui_point_pos: None,
            egui_textures,
            pick_pipeline,
            pick_texture,
            pick_depth_texture,
            game: None,
            project: Project::new(opts.project_path).unwrap(),
            building: None,
            selection: Selection::None,
            viewports: vec![
                Viewport {
                    texture_id: 0,
                    ty: ViewportType::Editor {
                        camera: Camera::new(),
                    },
                },
                Viewport {
                    texture_id: 1,
                    ty: ViewportType::Game,
                },
            ],
        }
    }

    pub fn build(&mut self) -> std::io::Result<()> {
        let mut command = std::process::Command::new("cargo");
        command.arg("build");

        #[cfg(not(debug_assertions))]
        command.arg("--release");

        command
            .arg("--manifest-path")
            .arg(&self.project.path.join("Cargo.toml"));

        command.stdout(Stdio::piped()).stderr(Stdio::piped());

        let child = command.spawn()?;

        self.building = Some(child);

        log::info!("Building project!");

        Ok(())
    }

    pub fn start_game(&mut self, instance: &Instance) {
        self.save_scene();

        if let Some(game) = &mut self.game {
            game.running = true;

            log::debug!("running game");

            if let Some(state) = &mut game.state {
                state.start(TARGET_FORMAT, instance);
            }
        }
    }

    pub fn load_scene(&self) -> Option<Vec<u8>> {
        let path = self.project.path.join("scene.scn");

        log::debug!("loading scene from: {}", path.display());

        if let Ok(scene) = std::fs::read(path) {
            Some(scene)
        } else {
            None
        }
    }

    pub fn save_scene(&self) {
        if let Some(game) = &self.game {
            if !game.running {
                let path = self.project.path.join("scene.scn");

                log::debug!("saving scene to: {}", path.display());

                if let Ok(file) = std::fs::File::create(path) {
                    let mut serializer = ron::Serializer::new(file, Some(ron::ser::PrettyConfig::default()), true).unwrap();
                        //serde_cbor::Serializer::new(serde_cbor::ser::IoWrite::new(file));

                    if let Some(state) = &game.state {
                        state.serialize_tree(&mut serializer).unwrap();
                    }
                }
            }
        }
    }

    pub fn reload_game(&mut self, scene: &[u8], instance: &Instance) {
        if let Some(game) = &mut self.game {
            let mut deserializer = ron::Deserializer::from_bytes(scene).unwrap();
                //serde_cbor::Deserializer::from_slice(scene);

            game.reload(&mut deserializer, instance);

            if let Some(state) = &mut game.state {
                state.editor_start(TARGET_FORMAT, instance);
            }
        } else {
            self.load(Some(scene), instance);
        }
    }

    pub fn load(&mut self, scene: Option<&[u8]>, instance: &Instance) {
        let mut game = if let Some(scene) = scene {
            let mut deserializer = ron::Deserializer::from_bytes(scene).unwrap();
                //serde_cbor::Deserializer::from_slice(scene);

            GameState::deserialize(
                &mut deserializer,
                &self.project.path.join(LIB_PATH).join("testproject.dll"),
                instance,
            )
        } else {
            GameState::load(
                &self.project.path.join(LIB_PATH).join("testproject.dll"),
                instance,
            )
        };

        if let Some(state) = &mut game.state {
            state.editor_start(TARGET_FORMAT, instance);
        }

        self.game = Some(game);
    }
}

impl State for EditorState {
    fn update(&mut self, ctx: UpdateCtx<'_>) -> Trans {
        let size = ctx.window.size();
        let size = egui::Vec2::new(size.x, size.y);
        self.egui_raw_input.screen_rect = Some(Rect::from_min_size(Default::default(), size));
        self.egui_raw_input.predicted_dt = ctx.delta_time;
        self.egui_point_pos = Some(ctx.mouse.position);

        if let Some(building) = &mut self.building {
            let exit_status = building.try_wait().unwrap();

            if let Some(exit_status) = exit_status {
                if exit_status.success() {
                    log::info!("Build finished successfully!");
                    log::info!("Loading build");

                    let scene = self.load_scene();

                    self.load(scene.as_ref().map(|s| s.as_ref()), ctx.instance);

                    log::info!("Build loaded!");
                } else {
                    log::error!("Build failed!");
                }

                self.building = None;
            }
        }

        if let Some(game) = &mut self.game {
            if let Some(state) = &mut game.state {
                if game.running {
                    state.update(TARGET_FORMAT, ctx.instance);
                } else {
                    state.editor_update(TARGET_FORMAT, ctx.instance);
                }
            }
        }

        self.project.update_files().unwrap();

        Trans::None
    }

    fn handle_event(&mut self, _instance: &Instance, event: &event::Event<()>) -> Trans {
        match event {
            event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::MouseWheel { delta, .. } => {
                    let delta = match delta {
                        MouseScrollDelta::LineDelta(x, y) => egui::Vec2::new(*x, *y),
                        MouseScrollDelta::PixelDelta(pos) => {
                            egui::Vec2::new(pos.x as f32, pos.y as f32)
                        }
                    };

                    self.egui_raw_input.scroll_delta = delta * 10.0;
                    self.egui_raw_input.zoom_delta = delta.y;
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(keycode) = input.virtual_keycode {
                        let pressed = match &input.state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };

                        let key = match keycode {
                            VKey::Back => Some(Key::Backspace),
                            VKey::Return => Some(Key::Enter),
                            VKey::Space => Some(Key::Space),
                            VKey::Left => Some(Key::ArrowLeft),
                            VKey::Right => Some(Key::ArrowRight),
                            VKey::Up => Some(Key::ArrowUp),
                            VKey::Down => Some(Key::ArrowDown),
                            VKey::Key0 => Some(Key::Num0),
                            VKey::Key1 => Some(Key::Num1),
                            VKey::Key2 => Some(Key::Num2),
                            VKey::Key3 => Some(Key::Num3),
                            VKey::Key4 => Some(Key::Num4),
                            VKey::Key5 => Some(Key::Num5),
                            VKey::Key6 => Some(Key::Num6),
                            VKey::Key7 => Some(Key::Num7),
                            VKey::Key8 => Some(Key::Num8),
                            VKey::Key9 => Some(Key::Num9),
                            VKey::A => Some(Key::A),
                            VKey::B => Some(Key::B),
                            VKey::C => Some(Key::C),
                            VKey::D => Some(Key::D),
                            VKey::E => Some(Key::E),
                            VKey::F => Some(Key::F),
                            VKey::G => Some(Key::G),
                            VKey::H => Some(Key::H),
                            VKey::I => Some(Key::I),
                            VKey::J => Some(Key::J),
                            VKey::K => Some(Key::K),
                            VKey::L => Some(Key::L),
                            VKey::M => Some(Key::M),
                            VKey::N => Some(Key::N),
                            VKey::O => Some(Key::O),
                            VKey::P => Some(Key::P),
                            VKey::Q => Some(Key::Q),
                            VKey::R => Some(Key::R),
                            VKey::S => Some(Key::S),
                            VKey::T => Some(Key::T),
                            VKey::U => Some(Key::U),
                            VKey::V => Some(Key::V),
                            VKey::W => Some(Key::W),
                            VKey::X => Some(Key::X),
                            VKey::Y => Some(Key::Y),
                            VKey::Z => Some(Key::Z),
                            _ => None,
                        };

                        if let Some(key) = key {
                            self.egui_raw_input.events.push(Event::Key {
                                key,
                                pressed,
                                modifiers: self.egui_raw_input.modifiers,
                            });
                        }
                    }
                }
                WindowEvent::ReceivedCharacter(c) => {
                    let c = *c;
                    if !c.is_control() {
                        self.egui_raw_input.events.push(Event::Text(c.to_string()));
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let position = Pos2::new(position.x as f32, position.y as f32);

                    self.egui_raw_input
                        .events
                        .push(Event::PointerMoved(position));
                }
                WindowEvent::CursorLeft { .. } => {
                    self.egui_raw_input.events.push(Event::PointerGone);
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    let button = match button {
                        MouseButton::Left => Some(PointerButton::Primary),
                        MouseButton::Middle => Some(PointerButton::Middle),
                        MouseButton::Right => Some(PointerButton::Secondary),
                        _ => None,
                    };

                    let pressed = match state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };

                    if let Some(button) = button {
                        if let Some(pos) = self.egui_point_pos {
                            self.egui_raw_input.events.push(Event::PointerButton {
                                pos: Pos2::new(pos.x, pos.y),
                                button,
                                pressed,
                                modifiers: self.egui_raw_input.modifiers.clone(),
                            });
                        }
                    }
                }
                WindowEvent::ModifiersChanged(modifiers) => {
                    self.egui_raw_input.modifiers.alt = modifiers.alt();
                    self.egui_raw_input.modifiers.ctrl = modifiers.ctrl();
                    self.egui_raw_input.modifiers.shift = modifiers.shift();
                    self.egui_raw_input.modifiers.command = modifiers.logo();
                    self.egui_raw_input.modifiers.mac_cmd = modifiers.logo();
                }
                _ => {}
            },
            _ => {}
        }

        Trans::None
    }

    fn render(&mut self, instance: &Instance, target: TextureView) {
        if let Some(game) = &mut self.game {
            if game.state.is_none() {
                drop(self.game.take());
            }
        }

        let size = self
            .egui_raw_input
            .screen_rect
            .unwrap_or(Rect::from_min_size(Default::default(), egui::Vec2::ZERO));
        let screen_size = Vec2::new(size.width(), size.height());
        self.egui_pipeline.bind_uniform("ScreenSize", &screen_size);

        self.egui_ctx.begin_frame(self.egui_raw_input.take());

        self.ui(instance);

        let (_output, shapes) = self.egui_ctx.end_frame();
        let clipped_meshes = self.egui_ctx.tessellate(shapes);

        let egui_texture = self.egui_ctx.texture();

        if self.egui_texture_version != Some(egui_texture.version) {
            self.egui_texture_version = Some(egui_texture.version);

            self.egui_texture = Texture2d::new(
                &TextureDescriptor::default_settings(D2::new(
                    egui_texture.width as u32,
                    egui_texture.height as u32,
                )),
                instance,
            );

            self.egui_texture.write(instance, |data| {
                let mut pixels = egui_texture.srgba_pixels();

                for x in 0..egui_texture.width {
                    for y in 0..egui_texture.height {
                        let color = Rgba::from(pixels.next().unwrap());
                        data[x][y] = Color::rgba(color.r(), color.g(), color.b(), color.a());
                    }
                }
            });

            self.egui_pipeline.bind("tex", self.egui_texture.view());
            self.egui_pipeline
                .bind("tex_sampler", self.egui_sampler.clone());
        }

        let mut render_ctx = instance.render();

        for viewport in &self.viewports {
            if let Some(texture) = self.egui_textures.get(&viewport.texture_id) {
                if let Some(game) = &mut self.game {
                    if let Some(state) = &mut game.state {
                        let view = texture.view().map_format(|_| TARGET_FORMAT);

                        match &viewport.ty {
                            ViewportType::Editor { camera } => {
                                if self.pick_texture.dimensions.width != texture.dimensions.width
                                    || self.pick_texture.dimensions.height
                                        != texture.dimensions.height
                                {
                                    self.pick_texture = Texture2d::new(
                                        &TextureDescriptor::default_settings(
                                            texture.dimensions.clone(),
                                        ),
                                        instance,
                                    );

                                    self.pick_depth_texture = Texture2d::new(
                                        &TextureDescriptor::default_settings(
                                            texture.dimensions.clone(),
                                        ),
                                        instance,
                                    );
                                }

                                state.viewport_pick_render(
                                    &camera.view_proj(),
                                    &self.pick_pipeline,
                                    &self.pick_texture,
                                    &self.pick_depth_texture,
                                    &mut render_ctx,
                                    instance,
                                );

                                state.viewport_render(
                                    &Some(camera.view_proj()),
                                    view,
                                    &mut render_ctx,
                                    instance,
                                );
                            }
                            ViewportType::Game => {
                                state.render(view, &mut render_ctx, instance);
                            }
                        }
                    }
                }
            }
        }

        let desc = RenderPassDescriptor::default_settings(target);

        let mut pass = render_ctx.render_pass(&desc, &self.egui_pipeline);

        for ClippedMesh(rect, mesh) in &clipped_meshes {
            // TODO: use scissor rect
            let clip_rect = Vec4::new(rect.min.x, rect.min.y, rect.max.x, rect.max.y);
            self.egui_pipeline.bind_uniform("ClipRect", &clip_rect);

            match &mesh.texture_id {
                TextureId::Egui => self.egui_pipeline.bind("tex", self.egui_texture.view()),
                TextureId::User(id) => {
                    if let Some(texture) = self.egui_textures.get(id) {
                        self.egui_pipeline.bind("tex", texture.view());
                    }
                }
            }

            let indices = mesh.indices.clone();
            let mut pos = Vec::with_capacity(mesh.vertices.len());
            let mut uv = Vec::with_capacity(mesh.vertices.len());
            let mut color = Vec::with_capacity(mesh.vertices.len());

            for vertex in &mesh.vertices {
                let v_color = Rgba::from(vertex.color);

                pos.push(Vec2::new(vertex.pos.x, vertex.pos.y));
                uv.push(Vec2::new(vertex.uv.x, vertex.uv.y));

                if let TextureId::Egui = mesh.texture_id {
                    color.push(Color::rgba(
                        v_color.r() * 4.0,
                        v_color.g() * 4.0,
                        v_color.b() * 4.0,
                        v_color.a() * 4.0,
                    ));
                } else {
                    color.push(Color::rgba(
                        v_color.r(),
                        v_color.g(),
                        v_color.b(),
                        v_color.a(),
                    ));
                }
            }

            let mut mesh = Mesh::new();
            mesh.set_attribute("pos", pos);
            mesh.set_attribute("uv", uv);
            mesh.set_attribute("color", color);
            mesh.set_indices(indices);

            pass.draw_mesh(&mesh);
        }
    }
}
