pub mod app;
pub mod color;
pub mod event;
pub mod input;
pub mod mouse;
pub mod render;
pub mod state;
pub mod transform;
pub mod window;

pub use wgpu;

pub mod framework {
    pub use crate::app::App;
    pub use crate::input::*;
    pub use crate::mouse::*;
    pub use crate::state::{State, Trans, UpdateCtx};
}

pub mod prelude {
    pub use crate::color::*;
    pub use crate::render::*;
    pub use crate::transform::*;
    pub use glam::{swizzles::*, *};
}
