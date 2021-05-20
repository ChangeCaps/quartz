use crate::render::*;

pub struct RenderCtx<'a> {
    pub render_target: &'a wgpu::TextureView,
    pub(crate) render_resource: &'a RenderResource,
    pub(crate) encoder: wgpu::CommandEncoder,
}

impl<'a> RenderCtx<'a> {
    pub fn render_pass_empty<'b, C: TextureFormat, D: TextureFormat>(
        &'b mut self,
        descriptor: &'b RenderPassDescriptor<C, D>,
    ) -> EmptyRenderPass<'b, 'a, C, D> {
        let pass = EmptyRenderPass {
            commands: Vec::new(),
            descriptor,
            ctx: self,
        };

        pass
    }

    pub fn render_pass<'b, C: TextureFormat, D: TextureFormat>(
        &'b mut self,
        descriptor: &'b RenderPassDescriptor<C, D>,
        pipeline: &'b RenderPipeline<C, D>,
    ) -> RenderPass<'b, 'a, C, D> {
        let mut pass = RenderPass {
            commands: Vec::new(),
            pipeline: pipeline,
            descriptor,
            ctx: self,
        };

        pass.set_pipeline(pipeline);

        pass
    }
}
