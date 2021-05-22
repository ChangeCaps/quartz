use crate::component::*;
use crate::plugin::*;
use crate::render::prelude::*;
use crate::tree::*;
use serde::Serialize;

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

    pub fn start(&mut self, render_resource: &RenderResource) {
        let plugin_ctx = PluginCtx {
            tree: &mut self.tree,
            render_resource,
        };

        self.plugins.start(plugin_ctx);
    }

    pub fn editor_start(&mut self, render_resource: &RenderResource) {
        let plugin_ctx = PluginCtx {
            tree: &mut self.tree,
            render_resource,
        };

        self.plugins.editor_start(plugin_ctx);
    }

    pub fn update(&mut self, render_resource: &RenderResource) {
        self.tree.update_transforms();

        let plugin_ctx = PluginCtx {
            tree: &mut self.tree,
            render_resource,
        };

        self.plugins.update(plugin_ctx);

        self.tree.update(&self.plugins, render_resource);

        let nodes = std::mem::replace(&mut self.tree.despawn, Vec::new());

        for node_id in &nodes {
            self.tree
                .despawn_recursive(node_id, &self.plugins, render_resource);
        }

        for node_id in nodes {
            self.tree.remove_recursive(node_id);
        }
    }

    pub fn editor_update(&mut self, render_resource: &RenderResource) {
        self.tree.update_transforms();

        let plugin_ctx = PluginCtx {
            tree: &mut self.tree,
            render_resource,
        };

        self.plugins.editor_update(plugin_ctx);

        self.tree.editor_update(&self.plugins, render_resource);

        let nodes = std::mem::replace(&mut self.tree.despawn, Vec::new());

        for node_id in &nodes {
            self.tree
                .despawn_recursive(node_id, &self.plugins, render_resource);
        }

        for node_id in nodes {
            self.tree.remove_recursive(node_id);
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

    pub fn serialize_tree<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.tree.serialize(serializer)
    }
}