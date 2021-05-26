use crate::component::*;
use crate::plugin::*;
use crate::reflect::*;
use quartz_render::instance::Instance;

pub struct Types<'a> {
    pub instance: &'a Instance,
    pub plugins: Plugins,
    pub components: Components,
}

impl<'a> Types<'a> {
    pub fn new(instance: &'a Instance) -> Self {
        Self {
            instance,
            plugins: Plugins::new(),
            components: Components::new(),
        }
    }

    pub fn register_component<C: InitComponent + Reflect>(&mut self) {
        self.components.register_component::<C>()
    }

    pub fn register_plugin<P: Plugin>(&mut self) {
        self.plugins.register_plugin::<P>(PluginInitCtx {
            instance: &self.instance,
        });
    }
}
