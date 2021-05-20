use crate::node::*;
use crate::plugin::*;
use crate::tree::*;
use quartz_render::prelude::*;

pub struct ComponentCtx<'a> {
    //pub global_transform: &'a Transform,
    pub tree: &'a mut Tree,
    pub node_id: &'a NodeId,
    pub transform: &'a mut Transform,
    pub global_transform: &'a Transform,
    pub render_resource: &'a RenderResource,
}

pub struct ComponentRenderCtx<'a, 'b, 'c> {
    //pub global_transform: &'a Transform,
    pub render_resource: &'a RenderResource,
    pub tree: &'a mut Tree,
    pub node_id: &'a NodeId,
    pub transform: &'a mut Transform,
    pub global_transform: &'a Transform,
    pub render_pass: &'a mut EmptyRenderPass<'b, 'c, format::Rgba8UnormSrgb, format::Depth32Float>,
}

pub trait Init: Component {
    fn init(state: <Self::Plugins as PluginFetch<'_>>::Item) -> Self;
}

impl<T> Init for T
where
    T: Default + Component,
{
    fn init(_: <T::Plugins as PluginFetch<'_>>::Item) -> Self {
        Default::default()
    }
}

pub trait Component: 'static {
    type Plugins: for<'a> PluginFetch<'a>;

    fn name() -> &'static str
    where
        Self: Sized;

    fn inspector_ui(
        &mut self,
        _plugins: <Self::Plugins as PluginFetch<'_>>::Item,
        _ctx: ComponentCtx,
        _ui: &mut egui::Ui,
    ) {
    }

    fn update(&mut self, _plugins: <Self::Plugins as PluginFetch<'_>>::Item, _ctx: ComponentCtx) {}

    fn render(
        &mut self,
        _plugins: <Self::Plugins as PluginFetch<'_>>::Item,
        _ctx: ComponentRenderCtx,
    ) {
    }
}

pub trait ToPod {
    fn to_pod(self) -> Box<dyn ComponentPod>;
}

impl<T: ComponentPod> ToPod for T {
    fn to_pod(self) -> Box<dyn ComponentPod> {
        Box::new(self)
    }
}

impl ToPod for Box<dyn ComponentPod> {
    fn to_pod(self) -> Box<dyn ComponentPod> {
        self
    }
}

pub trait ComponentPod: 'static {
    fn name(&self) -> &str;
    fn inspector_ui(&mut self, plugins: &Plugins, ctx: ComponentCtx, ui: &mut egui::Ui);
    fn update(&mut self, plugins: &Plugins, ctx: ComponentCtx);
    fn render(&mut self, plugins: &Plugins, ctx: ComponentRenderCtx);
}

impl<T: Component> ComponentPod for T {
    fn name(&self) -> &str {
        T::name()
    }

    fn inspector_ui(&mut self, plugins: &Plugins, ctx: ComponentCtx, ui: &mut egui::Ui) {
        T::Plugins::fetch(plugins, |plugins| {
            Component::inspector_ui(self, plugins, ctx, ui);
        });
    }

    fn update(&mut self, plugins: &Plugins, ctx: ComponentCtx) {
        T::Plugins::fetch(plugins, |plugins| {
            Component::update(self, plugins, ctx);
        });
    }

    fn render(&mut self, plugins: &Plugins, ctx: ComponentRenderCtx) {
        T::Plugins::fetch(plugins, |plugins| {
            Component::render(self, plugins, ctx);
        });
    }
}
