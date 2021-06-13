use crate::prelude::*;
use std::sync::{Arc, Mutex};
use wgpu::BIND_BUFFER_ALIGNMENT;
pub use wgpu::{LoadOp, Operations};

pub struct ColorAttachment<'a, F: TextureFormat> {
    pub texture: TextureView<'a, F>,
    pub resolve_target: Option<TextureView<'a, F>>,
    pub ops: Operations<wgpu::Color>,
}

pub struct DepthAttachment<'a, F: TextureFormat> {
    pub texture: TextureView<'a, F>,
    pub depth_ops: Option<Operations<f32>>,
    pub stencil_ops: Option<Operations<u32>>,
}

impl<'a, F: TextureFormat> DepthAttachment<'a, F> {
    pub fn default_settings(texture: TextureView<'a, F>) -> Self {
        Self {
            texture,
            depth_ops: Some(Operations {
                load: LoadOp::Clear(1.0),
                store: true,
            }),
            stencil_ops: None,
        }
    }
}

pub struct RenderPassDescriptor<
    'a,
    C: TextureFormat = format::TargetFormat,
    D: TextureFormat = format::Depth32Float,
> {
    pub label: Option<String>,
    pub color_attachments: Vec<ColorAttachment<'a, C>>,
    pub depth_attachment: Option<DepthAttachment<'a, D>>,
}

impl<'a, C: TextureFormat, D: TextureFormat> RenderPassDescriptor<'a, C, D> {
    pub fn default_settings(texture: TextureView<'a, C>) -> Self {
        Self {
            label: Some("Render Pass".into()),
            color_attachments: vec![ColorAttachment::<'a, C> {
                texture,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_attachment: None,
        }
    }
}

#[derive(Debug)]
pub(crate) enum Command {
    SetPipeline {
        pipeline: Arc<wgpu::RenderPipeline>,
    },
    SetBindGroup {
        set: u32,
        bind_group: Arc<wgpu::BindGroup>,
    },
    Draw {
        vertices: std::ops::Range<u32>,
        instances: std::ops::Range<u32>,
    },
    DrawIndexed {
        indices: std::ops::Range<u32>,
        base_vertex: i32,
        instances: std::ops::Range<u32>,
    },
    SetVertexBuffer {
        buffer: Arc<wgpu::Buffer>,
        slot: u32,
    },
    SetIndexBuffer {
        buffer: Arc<wgpu::Buffer>,
        format: wgpu::IndexFormat,
    },
}

pub struct RenderPass<'a, 'b, 'c, C: TextureFormat, D: TextureFormat> {
    pub(crate) commands: Vec<Command>,
    pub(crate) pipeline: &'a RenderPipeline<C, D>,
    pub(crate) descriptor: &'a RenderPassDescriptor<'c, C, D>,
    pub(crate) ctx: &'a mut RenderCtx<'b>,
}

impl<'a, 'b, 'c, C: TextureFormat, D: TextureFormat> RenderPass<'a, 'b, 'c, C, D> {
    pub fn set_pipeline(&mut self, pipeline: &'a RenderPipeline<C, D>) -> &mut Self {
        self.commands.push(Command::SetPipeline {
            pipeline: pipeline.pipeline.clone(),
        });

        self.pipeline = pipeline;

        self
    }

    pub fn set_bindings(&mut self, bindings: &mut Bindings) -> &mut Self {
        bindings.generate_groups(self.ctx.instance);

        for (set, bind_group) in &bindings.bind_groups {
            self.commands.push(Command::SetBindGroup {
                set: (*set) as u32,
                bind_group: bind_group.clone(),
            });
        }

        self
    }

    pub fn set_bind_groups(&mut self, bind_groups: &Vec<Arc<wgpu::BindGroup>>) -> &mut Self {
        for (set, bind_group) in bind_groups.iter().enumerate() {
            self.commands.push(Command::SetBindGroup {
                set: set as u32,
                bind_group: bind_group.clone(),
            });
        }

        self
    }

    pub fn draw(&mut self, vertices: std::ops::Range<u32>) -> &mut Self {
        self.commands.push(Command::Draw {
            vertices,
            instances: 0..1,
        });

        self
    }

    pub fn draw_mesh(&mut self, mesh: &Mesh) -> &mut Self {
        if mesh.index_buffer.lock().unwrap().is_none() {
            mesh.create_index_buffer(self.ctx.instance);
        }

        mesh.create_vertex_buffers(&self.pipeline.layout, self.ctx.instance);

        for (name, attribute) in &self.pipeline.layout.vertex_attributes {
            let data = mesh.vertex_data.get(name).unwrap();

            if data.format == attribute.format {
                let buffer = mesh.get_vertex_buffer(name).unwrap();

                self.commands.push(Command::SetVertexBuffer {
                    slot: attribute.shader_location,
                    buffer: buffer,
                });
            }
        }

        self.commands.push(Command::SetIndexBuffer {
            buffer: mesh.index_buffer.lock().unwrap().clone().unwrap(),
            format: wgpu::IndexFormat::Uint32,
        });
        self.commands.push(Command::DrawIndexed {
            indices: 0..mesh.indices.len() as u32,
            base_vertex: 0,
            instances: 0..1,
        });

        self
    }
}

pub(crate) fn execute_commands<C: TextureFormat, D: TextureFormat>(
    descriptor: &RenderPassDescriptor<C, D>,
    commands: &Vec<Command>,
    ctx: &mut RenderCtx,
) {
    let color_attachments = descriptor
        .color_attachments
        .iter()
        .map(|attachment| {
            if let Some(download) = &attachment.texture.download {
                download.store(true, std::sync::atomic::Ordering::SeqCst);
            }

            wgpu::RenderPassColorAttachment {
                view: &attachment.texture.view(),
                resolve_target: attachment.resolve_target.as_ref().map(|t| &*t.view()),
                ops: attachment.ops.clone(),
            }
        })
        .collect::<Vec<_>>();

    let label = match &descriptor.label {
        Some(l) => Some(l.as_str()),
        None => None,
    };

    let descriptor = wgpu::RenderPassDescriptor {
        label,
        color_attachments: &color_attachments,
        depth_stencil_attachment: descriptor
            .depth_attachment
            .as_ref()
            .map(|depth_attachment| {
                if let Some(download) = &depth_attachment.texture.download {
                    download.store(true, std::sync::atomic::Ordering::SeqCst);
                }

                wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_attachment.texture.view(),
                    depth_ops: depth_attachment.depth_ops.clone(),
                    stencil_ops: depth_attachment.stencil_ops.clone(),
                }
            }),
    };

    let mut render_pass = ctx.encoder.as_mut().unwrap().begin_render_pass(&descriptor);

    for command in commands {
        match command {
            Command::SetPipeline { pipeline } => {
                render_pass.set_pipeline(&pipeline);
            }
            Command::SetBindGroup { set, bind_group } => {
                render_pass.set_bind_group(*set, &bind_group, &[]);
            }
            Command::Draw {
                vertices,
                instances,
            } => {
                render_pass.draw(vertices.clone(), instances.clone());
            }
            Command::DrawIndexed {
                indices,
                base_vertex,
                instances,
            } => {
                render_pass.draw_indexed(indices.clone(), *base_vertex, instances.clone());
            }
            Command::SetVertexBuffer { buffer, slot } => {
                render_pass.set_vertex_buffer(*slot, buffer.slice(..));
            }
            Command::SetIndexBuffer { buffer, format } => {
                render_pass.set_index_buffer(buffer.slice(..), *format);
            }
        }
    }
}

impl<C: TextureFormat, D: TextureFormat> Drop for RenderPass<'_, '_, '_, C, D> {
    fn drop(&mut self) {
        execute_commands(self.descriptor, &self.commands, self.ctx);
    }
}

pub struct EmptyRenderPass<'a, 'b, 'c, C: TextureFormat, D: TextureFormat> {
    pub(crate) commands: Vec<Command>,
    pub(crate) descriptor: &'a RenderPassDescriptor<'c, C, D>,
    pub(crate) ctx: &'a mut RenderCtx<'b>,
}

impl<'a, 'v, 'c, C: TextureFormat, D: TextureFormat> EmptyRenderPass<'a, 'v, 'c, C, D> {
    pub fn with_pipeline<'rp>(
        &'rp mut self,
        pipeline: &'rp RenderPipeline<C, D>,
    ) -> PipelineRenderPass<'a, 'rp, 'v, 'c, C, D> {
        let mut pass = PipelineRenderPass {
            pipeline,
            pass: self,
        };

        pass.set_pipeline(pipeline);

        pass
    }
}

pub struct PipelineRenderPass<'erp, 'rp, 'v, 'c, C: TextureFormat, D: TextureFormat> {
    pub(crate) pipeline: &'rp RenderPipeline<C, D>,
    pub(crate) pass: &'rp mut EmptyRenderPass<'erp, 'v, 'c, C, D>,
}

impl<'rp, C: TextureFormat, D: TextureFormat> PipelineRenderPass<'_, 'rp, '_, '_, C, D> {
    pub fn set_pipeline(&mut self, pipeline: &'rp RenderPipeline<C, D>) -> &mut Self {
        self.pass.commands.push(Command::SetPipeline {
            pipeline: pipeline.pipeline.clone(),
        });

        self.pipeline = pipeline;

        self
    }

    pub fn set_bindings(&mut self, bindings: &mut Bindings) -> &mut Self {
        bindings.generate_groups(self.pass.ctx.instance);

        for (set, bind_group) in &bindings.bind_groups {
            self.pass.commands.push(Command::SetBindGroup {
                set: (*set) as u32,
                bind_group: bind_group.clone(),
            });
        }

        self
    }

    pub fn set_bind_groups(&mut self, bind_groups: &Vec<Arc<wgpu::BindGroup>>) -> &mut Self {
        for (set, bind_group) in bind_groups.iter().enumerate() {
            self.pass.commands.push(Command::SetBindGroup {
                set: set as u32,
                bind_group: bind_group.clone(),
            });
        }

        self
    }

    pub fn draw(&mut self, vertices: std::ops::Range<u32>) -> &mut Self {
        self.pass.commands.push(Command::Draw {
            vertices,
            instances: 0..1,
        });

        self
    }

    pub fn draw_mesh(&mut self, mesh: &Mesh) -> &mut Self {
        if mesh.index_buffer.lock().unwrap().is_none() {
            mesh.create_index_buffer(self.pass.ctx.instance);
        }

        mesh.create_vertex_buffers(&self.pipeline.layout, self.pass.ctx.instance);

        for (name, attribute) in &self.pipeline.layout.vertex_attributes {
            let data = mesh.vertex_data.get(name).unwrap();

            if data.format == attribute.format {
                let buffer = mesh.get_vertex_buffer(name).unwrap();

                self.pass.commands.push(Command::SetVertexBuffer {
                    slot: attribute.shader_location,
                    buffer: buffer,
                });
            }
        }

        self.pass.commands.push(Command::SetIndexBuffer {
            buffer: mesh.index_buffer.lock().unwrap().clone().unwrap(),
            format: wgpu::IndexFormat::Uint32,
        });
        self.pass.commands.push(Command::DrawIndexed {
            indices: 0..mesh.indices.len() as u32,
            base_vertex: 0,
            instances: 0..1,
        });
 
        self
    }
}

impl<C: TextureFormat, D: TextureFormat> Drop for EmptyRenderPass<'_, '_, '_, C, D> {
    fn drop(&mut self) {
        execute_commands(self.descriptor, &self.commands, self.ctx);
    }
}
