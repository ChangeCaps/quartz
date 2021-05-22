pub use quartz_engine_core as core;
pub use quartz_engine_core::egui;
pub use quartz_engine_core::register_types;
pub use quartz_engine_core::render;
use quartz_engine_core::types::Types;
pub use quartz_engine_derive as derive;

#[cfg(feature = "builtins")]
pub fn register_builtin_types(types: &mut Types) {
    quartz_engine_builtins::register_types(types);
}

pub mod prelude {
    pub use crate::core::prelude::*;

    #[cfg(feature = "builtins")]
    pub use quartz_engine_builtins::*;
}
