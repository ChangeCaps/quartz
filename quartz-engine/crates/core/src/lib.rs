pub mod component;
pub mod inspect;
pub mod macros;
pub mod node;
pub mod plugin;
pub mod reflect;
pub mod scene;
pub mod transform;
pub mod tree;
pub mod types;

#[cfg(feature = "editor_bridge")]
pub mod bridge;
#[cfg(feature = "editor_bridge")]
pub mod editor_ui;
#[cfg(feature = "editor_bridge")]
pub mod game_state;

pub use egui;
pub use erased_serde;
pub use quartz_render as render;
pub use serde;

#[cfg(feature = "editor_bridge")]
pub mod editor_bridge {
    pub use crate::bridge::*;
    pub use crate::game_state::*;
}

pub mod prelude {
    pub use crate::component::{Component, ComponentCtx, ComponentPickCtx, ComponentRenderCtx};
    pub use crate::inspect::Inspect;
    pub use crate::node::*;
    pub use crate::plugin::{Plugin, PluginCtx, PluginInitCtx, PluginRenderCtx, Plugins};
    pub use crate::reflect::Reflect;
    pub use crate::render::prelude::*;
    pub use crate::transform::*;
    pub use crate::tree::Tree;
    pub use crate::types::*;
}

#[macro_export]
macro_rules! crate_path {
    ($($tt:tt)*) => {
        $crate$($tt)*
    };
}
