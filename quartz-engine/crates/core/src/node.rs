use crate::component::*;
use crate::inspect::*;
use crate::plugin::*;
use crate::tree::*;
use egui::*;
use quartz_render::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

pub struct Node {
    pub name: String,
    pub transform: Transform,
    pub(crate) global_transform: Transform,
    pub component: Box<dyn ComponentPod>,
}

impl Node {
    pub fn new(component: Box<dyn ComponentPod>) -> Self {
        Self {
            name: String::from(component.name()),
            transform: Transform::IDENTITY,
            global_transform: Transform::IDENTITY,
            component: component,
        }
    }
}

#[cfg(feature = "editor_bridge")]
impl Node {
    pub fn inspector_ui(
        &mut self,
        plugins: &Plugins,
        node_id: &NodeId,
        tree: &mut Tree,
        render_resource: &RenderResource,
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
                transform: &mut self.transform,
                global_transform: &self.global_transform,
                render_resource,
            };

            self.component.inspector_ui(plugins, ctx, ui);
        });
    }

    pub fn update(
        &mut self,
        plugins: &Plugins,
        node_id: &NodeId,
        tree: &mut Tree,
        render_resource: &RenderResource,
    ) {
        let ctx = ComponentCtx {
            tree,
            node_id,
            transform: &mut self.transform,
            global_transform: &self.global_transform,
            render_resource,
        };

        self.component.update(plugins, ctx);
    }

    pub fn editor_update(
        &mut self,
        plugins: &Plugins,
        node_id: &NodeId,
        tree: &mut Tree,
        render_resource: &RenderResource,
    ) {
        let ctx = ComponentCtx {
            tree,
            node_id,
            transform: &mut self.transform,
            global_transform: &self.global_transform,
            render_resource,
        };

        self.component.editor_update(plugins, ctx);
    }

    pub fn render(
        &mut self,
        plugins: &Plugins,
        node_id: &NodeId,
        tree: &mut Tree,
        render_resource: &RenderResource,
        render_pass: &mut EmptyRenderPass<'_, '_, format::TargetFormat, format::Depth32Float>,
    ) {
        let ctx = ComponentRenderCtx {
            render_resource,
            node_id,
            tree,
            transform: &mut self.transform,
            global_transform: &self.global_transform,
            render_pass,
        };

        self.component.render(plugins, ctx);
    }

    pub fn despawn(
        &mut self,
        plugins: &Plugins,
        node_id: &NodeId,
        tree: &mut Tree,
        render_resource: &RenderResource,
    ) {
        let ctx = ComponentCtx {
            tree,
            node_id,
            transform: &mut self.transform,
            global_transform: &self.global_transform,
            render_resource,
        };

        self.component.despawn(plugins, ctx);
    }
}
