#[macro_export]
macro_rules! register_types {
    {
        $register_types:path
    } => {
        mod quartz_engine_editor_bridge {
            use super::*;
            use quartz_engine::core::render::{instance::Instance, texture_format::TargetFormat};
            use quartz_engine::core::types::Types;
            use quartz_engine::register_builtin_types;
            use quartz_engine::core::plugin::Plugins;
            use quartz_engine::core::component::Components;
            use quartz_engine::core::bridge::InitFunction;

            #[no_mangle]
            pub unsafe extern "C" fn new(types: *mut Types) {
                register_builtin_types(&mut *types);

                $register_types(&mut *types);
            }
        }
    };
}
