use crate::editor_state::*;
use egui::*;
use quartz_engine::render::prelude::*;

impl EditorState {
    pub fn ui(&mut self, render_resource: &mut RenderResource) {
        let input = self.egui_ctx.input();

        if input.key_pressed(Key::S) && input.modifiers.ctrl {
            self.save_scene();
        }

        self.top_panel_ui(render_resource);
        self.left_panel_ui();
        self.inspector_panel_ui(render_resource);
        self.viewport_ui(render_resource);
    }

    pub fn top_panel_ui(&mut self, render_resource: &mut RenderResource) {
        let game = &mut self.game;
        let building = &self.building;
        let mut reload = false;
        let mut start = false;

        let build = TopPanel::top("top_panel")
            .show(&self.egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    let _file_response = ui.button("File");

                    let build_response = ui.add(Button::new("Build").enabled(building.is_none()));

                    if let Some(game) = game {
                        if game.running {
                            if ui.button("Stop").clicked() {
                                game.running = false;
                                reload = true;
                            }
                        } else {
                            if ui.button("Start").clicked() {
                                start = true;
                            }
                        }
                    }

                    build_response.clicked()
                })
                .inner
            })
            .inner;

        if build {
            self.save_scene();

            drop(self.game.take());

            self.build().unwrap();
        }

        if reload {
            let scene = self.load_scene().unwrap();

            self.reload_game(&scene, render_resource);
        }

        if start {
            self.start_game(render_resource);
        }
    }

    pub fn left_panel_ui(&mut self) {
        let files = &mut self.project.files;
        let game = &mut self.game;
        let selected_node = &mut self.selected_node;

        SidePanel::left("left_panel", 200.0).show(&self.egui_ctx, |ui| {
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
                            selected_node,
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
    }

    pub fn inspector_panel_ui(&mut self, render_resource: &mut RenderResource) {
        if let Some(game) = &mut self.game {
            if let Some(selected_node) = self.selected_node {
                if let Some(mut node) = game.state.tree.get_node(&selected_node) {
                    if let Some(render_texture) = self.egui_textures.get(&0) {
                        SidePanel::left("inspector_panel", 300.0).show(&self.egui_ctx, |ui| {
                            render_resource.target_texture(render_texture);

                            node.inspector_ui(
                                &game.state.plugins,
                                &selected_node,
                                &mut game.state.tree,
                                render_resource,
                                ui,
                            );

                            render_resource.target_swapchain();
                        });
                    }
                } else {
                    self.selected_node = None;
                }
            }
        }
    }

    pub fn viewport_ui(&mut self, render_resource: &mut RenderResource) {
        let textures = &mut self.egui_textures;
        let game = &mut self.game;
        let viewports = &mut self.viewports;

        CentralPanel::default().show(&self.egui_ctx, |ui| {
            if game.is_some() {
                let mut view_port_size = ui.available_size();
                view_port_size.y /= viewports.len() as f32;

                for viewport in viewports {
                    if let Some(render_texture) = textures.get_mut(&viewport.texture_id) {
                        let response = ui.add(
                            widgets::Image::new(
                                TextureId::User(viewport.texture_id),
                                view_port_size,
                            )
                            .sense(Sense::drag()),
                        );

                        let view_port_width = view_port_size.x.floor() as u32;
                        let view_port_height = view_port_size.y.floor() as u32;

                        let aspect = view_port_width as f32 / view_port_height as f32;

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

                        if let ViewportType::Editor { camera } = &mut viewport.ty {
                            camera.projection.aspect = aspect;

                            if response.dragged_by(PointerButton::Middle) {
                                let local_x = camera.transform.rotation * Vec3::X;
                                let local_y = camera.transform.rotation * Vec3::Y;
                                let local_z = camera.transform.rotation * Vec3::Z;

                                if ui.input().modifiers.shift {
                                    camera.transform.translation -=
                                        local_x * ui.input().pointer.delta().x * 0.01;
                                    camera.transform.translation +=
                                        local_y * ui.input().pointer.delta().y * 0.01;
                                } else {
                                    let delta = ui.input().pointer.delta();
                                    camera.euler.x -= delta.x * 0.002;
                                    camera.euler.y -= delta.y * 0.002;

                                    camera.transform.rotation = Quat::from_euler(
                                        EulerRot::YXZ,
                                        camera.euler.x,
                                        camera.euler.y,
                                        0.0,
                                    );

                                    if ui.input().key_down(Key::W) {
                                        camera.transform.translation -= local_z;
                                    }

                                    if ui.input().key_down(Key::S) {
                                        camera.transform.translation += local_z;
                                    }

                                    if ui.input().key_down(Key::A) {
                                        camera.transform.translation -= local_x;
                                    }

                                    if ui.input().key_down(Key::D) {
                                        camera.transform.translation += local_x;
                                    }
                                }
                            }

                            if response.hovered() {
                                let local_z = camera.transform.rotation * Vec3::Z;
                                let delta = ui.input().scroll_delta.y * 0.05;

                                camera.transform.translation -= local_z * delta;
                            }
                        }
                    }
                }
            }
        });
    }
}
