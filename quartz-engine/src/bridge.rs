use crate::game_state::*;
use libloading::*;
use quartz_render::prelude::*;

pub struct Bridge {
    lib: Library,
}

impl Bridge {
    pub unsafe fn load(path: &std::path::Path) -> Result<Self, Error> {
        let lib = Library::new(path)?;

        Ok(Self { lib })
    }

    pub fn new(&self, render_resource: &RenderResource) -> Result<GameState, Error> {
        let new: Symbol<fn(render_resource: &RenderResource) -> GameState> =
            unsafe { self.lib.get(b"new") }?;

        Ok(new(render_resource))
    }
}

#[macro_export]
macro_rules! bridge {
    {
        components: { $( $component:path ),+ $(,)? }
        plugins: { $( $plugin:path ),+ $(,)? }
    } => {
        mod new {
            use super::*;

            #[no_mangle]
            pub fn new(
                render_resource: &quartz_engine::render::prelude::RenderResource,
            ) -> quartz_engine::game_state::GameState {
                let mut components = quartz_engine::state::Components::new();
                let mut plugins = quartz_engine::plugin::Plugins::new();
                let mut tree = quartz_engine::prelude::Tree::new();

                $(
                    components.register_component::<$component>();
                )+


                $(
                    let init_ctx = quartz_engine::prelude::InitCtx {
                        tree: &mut tree,
                        render_resource,
                    };

                    plugins.register_plugin::<$plugin>(init_ctx);
                )+

                quartz_engine::game_state::GameState::new(tree, plugins, components, render_resource)
            }
        }
    };
}
