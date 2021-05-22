#[macro_export]
macro_rules! register_types {
    {
        $register_types:path
    } => {
        mod quartz_engine_editor_bridge {
            use super::*;
            use quartz_engine::core::render::render::RenderResource;
            use quartz_engine::core::types::Types;
            use quartz_engine::register_builtin_types;
            use quartz_engine::core::plugin::Plugins;
            use quartz_engine::core::component::Components;

            #[no_mangle]
            pub fn new(render_resource: &RenderResource) -> (Components, Plugins) {
                let mut types = Types::new(render_resource);

                register_builtin_types(&mut types);

                $register_types(&mut types);

                (types.components, types.plugins)
            }
        }
    };
}
