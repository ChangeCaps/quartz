use crate::component::*;
use crate::plugin::*;
use crate::tree::*;
use egui::*;
use quartz_render::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

pub struct Node {
    pub name: String,
    pub transform: Transform,
    pub global_transform: Transform,
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

    pub fn inspector_ui(
        &mut self,
        plugins: &Plugins,
        node_id: &NodeId,
        tree: &mut Tree,
        render_resource: &RenderResource,
        ui: &mut Ui,
    ) {
        ui.text_edit_singleline(&mut self.name);

        ui.horizontal(|ui| {
            if ui.button("Remove").clicked() {
                tree.despawn(*node_id);
            }
        });

        ui.separator();

        let speed = 0.1;

        ui.columns(3, |columns| {
            columns[0].add(DragValue::new(&mut self.transform.translation.x).speed(speed));
            columns[1].add(DragValue::new(&mut self.transform.translation.y).speed(speed));
            columns[2].add(DragValue::new(&mut self.transform.translation.z).speed(speed));
        });

        let (mut yaw, mut pitch, mut roll) = self.transform.rotation.to_euler(EulerRot::XYZ);

        ui.columns(3, |columns| {
            columns[0].add(DragValue::new(&mut yaw).speed(speed));
            columns[1].add(DragValue::new(&mut pitch).speed(speed));
            columns[2].add(DragValue::new(&mut roll).speed(speed));
        });

        self.transform.rotation = Quat::from_euler(EulerRot::XYZ, yaw, pitch, roll);

        ui.columns(3, |columns| {
            columns[0].add(DragValue::new(&mut self.transform.scale.x).speed(speed));
            columns[1].add(DragValue::new(&mut self.transform.scale.y).speed(speed));
            columns[2].add(DragValue::new(&mut self.transform.scale.z).speed(speed));
        });

        ui.separator();

        let ctx = ComponentCtx {
            tree,
            node_id,
            transform: &mut self.transform,
            global_transform: &self.global_transform,
            render_resource,
        };

        self.component.inspector_ui(plugins, ctx, ui);
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

    pub fn render(
        &mut self,
        plugins: &Plugins,
        node_id: &NodeId,
        tree: &mut Tree,
        render_resource: &RenderResource,
        render_pass: &mut EmptyRenderPass<'_, '_, format::Rgba8UnormSrgb, format::Depth32Float>,
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
}
