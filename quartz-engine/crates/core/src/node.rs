use crate::component::*;
use crate::inspect::*;
use crate::plugin::*;
use crate::transform::*;
use crate::tree::*;
use egui::*;
use quartz_render::prelude::*;
use serde::{Deserialize, Serialize};

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

pub struct Node {
    pub name: String,
    pub transform: Transform,
    pub(crate) global_transform: Transform,
    pub(crate) component: Box<dyn ComponentPod>,
}

impl Node {
    pub fn new(component: Box<dyn ComponentPod>) -> Self {
        Self {
            name: String::from(component.short_name()),
            transform: Transform::IDENTITY,
            global_transform: Transform::IDENTITY,
            component: component,
        }
    }

    pub fn global_transform(&self) -> &Transform {
        &self.global_transform
    }

    pub fn get_component<T: ComponentPod>(&self) -> Option<&T> {
        self.component.as_ref().as_any().downcast_ref::<T>()
    }

    pub fn get_component_mut<T: ComponentPod>(&mut self) -> Option<&mut T> {
        self.component.as_mut().as_any_mut().downcast_mut::<T>()
    }

    pub fn components(&self) -> &dyn ComponentPod {
        self.component.as_ref()
    }

    pub fn components_mut(&mut self) -> &mut dyn ComponentPod {
        self.component.as_mut()
    }
}

#[cfg(feature = "editor_bridge")]
impl Node {
    pub fn inspector_ui(
        &mut self,
        plugins: &Plugins,
        node_id: &NodeId,
        tree: &mut Tree,
        instance: &Instance,
        ui: &mut Ui,
    ) {
        ui.text_edit_singleline(&mut self.name);

        ui.separator();

        ScrollArea::auto_sized().show(ui, |ui| {
            self.transform.inspect(ui);

            ui.separator();

            let ctx = ComponentCtx {
                tree,
                node_id,
                plugins,
                transform: &mut self.transform,
                global_transform: &self.global_transform,
                instance,
            };

            self.component.inspector_ui(plugins, ctx, ui);
        });
    }

    pub fn update(
        &mut self,
        plugins: &Plugins,
        node_id: &NodeId,
        tree: &mut Tree,
        instance: &Instance,
    ) {
        let ctx = ComponentCtx {
            tree,
            node_id,
            plugins,
            transform: &mut self.transform,
            global_transform: &self.global_transform,
            instance,
        };

        self.component.update(plugins, ctx);
    }

    pub fn editor_update(
        &mut self,
        plugins: &Plugins,
        node_id: &NodeId,
        tree: &mut Tree,
        instance: &Instance,
    ) {
        let ctx = ComponentCtx {
            tree,
            node_id,
            plugins,
            transform: &mut self.transform,
            global_transform: &self.global_transform,
            instance,
        };

        self.component.editor_update(plugins, ctx);
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
        let ctx = ComponentRenderCtx {
            viewport_camera,
            instance,
            node_id,
            tree,
            plugins,
            transform: &mut self.transform,
            global_transform: &self.global_transform,
            render_pass,
        };

        self.component.render(plugins, ctx);
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
        let ctx = ComponentRenderCtx {
            viewport_camera,
            instance,
            node_id,
            tree,
            plugins,
            transform: &mut self.transform,
            global_transform: &self.global_transform,
            render_pass,
        };

        self.component.viewport_render(plugins, ctx);
    }

    pub fn viewport_pick_render(
        &mut self,
        plugins: &Plugins,
        node_id: &NodeId,
        tree: &mut Tree,
        viewport_camera: &Mat4,
        instance: &Instance,
        render_pass: &mut RenderPass<'_, '_, '_, format::TargetFormat, format::Depth32Float>,
    ) {
        let ctx = ComponentPickCtx {
            viewport_camera,
            instance,
            node_id,
            tree,
            plugins,
            transform: &mut self.transform,
            global_transform: &self.global_transform,
            render_pass,
        };

        self.component.viewport_pick_render(plugins, ctx);
    }

    pub fn despawn(
        &mut self,
        plugins: &Plugins,
        node_id: &NodeId,
        tree: &mut Tree,
        instance: &Instance,
    ) {
        let ctx = ComponentCtx {
            tree,
            node_id,
            plugins,
            transform: &mut self.transform,
            global_transform: &self.global_transform,
            instance,
        };

        self.component.despawn(plugins, ctx);
    }
}
