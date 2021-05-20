pub mod bridge;
pub mod component;
pub mod game_state;
pub mod node;
pub mod plugin;
pub mod state;
pub mod tree;

pub use egui;
pub use quartz_render as render;

pub mod prelude {
    pub use crate::bridge::*;
    pub use crate::component::*;
    pub use crate::node::*;
    pub use crate::plugin::{InitCtx, Plugin, Plugins, UpdateCtx};
    pub use crate::render::prelude::*;
    pub use crate::state::*;
    pub use crate::tree::Tree;
}
