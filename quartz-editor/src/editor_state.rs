use crate::project::*;
use egui::Key;
use egui::*;
use quartz_engine::{editor_bridge::*, prelude::*};
use quartz_render::{
    framework::*,
    prelude::{Vec2, *},
};
use std::collections::HashMap;
use std::path::Path;
use winit::event::{self, ElementState, MouseScrollDelta, VirtualKeyCode as VKey, WindowEvent};

pub struct GameState {
    pub state: quartz_engine::game_state::GameState,
    pub bridge: Bridge,
    pub running: bool,
}

impl GameState {
    pub fn load(path: impl AsRef<Path>, render_resource: &RenderResource) -> Self {
        let bridge = unsafe { Bridge::load(path.as_ref()) }.unwrap();
        let state = bridge.new(render_resource).unwrap();

        Self {
            state,
            bridge,
            running: false,
        }
    }

    pub fn deserialize<'de, D: quartz_engine::serde::Deserializer<'de>>(
        deserializer: D,
        path: impl AsRef<Path>,
        render_resource: &RenderResource,
    ) -> Self {
        let bridge = unsafe { Bridge::load(path.as_ref()) }.unwrap();
        let state = bridge.deserialize(deserializer, render_resource).unwrap();

        Self {
            state,
            bridge,
            running: false,
        }
    }

    pub fn reload<'de, D: quartz_engine::serde::Deserializer<'de>>(
        &mut self,
        deserializer: D,
        render_resource: &RenderResource,
    ) {
        self.state = self
            .bridge
            .deserialize(deserializer, render_resource)
            .unwrap();
    }
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
    pub game: Option<GameState>,
    pub project: Project,
    pub building: Option<std::process::Child>,
    pub selected_node: Option<NodeId>,
}

impl EditorState {
    pub fn new(render_resource: &RenderResource) -> Self {
        log::debug!("Loading egui shader");
        let egui_shader = Shader::from_glsl(
            include_str!("shaders/egui.vert"),
            include_str!("shaders/egui.frag"),
        )
        .unwrap();
        let egui_pipeline = RenderPipeline::new(
            PipelineDescriptor::default_settings(egui_shader),
            render_resource,
        )
        .unwrap();

        let egui_texture = Texture2d::new(
            &TextureDescriptor::default_settings(D2::new(1, 1)),
            render_resource,
        );
        let egui_sampler = Sampler::new(&Default::default(), render_resource);

        let render_texture = Texture2d::new(
            &TextureDescriptor::default_settings(D2::new(500, 500)),
            render_resource,
        );

        let mut egui_textures = HashMap::new();

        egui_textures.insert(0, render_texture);

        Self {
            egui_pipeline,
            egui_ctx: CtxRef::default(),
            egui_raw_input: RawInput::default(),
            egui_texture_version: None,
            egui_texture,
            egui_sampler,
            egui_point_pos: None,
            egui_textures,
            game: None,
            project: Project::new("../testproject").unwrap(),
            building: None,
            selected_node: None,
        }
    }

    pub fn build(&mut self) -> std::io::Result<()> {
        log::info!("Building project");

        let child = std::process::Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("--manifest-path")
            .arg(&self.project.path.join("Cargo.toml"))
            .spawn()?;

        self.building = Some(child);

        Ok(())
    }

    pub fn start_game(&mut self, render_resource: &mut RenderResource) {
        self.save_scene();

        if let Some(game) = &mut self.game {
            game.running = true;

            if let Some(render_texture) = self.egui_textures.get(&0) {
                render_resource.target_texture(render_texture);

                game.state.start(render_resource);

                render_resource.target_swapchain();
            }
        }
    }

    pub fn load_scene(&self) -> Option<String> {
        if let Ok(scene) = std::fs::read_to_string(self.project.path.join("scene.scn")) {
            Some(scene)
        } else {
            None
        }
    }

    pub fn save_scene(&self) {
        if let Some(game) = &self.game {
            if !game.running {
                if let Ok(file) = std::fs::File::create(self.project.path.join("scene.scn")) {
                    let mut serializer =
                        ron::Serializer::new(file, Some(Default::default()), true).unwrap();

                    game.state.serialize_tree(&mut serializer).unwrap();
                }
            }
        }
    }

    pub fn reload_game(&mut self, scene: &str, render_resource: &mut RenderResource) {
        if let Some(game) = &mut self.game {
            let mut deserializer = ron::Deserializer::from_str(scene).unwrap();

            render_resource
                .target_texture(self.egui_textures.get(&0).expect("main texture not found"));

            game.reload(&mut deserializer, render_resource);

            render_resource.target_swapchain();
        } else {
            self.load(Some(scene), render_resource);
        }
    }

    pub fn load(&mut self, scene: Option<&str>, render_resource: &mut RenderResource) {
        if let Some(render_texture) = self.egui_textures.get(&0) {
            render_resource.target_texture(render_texture);

            let mut state = if let Some(scene) = scene {
                let mut deserializer = ron::Deserializer::from_str(scene).unwrap();

                GameState::deserialize(
                    &mut deserializer,
                    &self.project.path.join("target/release/testproject.dll"),
                    render_resource,
                )
            } else {
                GameState::load(
                    &self.project.path.join("target/release/testproject.dll"),
                    render_resource,
                )
            };

            state.state.editor_start(render_resource);

            render_resource.target_swapchain();

            self.game = Some(state);
        }
    }
}

impl quartz_render::framework::State for EditorState {
    fn update(&mut self, ctx: quartz_render::framework::UpdateCtx<'_>) -> Trans {
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

                    self.load(scene.as_ref().map(|s| s.as_ref()), ctx.render_resource);

                    log::info!("Build loaded!");
                } else {
                    log::error!("Build failed!");

                    if let Some(stdout) = building.stdout.take() {
                        log::error!("{:?}", stdout);
                    }
                }

                self.building = None;
            }
        }

        if let Some(game) = &mut self.game {
            if let Some(render_texture) = self.egui_textures.get(&0) {
                ctx.render_resource.target_texture(render_texture);

                if game.running {
                    game.state.update(ctx.render_resource);
                } else {
                    game.state.editor_update(ctx.render_resource);
                }

                ctx.render_resource.target_swapchain();
            }
        }

        self.project.update_files().unwrap();

        Trans::None
    }

    fn handle_event(
        &mut self,
        _render_resource: &RenderResource,
        event: &event::Event<()>,
    ) -> Trans {
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
                        MouseButton::Middle => Some(PointerButton::Primary),
                        MouseButton::Right => Some(PointerButton::Primary),
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

    fn render(&mut self, render_resource: &mut RenderResource) {
        let size = self
            .egui_raw_input
            .screen_rect
            .unwrap_or(Rect::from_min_size(Default::default(), egui::Vec2::ZERO));
        let screen_size = Vec2::new(size.width(), size.height());
        self.egui_pipeline.bind_uniform("ScreenSize", screen_size);

        self.egui_ctx.begin_frame(self.egui_raw_input.take());

        self.ui(render_resource);

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
                render_resource,
            );

            self.egui_texture.write(render_resource, |data| {
                let mut pixels = egui_texture.srgba_pixels();

                for x in 0..egui_texture.width {
                    for y in 0..egui_texture.height {
                        let color = pixels.next().unwrap();
                        data[x][y] = Color::rgba(
                            color.r() as f32 / 255.0,
                            color.g() as f32 / 255.0,
                            color.b() as f32 / 255.0,
                            color.a() as f32 / 255.0 * 2.0,
                        );
                    }
                }
            });

            self.egui_pipeline.bind("tex", self.egui_texture.view());
            self.egui_pipeline
                .bind("tex_sampler", self.egui_sampler.clone());
        }

        if let Some(render_texture) = self.egui_textures.get(&0) {
            if let Some(state) = &mut self.game {
                render_resource.target_texture(render_texture);

                state.state.render(render_resource);

                render_resource.target_swapchain();
            }
        }

        render_resource
            .render(|ctx| {
                let desc = Default::default();

                let mut pass = ctx.render_pass(&desc, &self.egui_pipeline);

                for ClippedMesh(rect, mesh) in &clipped_meshes {
                    // TODO: use scissor rect
                    let clip_rect = Vec4::new(rect.min.x, rect.min.y, rect.max.x, rect.max.y);
                    self.egui_pipeline.bind_uniform("ClipRect", clip_rect);

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
                        pos.push(Vec2::new(vertex.pos.x, vertex.pos.y));
                        uv.push(Vec2::new(vertex.uv.x, vertex.uv.y));
                        color.push(Color::rgba(
                            vertex.color.r() as f32 / 255.0,
                            vertex.color.g() as f32 / 255.0,
                            vertex.color.b() as f32 / 255.0,
                            vertex.color.a() as f32 / 255.0 * 2.0,
                        ));
                    }

                    let mut mesh = Mesh::new();
                    mesh.set_attribute("pos", pos);
                    mesh.set_attribute("uv", uv);
                    mesh.set_attribute("color", color);
                    mesh.set_indices(indices);

                    pass.draw_mesh(&mesh);
                }
            })
            .unwrap();
    }
}
