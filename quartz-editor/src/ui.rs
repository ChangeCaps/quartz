use crate::editor_state::*;
use egui::*;
use quartz_engine::core::node::NodeId;
use quartz_engine::core::plugin::PluginCtx;
use quartz_engine::render::prelude::{Vec2, *};

impl EditorState {
    pub fn ui(&mut self, instance: &Instance) {
        let input = self.egui_ctx.input();

        if input.key_pressed(Key::S) && input.modifiers.ctrl {
            self.save_scene();
        }

        self.top_panel_ui(instance);
        self.left_panel_ui();
        self.inspector_panel_ui(instance);
        self.viewport_ui(instance);
    }

    pub fn top_panel_ui(&mut self, instance: &Instance) {
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

            if let Some(game) = &mut self.game {
                drop(game.state.take());
            }

            self.build().unwrap();
        }

        if reload {
            let scene = self.load_scene().unwrap();

            self.reload_game(&scene, instance);
        }

        if start {
            self.start_game(instance);
        }
    }

    pub fn left_panel_ui(&mut self) {
        let files = &mut self.project.files;
        let game = &mut self.game;
        let selection = &mut self.selection;

        SidePanel::left("left_panel", 200.0).show(&self.egui_ctx, |ui| {
            ui.separator();

            let available_size = ui.available_size();

            if let Some(game) = game {
                if let Some(state) = &mut game.state {
                    ui.collapsing("Plugins", |ui| {
                        ScrollArea::from_max_height(available_size.y / 3.0)
                            .id_source("plugins_scroll_area")
                            .show(ui, |ui| {
                                for plugin_id in state.plugins.plugins() {
                                    state
                                        .plugins
                                        .get_mut_dyn(&plugin_id, |plugin| {
                                            if ui.button(plugin.short_name()).clicked() {
                                                *selection = Selection::Plugin(plugin_id);
                                            }
                                        })
                                        .unwrap();
                                }
                            });
                    });

                    ui.separator();

                    let mut selected_node = if let Selection::Node(node_id) = selection {
                        Some(*node_id)
                    } else {
                        None
                    };

                    ui.collapsing("Nodes", |ui| {
                        ScrollArea::from_max_height(available_size.y / 3.0)
                            .id_source("nodes_scroll_area")
                            .show(ui, |ui| {
                                state.tree.nodes_ui(
                                    ui,
                                    &state.components,
                                    &state.plugins,
                                    &mut selected_node,
                                );
                            });
                    });

                    if let Some(node_id) = selected_node {
                        *selection = Selection::Node(node_id);
                    }

                    ui.separator();
                }
            }

            ScrollArea::auto_sized()
                .id_source("file_scroll_area")
                .show(ui, |ui| {
                    files.ui(ui);
                });
        });
    }

    pub fn inspector_panel_ui(&mut self, instance: &Instance) {
        let egui_ctx = &self.egui_ctx;
        if let Some(game) = &mut self.game {
            if let Some(state) = &mut game.state {
                match &self.selection {
                    Selection::Node(node_id) => {
                        if let Some(mut node) = state.tree.get_node(node_id) {
                            SidePanel::left("inspector_panel", 300.0).show(egui_ctx, |ui| {
                                node.inspector_ui(
                                    &state.plugins,
                                    &state.components,
                                    node_id,
                                    &mut state.tree,
                                    instance,
                                    ui,
                                );
                            });
                        } else {
                            self.selection = Selection::None;
                        }
                    }
                    Selection::Plugin(plugin_id) => {
                        let tree = &mut state.tree;
                        let plugins = &state.plugins;
                        plugins
                            .get_mut_dyn(plugin_id, |plugin| {
                                let ctx = PluginCtx {
                                    tree: tree,
                                    plugins: plugins,
                                    target_format: TARGET_FORMAT,
                                    instance,
                                };

                                SidePanel::left("inspector_panel", 300.0).show(egui_ctx, |ui| {
                                    plugin.inspector_ui(ctx, ui);
                                });
                            })
                            .unwrap();
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn viewport_ui(&mut self, instance: &Instance) {
        let textures = &mut self.egui_textures;
        let game = &mut self.game;
        let viewports = &mut self.viewports;
        let selection = &mut self.selection;
        let pick_texture = &self.pick_texture;

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
                            .sense(Sense::click_and_drag()),
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
                                instance,
                            );
                        }

                        if let ViewportType::Editor { camera } = &mut viewport.ty {
                            if response.clicked_by(PointerButton::Primary) {
                                if let Some(mut pos) = response.interact_pointer_pos() {
                                    pos.x -= response.rect.min.x;
                                    pos.y -= response.rect.min.y;

                                    let x = pos.x.round() as usize;
                                    let y = pos.y.round() as usize;

                                    let id = pick_texture.read(instance, |data| {
                                        let id = data[x][y];

                                        if id < std::u32::MAX {
                                            Some(NodeId(id as u64))
                                        } else {
                                            None
                                        }
                                    });

                                    if let Some(node_id) = id {
                                        *selection = Selection::Node(node_id);
                                    }
                                }
                            }

                            camera.projection.aspect = aspect;

                            if response.dragged_by(PointerButton::Middle) {
                                let local_x = camera.transform.rotation * Vec3::X;
                                let local_y = camera.transform.rotation * Vec3::Y;
                                let local_z = camera.transform.rotation * Vec3::Z;

                                if ui.input().modifiers.shift {
                                    camera.transform.translation -=
                                        local_x * response.drag_delta().x * 0.01;
                                    camera.transform.translation +=
                                        local_y * response.drag_delta().y * 0.01;
                                } else {
                                    let delta = response.drag_delta();
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
