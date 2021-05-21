use crate::component::*;
use crate::node::*;
use crate::plugin::*;
use crate::tree::*;
use egui::*;

impl Tree {
    pub fn nodes_ui(
        &mut self,
        ui: &mut Ui,
        components: &Components,
        plugins: &Plugins,
        selected_node: &mut Option<NodeId>,
    ) {
        for id in self.base.clone() {
            self.node_ui(&id, components, plugins, ui, selected_node);
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

            let popup_id = ui.make_persistent_id("add_component_id");

            let response = ui
                .horizontal(|ui| {
                    if ui.button(&node.name).clicked() {
                        *selected_node = Some(*node_id);
                    }

                    let response = ui.button("+");

                    if response.clicked() {
                        *selected_node = Some(*node_id);

                        ui.memory().toggle_popup(popup_id);
                    }

                    if ui.button("-").clicked() {
                        self.despawn(*node_id);
                    }

                    response
                })
                .inner;

            if !children.is_empty() {
                ui.vertical(|ui| {
                    ui.indent(node_id, |ui| {
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
                popup::popup_below_widget(ui, popup_id, &response, |ui| {
                    ui.set_max_width(200.0);

                    ScrollArea::from_max_height(300.0).show(ui, |ui| {
                        for component in components.components() {
                            if ui.button(component).clicked() {
                                let component = components.init(component, plugins).unwrap();

                                self.spawn_child(component, &selected_node.unwrap());
                            }
                        }
                    });
                });
            }
        }
    }
}
