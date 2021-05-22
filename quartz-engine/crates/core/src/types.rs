use crate::component::*;
use crate::plugin::*;
use crate::reflect::*;
use quartz_render::render::RenderResource;

pub struct Types<'a> {
    pub render_resource: &'a RenderResource,
    pub plugins: Plugins,
    pub components: Components,
}

impl<'a> Types<'a> {
    pub fn new(render_resource: &'a RenderResource) -> Self {
        Self {
            render_resource,
            plugins: Plugins::new(),
            components: Components::new(),
        }
    }

    pub fn register_component<C: InitComponent + Reflect>(&mut self) {
        self.components.register_component::<C>()
    }

    pub fn register_plugin<P: Plugin>(&mut self) {
        self.plugins.register_plugin::<P>(PluginInitCtx {
            render_resource: &self.render_resource,
        });
    }
}
