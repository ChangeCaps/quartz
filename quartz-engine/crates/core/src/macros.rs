#[macro_export]
macro_rules! bridge {
    {
        components: { $( $component:path ),+ $(,)? }
        plugins: { $( $plugin:path ),+ $(,)? }
    } => {
        mod new {
            use super::*;

            #[no_mangle]
            pub fn new(render_resource: &quartz_engine::render::prelude::RenderResource) ->
                (quartz_engine::component::Components, quartz_engine::plugin::Plugins)
            {
                let mut components = quartz_engine::component::Components::new();
                let mut plugins = quartz_engine::plugin::Plugins::new();

                $(
                    components.register_component::<$component>();
                )+


                $(
                    let init_ctx = quartz_engine::prelude::PluginInitCtx {
                        render_resource,
                    };

                    plugins.register_plugin::<$plugin>(init_ctx);
                )+

                (components, plugins)
            }
        }
    };
}
