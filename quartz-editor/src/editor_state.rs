use crate::project::*;
use egui::Key;
use egui::*;
use quartz_engine::prelude::*;
use quartz_render::{
    framework::*,
    prelude::{Vec2, *},
};
use std::collections::HashMap;
use std::path::Path;
use winit::event::{self, ElementState, MouseScrollDelta, VirtualKeyCode as VKey, WindowEvent};

pub struct GameState {
    pub state: quartz_engine::game_state::GameState,
    pub selected_node: Option<NodeId>,
    pub bridge: Bridge,
}

impl GameState {
    pub fn load(path: impl AsRef<Path>, render_resource: &RenderResource) -> Self {
        let bridge = unsafe { Bridge::load(path.as_ref()) }.unwrap();
        let state = bridge.new(render_resource).unwrap();

        Self {
            state,
            selected_node: None,
            bridge,
        }
    }
}

pub struct EditorState {
    egui_pipeline: RenderPipeline,
    egui_ctx: CtxRef,
    egui_raw_input: RawInput,
    egui_texture_version: Option<u64>,
    egui_texture: Texture2d,
    egui_sampler: Sampler,
    egui_point_pos: Option<Vec2>,
    egui_textures: HashMap<u64, Texture2d>,
    game: Option<GameState>,
    project: Project,
    building: Option<std::process::Child>,
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

    pub fn load(&mut self, render_resource: &RenderResource) {
        let mut state = GameState::load(
            &self.project.path.join("target/release/testproject.dll"),
            render_resource,
        );

        state.state.init(render_resource);

        self.game = Some(state);
    }

    pub fn ui(&mut self, render_resource: &mut RenderResource) {
        let game = &mut self.game;
        let building = &self.building;
        let build = TopPanel::top("top_panel")
            .show(&self.egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    let file_response = ui.button("File");

                    let build_response = ui.add(Button::new("Build").enabled(building.is_none()));

                    if ui.button("Stop").clicked() {
                        *game = None;
                    }

                    build_response.clicked()
                })
                .inner
            })
            .inner;

        if build {
            drop(self.game.take());

            self.build().unwrap();
        }

        let files = &mut self.project.files;
        let game = &mut self.game;

        SidePanel::left("file_panel", 200.0).show(&self.egui_ctx, |ui| {
            ui.separator();

            let available_size = ui.available_size();

            if let Some(game) = game {
                ScrollArea::from_max_height(available_size.y / 2.0)
                    .id_source("nodes_scroll_area")
                    .show(ui, |ui| {
                        game.state.tree.nodes_ui(
                            ui,
                            &game.state.components,
                            &game.state.plugins,
                            &mut game.selected_node,
                        );
                    });

                ui.separator();
            }

            ScrollArea::auto_sized()
                .id_source("file_scroll_area")
                .show(ui, |ui| {
                    files.ui(ui);
                });
        });

        if let Some(game) = &mut self.game {
            if let Some(selected_node) = game.selected_node {
                if let Some(mut node) = game.state.tree.get_node(&selected_node) {
                    SidePanel::left("inspector_panel", 200.0).show(&self.egui_ctx, |ui| {
                        node.inspector_ui(
                            &game.state.plugins,
                            &selected_node,
                            &mut game.state.tree,
                            render_resource,
                            ui,
                        );
                    });
                }
            }
        }

        let textures = &mut self.egui_textures;
        let game = &mut self.game;

        CentralPanel::default().show(&self.egui_ctx, |ui| {
            if game.is_some() {
                if let Some(render_texture) = textures.get_mut(&0) {
                    let view_port_size = ui.available_size();

                    ui.add(widgets::Image::new(TextureId::User(0), view_port_size));

                    let view_port_width = view_port_size.x.floor() as u32;
                    let view_port_height = view_port_size.y.floor() as u32;

                    if view_port_width != render_texture.dimensions.width
                        || view_port_height != render_texture.dimensions.height
                    {
                        *render_texture = Texture2d::new(
                            &TextureDescriptor::default_settings(D2::new(
                                view_port_width,
                                view_port_height,
                            )),
                            render_resource,
                        );
                    }
                }
            }
        });
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

                    self.load(ctx.render_resource);

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
            game.state.update(ctx.render_resource);
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
                            color.a() as f32 / 255.0,
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
                            vertex.color.a() as f32 / 255.0,
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
