use crate::component::*;
use crate::inspect::*;
use crate::plugin::*;
use crate::transform::*;
use crate::tree::*;
use egui::*;
use linked_hash_map::LinkedHashMap;
use quartz_render::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    any::TypeId,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

impl Into<NodeId> for &NodeId {
    fn into(self) -> NodeId {
        *self
    }
}

impl Into<Option<NodeId>> for &NodeId {
    fn into(self) -> Option<NodeId> {
        Some(*self)
    }
}

pub struct NodeComponents {
    pub(crate) components: LinkedHashMap<String, RwLock<Box<dyn ComponentPod>>>,
}

impl NodeComponents {
    pub fn new() -> Self {
        Self {
            components: LinkedHashMap::new(),
        }
    }

    pub fn add_component(&mut self, to_pod: impl ToPod) {
        let component = to_pod.to_pod();
        self.components
            .insert(component.long_name().to_string(), RwLock::new(component));
    }

    pub fn get_component<T: ComponentPod>(&self) -> Option<RwLockReadGuard<Box<T>>> {
        let component = self.components.get(T::long_name_const())?.read().unwrap();

        if component.as_ref().get_type_id() == TypeId::of::<T>() {
            Some(unsafe { std::mem::transmute(component) })
        } else {
            None
        }
    }

    pub fn get_component_mut<T: ComponentPod>(&self) -> Option<RwLockWriteGuard<Box<T>>> {
        let component = self.components.get(T::long_name_const())?.write().unwrap();

        if component.as_ref().get_type_id() == TypeId::of::<T>() {
            Some(unsafe { std::mem::transmute(component) })
        } else {
            None
        }
    }

    pub fn components(&self) -> impl Iterator<Item = RwLockReadGuard<Box<dyn ComponentPod>>> {
        self.components.values().map(|c| c.read().unwrap())
    }

    pub fn components_mut(&self) -> impl Iterator<Item = RwLockWriteGuard<Box<dyn ComponentPod>>> {
        self.components.values().map(|c| c.write().unwrap())
    }
}

pub struct Node {
    pub name: String,
    pub transform: Transform,
    pub(crate) global_transform: Transform,
    pub(crate) components: NodeComponents,
}

impl Node {
    pub fn new() -> Self {
        Self {
            name: String::from("Node"),
            transform: Transform::IDENTITY,
            global_transform: Transform::IDENTITY,
            components: NodeComponents::new(),
        }
    }

    pub fn global_transform(&self) -> &Transform {
        &self.global_transform
    }

    pub fn add_component(&mut self, component: impl ToPod) {
        self.components.add_component(component);
    }

    pub fn get_component<T: ComponentPod>(&self) -> Option<RwLockReadGuard<Box<T>>> {
        self.components.get_component::<T>()
    }

    pub fn get_component_mut<T: ComponentPod>(&self) -> Option<RwLockWriteGuard<Box<T>>> {
        self.components.get_component_mut::<T>()
    }
}

#[cfg(feature = "editor_bridge")]
impl Node {
    pub fn inspector_ui(
        &mut self,
        plugins: &Plugins,
        components: &Components,
        node_id: &NodeId,
        tree: &mut Tree,
        instance: &Instance,
        ui: &mut Ui,
    ) {
        ui.text_edit_singleline(&mut self.name);

        ui.separator();

        ScrollArea::auto_sized().show(ui, |ui| {
            self.transform.inspect(ui);

            let mut remove = Vec::new();

            for component in self.components.components.values() {
                ui.separator();

                ui.horizontal(|ui| {
                    let component = component.read().unwrap();

                    ui.label(component.short_name());
                    
                    if ui.button("-").clicked() {
                        remove.push(component.long_name().to_string());
                    }
                });

                let ctx = ComponentCtx {
                    tree,
                    node_id,
                    plugins,
                    components: &self.components,
                    transform: &mut self.transform,
                    global_transform: &self.global_transform,
                    instance,
                };

                component.write().unwrap().inspector_ui(plugins, ctx, ui);
            }

            for remove in remove {
                self.components.components.remove(&remove);
            }

            ui.separator();

            let add_component_response = ui.button("+");
            let add_component_id = ui.make_persistent_id(node_id);

            if add_component_response.clicked() {
                ui.memory().toggle_popup(add_component_id);
            }

            popup::popup_below_widget(ui, add_component_id, &add_component_response, |ui| {
                ui.set_max_width(300.0);

                for component in components.components() {
                    if ui.button(component).clicked() {
                        let component = components.init_short_name(component, plugins).unwrap();

                        self.add_component(component);
                    }
                }
            });
        });
    }

    pub fn start(
        &mut self,
        plugins: &Plugins,
        node_id: &NodeId,
        tree: &mut Tree,
        instance: &Instance,
    ) {
        for mut component in self.components.components_mut() {
            let ctx = ComponentCtx {
                tree,
                node_id,
                plugins,
                components: &self.components,
                transform: &mut self.transform,
                global_transform: &self.global_transform,
                instance,
            };

            component.start(plugins, ctx);
        }
    }

    pub fn editor_start(
        &mut self,
        plugins: &Plugins,
        node_id: &NodeId,
        tree: &mut Tree,
        instance: &Instance,
    ) {
        for mut component in self.components.components_mut() {
            let ctx = ComponentCtx {
                tree,
                node_id,
                plugins,
                components: &self.components,
                transform: &mut self.transform,
                global_transform: &self.global_transform,
                instance,
            };

            component.editor_start(plugins, ctx);
        }
    }

    pub fn update(
        &mut self,
        plugins: &Plugins,
        node_id: &NodeId,
        tree: &mut Tree,
        instance: &Instance,
    ) {
        for component in self.components.components.values() {
            let ctx = ComponentCtx {
                tree,
                node_id,
                plugins,
                components: &self.components,
                transform: &mut self.transform,
                global_transform: &self.global_transform,
                instance,
            };

            component.write().unwrap().update(plugins, ctx);
        }
    }

    pub fn editor_update(
        &mut self,
        plugins: &Plugins,
        node_id: &NodeId,
        tree: &mut Tree,
        instance: &Instance,
    ) {
        for component in self.components.components.values() {
            let ctx = ComponentCtx {
                tree,
                node_id,
                plugins,
                components: &self.components,
                transform: &mut self.transform,
                global_transform: &self.global_transform,
                instance,
            };

            component.write().unwrap().editor_update(plugins, ctx);
        }
    }

    pub fn render(
        &mut self,
        plugins: &Plugins,
        node_id: &NodeId,
        tree: &mut Tree,
        viewport_camera: &Option<Mat4>,
        instance: &Instance,
        render_pass: &mut EmptyRenderPass<'_, '_, '_, format::TargetFormat, format::Depth32Float>,
    ) {
        for component in self.components.components.values() {
            let ctx = ComponentRenderCtx {
                viewport_camera,
                instance,
                node_id,
                tree,
                plugins,
                components: &self.components,
                transform: &mut self.transform,
                global_transform: &self.global_transform,
                render_pass,
            };

            component.write().unwrap().render(plugins, ctx);
        }
    }

    pub fn viewport_render(
        &mut self,
        plugins: &Plugins,
        node_id: &NodeId,
        tree: &mut Tree,
        viewport_camera: &Option<Mat4>,
        instance: &Instance,
        render_pass: &mut EmptyRenderPass<'_, '_, '_, format::TargetFormat, format::Depth32Float>,
    ) {
        for component in self.components.components.values() {
            let ctx = ComponentRenderCtx {
                viewport_camera,
                instance,
                node_id,
                tree,
                plugins,
                components: &self.components,
                transform: &mut self.transform,
                global_transform: &self.global_transform,
                render_pass,
            };

            component.write().unwrap().viewport_render(plugins, ctx);
        }
    }

    pub fn viewport_pick_render(
        &mut self,
        plugins: &Plugins,
        node_id: &NodeId,
        tree: &mut Tree,
        viewport_camera: &Mat4,
        instance: &Instance,
        render_pass: &mut RenderPass<'_, '_, '_, format::R32Uint, format::Depth32Float>,
    ) {
        for component in self.components.components.values() {
            let ctx = ComponentPickCtx {
                viewport_camera,
                instance,
                node_id,
                tree,
                plugins,
                components: &self.components,
                transform: &mut self.transform,
                global_transform: &self.global_transform,
                render_pass,
            };

            component.write().unwrap().viewport_pick_render(plugins, ctx);
        }
    }

    pub fn despawn(
        &mut self,
        plugins: &Plugins,
        node_id: &NodeId,
        tree: &mut Tree,
        instance: &Instance,
    ) {
        for component in self.components.components.values() {
            let ctx = ComponentCtx {
                tree,
                node_id,
                plugins,
                components: &self.components,
                transform: &mut self.transform,
                global_transform: &self.global_transform,
                instance,
            };

            component.write().unwrap().despawn(plugins, ctx);
        }
    }
}
