use crate::render::prelude::*;
use crate::state::*;
use crate::tree::*;

pub struct GameState {
    pub tree: Tree,
    pub state: Box<dyn State>,
}

impl GameState {
    pub fn new(state: impl State, _render_pipeline: &RenderPipeline) -> Self {
        Self {
            tree: Tree::new(),
            state: Box::new(state),
        }
    }

    pub fn init(&mut self, render_resource: &RenderResource) {
        let ctx = InitCtx {
            tree: &mut self.tree,
            render_resource: render_resource,
        };

        self.state.init(ctx);
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self, render_resource: &RenderResource) {
        render_resource
            .render(|render_ctx| {
                let desc = Default::default();
                let mut render_pass = render_ctx.render_pass_empty(&desc);

                self.tree.render(render_resource, &mut render_pass);
            })
            .unwrap();
    }
}
