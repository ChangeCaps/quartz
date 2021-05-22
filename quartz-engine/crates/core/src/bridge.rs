use crate::component::*;
use crate::game_state::*;
use crate::plugin::*;
use crate::tree::*;
use crate::types::*;
use libloading::*;
use quartz_render::prelude::*;
use serde::{de::DeserializeSeed, Deserializer};

pub struct Bridge {
    lib: Library,
}

impl Bridge {
    pub unsafe fn load(path: &std::path::Path) -> Result<Self, Error> {
        let lib = Library::new(path)?;

        Ok(Self { lib })
    }

    pub fn new(&self, render_resource: &RenderResource) -> Result<GameState, Error> {
        let new: Symbol<fn(&RenderResource) -> (Components, Plugins)> =
            unsafe { self.lib.get(b"new") }?;

        let (components, plugins) = new(render_resource);

        let tree = Tree::new();

        Ok(GameState::new(tree, plugins, components, render_resource))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        &self,
        deserializer: D,
        render_resource: &RenderResource,
    ) -> Result<GameState, Error> {
        let new: Symbol<fn(&RenderResource) -> (Components, Plugins)> =
            unsafe { self.lib.get(b"new") }?;

        let (components, plugins) = new(render_resource);

        let tree = crate::reflect::serde::TreeDeserializer {
            components: &components,
            plugins: &plugins,
        }
        .deserialize(deserializer)
        .unwrap();

        Ok(GameState::new(tree, plugins, components, render_resource))
    }
}
