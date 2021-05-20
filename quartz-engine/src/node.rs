use crate::component::*;
use egui::*;
use quartz_render::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

pub struct Node {
    pub name: String,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub transform: Transform,
    pub component: Box<dyn Component>,
}

impl Node {
    pub fn new(component: impl Component) -> Self {
        Self {
            name: String::from("Node"),
            parent: None,
            children: Vec::new(),
            transform: Transform::IDENTITY,
            component: Box::new(component),
        }
    }

    pub fn inspector_ui(&mut self, render_resource: &RenderResource, ui: &mut Ui) {
        ui.text_edit_singleline(&mut self.name);

        ui.separator();

        let speed = 0.1;

        ui.columns(3, |columns| {
            columns[0].add(DragValue::new(&mut self.transform.translation.x).speed(speed));
            columns[1].add(DragValue::new(&mut self.transform.translation.y).speed(speed));
            columns[2].add(DragValue::new(&mut self.transform.translation.z).speed(speed));
        });
        ui.columns(3, |columns| {
            columns[0].add(DragValue::new(&mut self.transform.scale.x).speed(speed));
            columns[1].add(DragValue::new(&mut self.transform.scale.y).speed(speed));
            columns[2].add(DragValue::new(&mut self.transform.scale.z).speed(speed));
        });

        ui.separator();

        let ctx = ComponentCtx {
            transform: &mut self.transform,
            render_resource,
        };

        self.component.inspector_ui(ctx, ui);
    }

    pub fn update(&mut self, render_resource: &RenderResource) {
        let ctx = ComponentCtx {
            transform: &mut self.transform,
            render_resource,
        };

        self.component.update(ctx);
    }

    pub fn render(
        &mut self,
        render_resource: &RenderResource,
        render_pass: &mut EmptyRenderPass<'_, '_, format::Rgba8UnormSrgb, format::Depth32Float>,
    ) {
        let ctx = ComponentRenderCtx {
            transform: &mut self.transform,
            render_resource,
            render_pass,
        };

        self.component.render(ctx);
    }
}
