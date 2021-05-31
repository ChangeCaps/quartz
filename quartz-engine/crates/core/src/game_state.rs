use crate::component::*;
use crate::plugin::*;
use crate::render::prelude::*;
use crate::render::wgpu;
use crate::tree::*;
use crate::scene::*;
use serde::Serialize;

pub struct GameState {
    pub tree: Tree,
    pub plugins: Box<Plugins>,
    pub components: Box<Components>,
    pub depth_texture: Texture2d<format::Depth32Float>,
}

impl GameState {
    pub fn new(
        tree: Tree,
        plugins: Box<Plugins>,
        components: Box<Components>,
        instance: &Instance,
    ) -> Self {
        let depth_texture = Texture::new(
            &TextureDescriptor::default_settings(D2::new(1, 1)),
            instance,
        );

        Self {
            tree,
            plugins,
            components,
            depth_texture,
        }
    }

    pub fn start(&mut self, target_format: format::TargetFormat, instance: &Instance) {
        let plugin_ctx = PluginCtx {
            tree: &mut self.tree,
            plugins: &self.plugins,
            instance,
            target_format,
        };

        self.plugins.start(plugin_ctx);
        self.tree.start(&self.plugins, instance);
    }

    pub fn editor_start(&mut self, target_format: format::TargetFormat, instance: &Instance) {
        let plugin_ctx = PluginCtx {
            tree: &mut self.tree,
            plugins: &self.plugins,
            instance,
            target_format,
        };

        self.plugins.editor_start(plugin_ctx);
        self.tree.editor_start(&self.plugins, instance);
    }

    pub fn update(&mut self, target_format: format::TargetFormat, instance: &Instance) {
        self.tree.update_transforms();

        let plugin_ctx = PluginCtx {
            tree: &mut self.tree,
            plugins: &self.plugins,
            instance,
            target_format,
        };

        self.plugins.update(plugin_ctx);

        self.tree.update(&self.plugins, instance);

        let nodes = std::mem::replace(&mut self.tree.despawn, Vec::new());

        for node_id in &nodes {
            self.tree
                .despawn_recursive(node_id, &self.plugins, instance);
        }

        for node_id in nodes {
            self.tree.remove_recursive(node_id);
        }
    }

    pub fn editor_update(&mut self, target_format: format::TargetFormat, instance: &Instance) {
        self.tree.update_transforms();

        let plugin_ctx = PluginCtx {
            tree: &mut self.tree,
            plugins: &self.plugins,
            instance,
            target_format,
        };

        self.plugins.editor_update(plugin_ctx);

        self.tree.editor_update(&self.plugins, instance);

        let nodes = std::mem::replace(&mut self.tree.despawn, Vec::new());

        for node_id in &nodes {
            self.tree
                .despawn_recursive(node_id, &self.plugins, instance);
        }

        for node_id in nodes {
            self.tree.remove_recursive(node_id);
        }
    }

    pub fn resize_depth_texture(&mut self, width: u32, height: u32, instance: &Instance) {
        if self.depth_texture.dimensions.width != width
            || self.depth_texture.dimensions.height != height
        {
            self.depth_texture = Texture::new(
                &TextureDescriptor::default_settings(D2::new(width, height)),
                instance,
            );
        }
    }

    pub fn render(
        &mut self,
        target: TextureView<format::TargetFormat>,
        render_ctx: &mut RenderCtx,
        instance: &Instance,
    ) {
        self.resize_depth_texture(target.width(), target.height(), instance);

        let plugin_ctx = PluginRenderCtx {
            tree: &mut self.tree,
            plugins: &self.plugins,
            instance: instance,
            render_ctx,
            target: &target,
        };

        self.plugins.render(plugin_ctx);

        let desc = RenderPassDescriptor {
            depth_attachment: Some(DepthAttachment::default_settings(self.depth_texture.view())),
            ..RenderPassDescriptor::default_settings(target)
        };
        let mut render_pass = render_ctx.render_pass_empty(&desc);

        self.tree
            .render(&self.plugins, &None, instance, &mut render_pass);
    }

    pub fn viewport_render(
        &mut self,
        camera: &Option<Mat4>,
        target: TextureView<format::TargetFormat>,
        render_ctx: &mut RenderCtx,
        instance: &Instance,
    ) {
        self.resize_depth_texture(target.width(), target.height(), instance);

        let plugin_ctx = PluginRenderCtx {
            tree: &mut self.tree,
            plugins: &self.plugins,
            instance: instance,
            render_ctx,
            target: &target,
        };

        self.plugins.viewport_render(plugin_ctx);

        let desc = RenderPassDescriptor {
            depth_attachment: Some(DepthAttachment::default_settings(self.depth_texture.view())),
            ..RenderPassDescriptor::default_settings(target)
        };
        let mut render_pass = render_ctx.render_pass_empty(&desc);

        self.tree
            .viewport_render(&self.plugins, camera, instance, &mut render_pass);
    }

    pub fn viewport_pick_render(
        &mut self,
        camera: &Mat4,
        pipeline: &RenderPipeline<format::R32Uint>,
        texture: &Texture2d<format::R32Uint>,
        depth_texture: &Texture2d<format::Depth32Float>,
        render_ctx: &mut RenderCtx,
        instance: &Instance,
    ) {
        let desc = RenderPassDescriptor {
            label: Some("Viewport pick pass".to_string()),
            color_attachments: vec![ColorAttachment {
                texture: texture.view(),
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(wgpu::Color {
                        r: std::u32::MAX as f64,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: true,
                },
            }],
            depth_attachment: Some(DepthAttachment::default_settings(depth_texture.view())),
        };

        let mut render_pass = render_ctx.render_pass(&desc, pipeline);

        self.tree
            .viewport_pick_render(&self.plugins, camera, pipeline, instance, &mut render_pass);
    }

    pub fn serialize_scene<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Scene {
            tree: &self.tree,
            plugins: &self.plugins,
        }.serialize(serializer)
    }
}
