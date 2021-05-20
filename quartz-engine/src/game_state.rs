use crate::plugin::*;
use crate::render::prelude::*;
use crate::state::*;
use crate::tree::*;

pub struct GameState {
    pub tree: Tree,
    pub plugins: Plugins,
    pub components: Components,
}

impl GameState {
    pub fn new(
        tree: Tree,
        plugins: Plugins,
        components: Components,
        _render_resource: &RenderResource,
    ) -> Self {
        Self {
            tree,
            plugins,
            components,
        }
    }

    pub fn init(&mut self, _render_resource: &RenderResource) {}

    pub fn update(&mut self, render_resource: &RenderResource) {
        self.tree.update(&self.plugins, render_resource);

        for node in std::mem::replace(&mut self.tree.despawn, Vec::new()) {
            self.tree.remove_recursive(node);
        }
    }

    pub fn render(&mut self, render_resource: &RenderResource) {
        render_resource
            .render(|render_ctx| {
                let desc = Default::default();
                let mut render_pass = render_ctx.render_pass_empty(&desc);

                self.tree
                    .render(&self.plugins, render_resource, &mut render_pass);
            })
            .unwrap();
    }
}
