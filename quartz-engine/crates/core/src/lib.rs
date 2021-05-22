pub mod component;
pub mod inspect;
pub mod macros;
pub mod node;
pub mod plugin;
pub mod reflect;
pub mod tree;

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
    pub use crate::component::*;
    pub use crate::inspect::Inspect;
    pub use crate::node::*;
    pub use crate::plugin::{Plugin, PluginCtx, PluginInitCtx, Plugins};
    pub use crate::reflect::Reflect;
    pub use crate::render::prelude::*;
    pub use crate::tree::Tree;
}
