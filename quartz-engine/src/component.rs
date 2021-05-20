use crate::node::*;
use quartz_render::prelude::*;

pub struct ComponentCtx<'a> {
    //pub global_transform: &'a Transform,
    pub transform: &'a mut Transform,
    pub render_resource: &'a RenderResource,
}

pub struct ComponentRenderCtx<'a, 'b, 'c> {
    //pub global_transform: &'a Transform,
    pub transform: &'a mut Transform,
    pub render_resource: &'a RenderResource,
    pub render_pass: &'a mut EmptyRenderPass<'b, 'c, format::Rgba8UnormSrgb, format::Depth32Float>,
}

pub trait Component: 'static {
    fn inspector_ui(&mut self, _ctx: ComponentCtx, _ui: &mut egui::Ui) {}

    fn update(&mut self, ctx: ComponentCtx);
    fn render(&mut self, ctx: ComponentRenderCtx);
}
