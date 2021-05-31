use crate::game_state::*;
use crate::tree::*;
use crate::types::*;
use libloading::*;
use quartz_render::prelude::*;
use serde::{de::DeserializeSeed, Deserializer};

pub type InitFunction = fn(*mut Types);

pub struct Bridge {
    lib: Library,
}

impl Bridge {
    pub unsafe fn load(path: &std::path::Path) -> Result<Self, Error> {
        let lib = Library::new(path)?;

        Ok(Self { lib })
    }

    pub fn close(self) -> Result<(), Error> {
        self.lib.close()
    }

    pub fn new(
        &self,
        instance: &Instance,
        target_format: format::TargetFormat,
    ) -> Result<GameState, Error> {
        let new: Symbol<InitFunction> = unsafe { self.lib.get(b"new") }?;

        let mut types = Types::new(instance, target_format);

        new(&mut types as *mut _);

        let tree = Tree::new();

        Ok(GameState::new(
            tree,
            Box::new(types.plugins),
            Box::new(types.components),
            instance,
        ))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        &self,
        deserializer: D,
        instance: &Instance,
        target_format: format::TargetFormat,
    ) -> Result<GameState, Error> {
        let new: Symbol<InitFunction> = unsafe { self.lib.get(b"new") }?;

        let mut types = Types::new(instance, target_format);

        new(&mut types as *mut _);

        let tree = crate::reflect::serde::SceneDeserializer {
            components: &types.components,
            plugins: &mut types.plugins,
        }
        .deserialize(deserializer)
        .unwrap();

        Ok(GameState::new(
            tree,
            Box::new(types.plugins),
            Box::new(types.components),
            instance,
        ))
    }
}
