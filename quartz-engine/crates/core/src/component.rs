use crate::node::*;
use crate::plugin::*;
use crate::reflect::*;
use crate::transform::*;
use crate::tree::*;
use egui::Ui;
use quartz_render::prelude::*;
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct ComponentCtx<'a> {
    //pub global_transform: &'a Transform,
    pub tree: &'a mut Tree,
    pub plugins: &'a Plugins,
    pub node_id: &'a NodeId,
    pub transform: &'a mut Transform,
    pub global_transform: &'a Transform,
    pub instance: &'a Instance,
}

pub struct ComponentRenderCtx<'a, 'b, 'c, 'd> {
    //pub global_transform: &'a Transform,
    pub viewport_camera: &'a Option<Mat4>,
    pub instance: &'a Instance,
    pub tree: &'a Tree,
    pub plugins: &'a Plugins,
    pub node_id: &'a NodeId,
    pub transform: &'a Transform,
    pub global_transform: &'a Transform,
    pub render_pass: &'a mut EmptyRenderPass<'b, 'c, 'd, format::TargetFormat, format::Depth32Float>,
}

pub struct ComponentPickCtx<'a, 'b, 'c, 'd> {
    pub viewport_camera: &'a Mat4,
    pub instance: &'a Instance,
    pub tree: &'a Tree,
    pub plugins: &'a Plugins,
    pub node_id: &'a NodeId,
    pub transform: &'a Transform,
    pub global_transform: &'a Transform,
    pub render_pass: &'a mut RenderPass<'b, 'c, 'd, format::TargetFormat, format::Depth32Float>,
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

    fn viewport_pick_render(
        &mut self,
        plugins: <Self::Plugins as PluginFetch<'_>>::Item,
        ctx: ComponentPickCtx,
    ) {
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

pub trait ComponentPod: Reflect + Any {
    fn short_name(&self) -> &str;
    fn long_name(&self) -> &str;
    fn inspector_ui(&mut self, plugins: &Plugins, ctx: ComponentCtx, ui: &mut Ui);
    fn update(&mut self, plugins: &Plugins, ctx: ComponentCtx);
    fn editor_update(&mut self, plugins: &Plugins, ctx: ComponentCtx);
    fn render(&mut self, plugins: &Plugins, ctx: ComponentRenderCtx);
    fn viewport_render(&mut self, plugins: &Plugins, ctx: ComponentRenderCtx);
    fn viewport_pick_render(&mut self, plugins: &Plugins, ctx: ComponentPickCtx);
    fn despawn(&mut self, plugins: &Plugins, ctx: ComponentCtx);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_type_id(&self) -> TypeId;
}

impl<T: Component + Reflect> ComponentPod for T {
    fn short_name(&self) -> &str {
        T::short_name_const()
    }

    fn long_name(&self) -> &str {
        T::long_name_const()
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

    fn viewport_render(&mut self, plugins: &Plugins, ctx: ComponentRenderCtx) {
        T::Plugins::fetch(plugins, |plugins| {
            Component::viewport_render(self, plugins, ctx);
        });
    }

    fn viewport_pick_render(&mut self, plugins: &Plugins, ctx: ComponentPickCtx) {
        T::Plugins::fetch(plugins, |plugins| {
            Component::viewport_pick_render(self, plugins, ctx);
        });
    }

    fn despawn(&mut self, plugins: &Plugins, ctx: ComponentCtx) {
        T::Plugins::fetch(plugins, |plugins| {
            Component::despawn(self, plugins, ctx);
        });
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

pub struct Components {
    pub inits_short_name: HashMap<&'static str, Box<fn(&Plugins) -> Box<dyn ComponentPod>>>,
    pub inits_long_name: HashMap<&'static str, Box<fn(&Plugins) -> Box<dyn ComponentPod>>>,
}

fn init<C: InitComponent + Reflect>(plugins: &Plugins) -> Box<dyn ComponentPod> {
    C::Plugins::fetch(plugins, |plugins| Box::new(C::init(plugins)))
}

impl Components {
    pub fn new() -> Self {
        Self {
            inits_short_name: HashMap::new(),
            inits_long_name: HashMap::new(),
        }
    }

    pub fn components(&self) -> Vec<&'static str> {
        self.inits_short_name.keys().cloned().collect()
    }

    pub fn register_component<C: InitComponent + Reflect>(&mut self) {
        self.inits_short_name
            .insert(C::short_name_const(), Box::new(init::<C>));
        self.inits_long_name
            .insert(C::long_name_const(), Box::new(init::<C>));
    }

    pub fn init_short_name(
        &self,
        component: &str,
        plugins: &Plugins,
    ) -> Option<Box<dyn ComponentPod>> {
        self.inits_short_name
            .get(component)
            .map(|init| init(plugins))
    }

    pub fn init_long_name(
        &self,
        component: &str,
        plugins: &Plugins,
    ) -> Option<Box<dyn ComponentPod>> {
        self.inits_long_name
            .get(component)
            .map(|init| init(plugins))
    }
}
