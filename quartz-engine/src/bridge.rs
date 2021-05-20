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
    ($new:path) => {
        mod new {
            use super::*;

            #[no_mangle]
            pub fn new(
                render_resource: &quartz_engine::render::prelude::RenderResource,
            ) -> quartz_engine::game_state::GameState {
                let state = $new(render_resource);
                quartz_engine::game_state::GameState::new(state, render_resource)
            }
        }
    };
}
