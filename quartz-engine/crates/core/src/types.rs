use crate::component::*;
use crate::plugin::*;
use crate::reflect::*;
use quartz_render::{instance::Instance, texture_format::TargetFormat};

pub struct Types<'a> {
    pub instance: &'a Instance,
    pub plugins: Plugins,
    pub components: Components,
    pub target_format: TargetFormat,
}

impl<'a> Types<'a> {
    pub fn new(instance: &'a Instance, target_format: TargetFormat) -> Self {
        Self {
            instance,
            plugins: Plugins::new(),
            components: Components::new(),
            target_format,
        }
    }

    pub fn register_component<C: InitComponent + Reflect>(&mut self) {
        self.components.register_component::<C>()
    }

    pub fn register_plugin<P: Plugin>(&mut self) {
        self.plugins.register_plugin::<P>(PluginInitCtx {
            instance: &self.instance,
            target_format: self.target_format,
        });
    }
}
