use crate::node::*;
use crate::plugin::*;
use crate::reflect::*;
use crate::tree::*;
use egui::Ui;
use quartz_render::prelude::*;
use std::collections::HashMap;

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
    pub tree: &'a Tree,
    pub node_id: &'a NodeId,
    pub transform: &'a Transform,
    pub global_transform: &'a Transform,
    pub render_pass: &'a mut EmptyRenderPass<'b, 'c, format::TargetFormat, format::Depth32Float>,
}

pub trait InitComponent: Component {
    fn init(state: <Self::Plugins as PluginFetch<'_>>::Item) -> Self;
}

impl<T> InitComponent for T
where
    T: Default + Component,
{
    fn init(_: <T::Plugins as PluginFetch<'_>>::Item) -> Self {
        Default::default()
    }
}

#[allow(unused_variables)]
pub trait Component: 'static {
    type Plugins: for<'a> PluginFetch<'a>;

    fn name() -> &'static str
    where
        Self: Sized;

    fn inspector_ui(
        &mut self,
        plugins: <Self::Plugins as PluginFetch<'_>>::Item,
        ctx: ComponentCtx,
        ui: &mut Ui,
    ) {
    }

    fn update(&mut self, plugins: <Self::Plugins as PluginFetch<'_>>::Item, ctx: ComponentCtx) {}

    fn editor_update(
        &mut self,
        plugins: <Self::Plugins as PluginFetch<'_>>::Item,
        ctx: ComponentCtx,
    ) {
    }

    fn render(
        &mut self,
        plugins: <Self::Plugins as PluginFetch<'_>>::Item,
        ctx: ComponentRenderCtx,
    ) {
    }

    fn viewport_render(
        &mut self,
        plugins: <Self::Plugins as PluginFetch<'_>>::Item,
        ctx: ComponentRenderCtx,
    ) {
        self.render(plugins, ctx);
    }

    fn despawn(&mut self, plugins: <Self::Plugins as PluginFetch<'_>>::Item, ctx: ComponentCtx) {}
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

pub trait ComponentPod: Reflect + 'static {
    fn name(&self) -> &str;
    fn inspector_ui(&mut self, plugins: &Plugins, ctx: ComponentCtx, ui: &mut Ui);
    fn update(&mut self, plugins: &Plugins, ctx: ComponentCtx);
    fn editor_update(&mut self, plugins: &Plugins, ctx: ComponentCtx);
    fn render(&mut self, plugins: &Plugins, ctx: ComponentRenderCtx);
    fn despawn(&mut self, plugins: &Plugins, ctx: ComponentCtx);
}

impl<T: Component + Reflect> ComponentPod for T {
    fn name(&self) -> &str {
        T::name()
    }

    fn inspector_ui(&mut self, plugins: &Plugins, ctx: ComponentCtx, ui: &mut Ui) {
        T::Plugins::fetch(plugins, |plugins| {
            Component::inspector_ui(self, plugins, ctx, ui);
        });
    }

    fn update(&mut self, plugins: &Plugins, ctx: ComponentCtx) {
        T::Plugins::fetch(plugins, |plugins| {
            Component::update(self, plugins, ctx);
        });
    }

    fn editor_update(&mut self, plugins: &Plugins, ctx: ComponentCtx) {
        T::Plugins::fetch(plugins, |plugins| {
            Component::editor_update(self, plugins, ctx);
        });
    }

    fn render(&mut self, plugins: &Plugins, ctx: ComponentRenderCtx) {
        T::Plugins::fetch(plugins, |plugins| {
            Component::render(self, plugins, ctx);
        });
    }

    fn despawn(&mut self, plugins: &Plugins, ctx: ComponentCtx) {
        T::Plugins::fetch(plugins, |plugins| {
            Component::despawn(self, plugins, ctx);
        });
    }
}

pub struct Components {
    pub inits: HashMap<&'static str, Box<fn(&Plugins) -> Box<dyn ComponentPod>>>,
}

fn init<C: InitComponent + Reflect>(plugins: &Plugins) -> Box<dyn ComponentPod> {
    C::Plugins::fetch(plugins, |plugins| Box::new(C::init(plugins)))
}

impl Components {
    pub fn new() -> Self {
        Self {
            inits: HashMap::new(),
        }
    }

    pub fn components(&self) -> Vec<&'static str> {
        self.inits.keys().cloned().collect()
    }

    pub fn register_component<C: InitComponent + Reflect>(&mut self) {
        self.inits.insert(C::name(), Box::new(init::<C>));
    }

    pub fn init(&self, component: &str, plugins: &Plugins) -> Option<Box<dyn ComponentPod>> {
        self.inits.get(component).map(|init| init(plugins))
    }
}