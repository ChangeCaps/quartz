pub mod app;
pub mod event;
pub mod input;
pub mod mouse;
pub mod state;
pub mod window;

pub use quartz_render as render;
pub use winit;

pub mod framework {}

pub mod prelude {
    pub use crate::app::App;
    pub use crate::input::*;
    pub use crate::mouse::*;
    pub use crate::state::{State, Trans, UpdateCtx};
    pub use quartz_render::prelude::*;
}
