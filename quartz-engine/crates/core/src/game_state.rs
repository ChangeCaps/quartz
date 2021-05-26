use crate::component::*;
use crate::plugin::*;
use crate::render::prelude::*;
use crate::tree::*;
use serde::Serialize;

pub struct GameState {
    pub tree: Tree,
    pub plugins: Plugins,
    pub components: Components,
    pub depth_texture: Texture2d<format::Depth32Float>,
}

impl GameState {
    pub fn new(
        tree: Tree,
        plugins: Plugins,
        components: Components,
        render_resource: &RenderResource,
    ) -> Self {
        let depth_texture = Texture::new(
            &TextureDescriptor::default_settings(D2::new(
                render_resource.target_width(),
                render_resource.target_height(),
            )),
            render_resource,
        );

        Self {
            tree,
            plugins,
            components,
            depth_texture,
        }
    }

    pub fn start(&mut self, render_resource: &RenderResource) {
        let plugin_ctx = PluginCtx {
            tree: &mut self.tree,
            plugins: &self.plugins,
            render_resource,
        };

        self.plugins.start(plugin_ctx);
    }

    pub fn editor_start(&mut self, render_resource: &RenderResource) {
        let plugin_ctx = PluginCtx {
            tree: &mut self.tree,
            plugins: &self.plugins,
            render_resource,
        };

        self.plugins.editor_start(plugin_ctx);
    }

    pub fn update(&mut self, render_resource: &RenderResource) {
        self.tree.update_transforms();

        let plugin_ctx = PluginCtx {
            tree: &mut self.tree,
            plugins: &self.plugins,
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
            plugins: &self.plugins,
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

    pub fn render(&mut self, render_ctx: &mut RenderCtx, render_resource: &RenderResource) {
        if self.depth_texture.dimensions.width != render_resource.target_width()
            || self.depth_texture.dimensions.height != render_resource.target_height()
        {
            self.depth_texture = Texture::new(
                &TextureDescriptor::default_settings(D2::new(
                    render_resource.target_width(),
                    render_resource.target_height(),
                )),
                render_resource,
            );
        }

        let plugin_ctx = PluginRenderCtx {
            tree: &mut self.tree,
            plugins: &self.plugins,
            render_resource: render_resource,
            render_ctx,
        };

        self.plugins.render(plugin_ctx);

        let desc = RenderPassDescriptor {
            depth_attachment: Some(DepthAttachment::default_settings(self.depth_texture.view())),
            ..Default::default()
        };
        let mut render_pass = render_ctx.render_pass_empty(&desc);

        self.tree
            .render(&self.plugins, &None, render_resource, &mut render_pass);
    }

    pub fn viewport_render(
        &mut self,
        camera: &Option<Mat4>,
        render_ctx: &mut RenderCtx,
        render_resource: &RenderResource,
    ) {
        if self.depth_texture.dimensions.width != render_resource.target_width()
            || self.depth_texture.dimensions.height != render_resource.target_height()
        {
            self.depth_texture = Texture::new(
                &TextureDescriptor::default_settings(D2::new(
                    render_resource.target_width(),
                    render_resource.target_height(),
                )),
                render_resource,
            );
        }

        let plugin_ctx = PluginRenderCtx {
            tree: &mut self.tree,
            plugins: &self.plugins,
            render_resource: render_resource,
            render_ctx,
        };

        self.plugins.viewport_render(plugin_ctx);

        let desc = RenderPassDescriptor {
            depth_attachment: Some(DepthAttachment::default_settings(self.depth_texture.view())),
            ..Default::default()
        };
        let mut render_pass = render_ctx.render_pass_empty(&desc);

        self.tree
            .viewport_render(&self.plugins, camera, render_resource, &mut render_pass);
    }

    pub fn viewport_pick_render(
        &mut self,
        camera: &Mat4,
        pipeline: &RenderPipeline,
        texture: &Texture2d<format::Depth32Float>,
        render_ctx: &mut RenderCtx,
        render_resource: &RenderResource,
    ) {
        let desc = RenderPassDescriptor {
            color_attachments: vec![],
            depth_attachment: Some(DepthAttachment::default_settings(texture.view())),
            ..Default::default()
        };

        let mut render_pass = render_ctx.render_pass(&desc, pipeline);

        self.tree.viewport_pick_render(
            &self.plugins,
            camera,
            pipeline,
            render_resource,
            &mut render_pass,
        );
    }

    pub fn serialize_tree<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.tree.serialize(serializer)
    }
}
