use crate::component::*;
use crate::plugin::*;
use crate::render::prelude::*;
use crate::tree::*;
use std::collections::HashMap;

pub struct Components {
    pub inits: HashMap<&'static str, Box<fn(&Plugins) -> Box<dyn ComponentPod>>>,
}

fn init<C: Init>(plugins: &Plugins) -> Box<dyn ComponentPod> {
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

    pub fn register_component<C: Init>(&mut self) {
        self.inits.insert(C::name(), Box::new(init::<C>));
    }

    pub fn init(
        &self,
        component: &'static str,
        plugins: &Plugins,
    ) -> Option<Box<dyn ComponentPod>> {
        self.inits.get(component).map(|init| init(plugins))
    }
}
