pub mod render3d;

use quartz_engine_core::types::Types;
pub use render3d::*;

pub fn register_types(types: &mut Types) {
    render3d::register_types(types);
}
