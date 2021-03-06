use crate::prelude::*;

pub struct RenderCtx<'a> {
    pub(crate) instance: &'a Instance,
    pub(crate) encoder: Option<wgpu::CommandEncoder>,
}

impl<'a> RenderCtx<'a> {
    pub fn render_pass_empty<
        'b,
        'c,
        C: ToColorAttachment<'c> + ColorTargetState,
        D: ToDepthAttachment<'c> + DepthStencilState,
    >(
        &'b mut self,
        descriptor: &'b RenderPassDescriptor<'c, C, D>,
    ) -> EmptyRenderPass<'b, 'a, 'c, C, D> {
        let pass = EmptyRenderPass {
            commands: Vec::new(),
            descriptor,
            ctx: self,
        };

        pass
    }

    pub fn render_pass<
        'b,
        'c,
        C: ToColorAttachment<'c> + ColorTargetState,
        D: ToDepthAttachment<'c> + DepthStencilState,
    >(
        &'b mut self,
        descriptor: &'b RenderPassDescriptor<'c, C, D>,
        pipeline: &'b RenderPipeline<C, D>,
    ) -> RenderPass<'b, 'a, 'c, C, D> {
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

impl<'a> Drop for RenderCtx<'a> {
    fn drop(&mut self) {
        self.instance
            .queue
            .submit(std::iter::once(self.encoder.take().unwrap().finish()));
    }
}
