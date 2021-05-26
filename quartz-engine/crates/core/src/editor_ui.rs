use crate::component::*;
use crate::node::*;
use crate::plugin::*;
use crate::tree::*;
use egui::*;

impl Tree {
    fn add_node_popup(
        &mut self,
        components: &Components,
        plugins: &Plugins,
        selected_node: &mut Option<NodeId>,
        ui: &mut Ui,
    ) {
        ui.set_max_width(200.0);

        ScrollArea::from_max_height(300.0).show(ui, |ui| {
            for component in components.components() {
                if ui.button(component).clicked() {
                    let component = components.init_short_name(component, plugins).unwrap();

                    if let Some(node) = selected_node {
                        self.spawn_child(component, *node);
                    } else {
                        self.spawn(component);
                    }
                }
            }
        });
    }

    pub fn nodes_ui(
        &mut self,
        ui: &mut Ui,
        components: &Components,
        plugins: &Plugins,
        selected_node: &mut Option<NodeId>,
    ) {
        let popup_id = ui.make_persistent_id("add_node_popup");

        ui.separator();

        for id in self.base.clone() {
            self.node_ui(&id, components, plugins, ui, selected_node);
        }

        let add_node_response = ui.button("+");

        if add_node_response.clicked() {
            *selected_node = None;
            ui.memory().toggle_popup(popup_id);
        }

        if selected_node.is_none() {
            popup::popup_below_widget(ui, popup_id, &add_node_response, |ui| {
                self.add_node_popup(components, plugins, selected_node, ui);
            });
        }

        if add_node_response.hovered() && ui.input().pointer.any_released() {
            if let Some(dragged) = ui
                .memory()
                .id_data_temp
                .get_or_default::<Option<NodeId>>(Id::new("tree_drag"))
            {
                self.set_parent(dragged, None);
            }
        }

        if ui.input().pointer.any_released() {
            ui.memory()
                .id_data_temp
                .insert::<Option<NodeId>>(Id::new("tree_drag"), None);
        }
    }

    pub fn node_ui(
        &mut self,
        node_id: &NodeId,
        components: &Components,
        plugins: &Plugins,
        ui: &mut Ui,
        selected_node: &mut Option<NodeId>,
    ) {
        if let Some(node) = self.get_node(node_id) {
            let selected = *selected_node == Some(*node_id);

            let children = self.get_children(*node_id).clone();

            let popup_id = ui.make_persistent_id("add_node_popup");

            let (response, add_response) = ui
                .horizontal(|ui| {
                    let response = ui.add(Button::new(&node.name).sense(Sense::click_and_drag()));

                    if response.clicked() {
                        *selected_node = Some(*node_id);
                    }

                    let add_response = ui.button("+");

                    if add_response.clicked() {
                        *selected_node = Some(*node_id);

                        ui.memory().toggle_popup(popup_id);
                    }

                    if ui.button("-").clicked() {
                        self.despawn(*node_id);
                    }

                    (response, add_response)
                })
                .inner;

            if response.drag_started() {
                ui.memory()
                    .id_data_temp
                    .insert(Id::new("tree_drag"), Some(*node_id));
            }

            let dragged = {
                ui.memory()
                    .id_data_temp
                    .get_or_default::<Option<NodeId>>(Id::new("tree_drag"))
                    .clone()
            };

            if response.hovered() && ui.input().pointer.any_released() {
                if let Some(dragged) = dragged {
                    if dragged != *node_id {
                        self.set_parent(dragged, node_id);
                    }
                }
            }

            let as_child = {
                if let Some(dragged) = dragged {
                    let is_child = children.iter().find(|c| **c == dragged).is_some();

                    ui.rect_contains_pointer(response.rect) && !is_child && dragged != *node_id
                } else {
                    false
                }
            };

            if !children.is_empty() || as_child {
                ui.vertical(|ui| {
                    ui.indent(node_id, |ui| {
                        if as_child {
                            self.node_ui(&dragged.unwrap(), components, plugins, ui, selected_node);
                        }

                        for child in children {
                            self.node_ui(&child, components, plugins, ui, selected_node);
                        }
                    });
                });
            };

            if ui.input().key_pressed(Key::A) && ui.input().modifiers.ctrl && selected {
                ui.memory().toggle_popup(popup_id);
            }

            if selected {
                popup::popup_below_widget(ui, popup_id, &add_response, |ui| {
                    self.add_node_popup(components, plugins, selected_node, ui);
                });
            }
        }
    }
}
