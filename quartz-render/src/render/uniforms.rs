use crate::render::*;
use bytemuck::*;
use std::sync::Arc;
use wgpu::util::DeviceExt;

pub trait UniformClone {
    fn box_clone(&self) -> Box<dyn Uniform>;
}

impl<T: Clone + Uniform> UniformClone for T {
    fn box_clone(&self) -> Box<dyn Uniform> {
        Box::new(self.clone())
    }
}

pub trait Uniform: 'static + UniformClone {
    fn size(&self) -> wgpu::BufferAddress;

    fn data(&self) -> &[u8];
}

impl Clone for Box<dyn Uniform> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

impl<T> Uniform for T
where
    T: Pod,
{
    fn size(&self) -> wgpu::BufferAddress {
        std::mem::size_of::<Self>() as u64
    }

    fn data(&self) -> &[u8] {
        bytes_of(self)
    }
}

#[derive(Clone)]
pub struct UniformBuffer {
    pub uniform: Box<dyn Uniform>,
    pub updated: bool,
    pub(crate) buffer: Option<Arc<wgpu::Buffer>>,
}

impl UniformBuffer {
    pub fn new<T: Uniform>(uniform: T) -> Self {
        let uniform = Box::new(uniform);

        Self {
            uniform,
            updated: false,
            buffer: None,
        }
    }

    pub fn set_uniform<T: Uniform>(&mut self, uniform: T) {
        if uniform.size() == self.uniform.size() {
            self.uniform = Box::new(uniform);
            self.updated = true;
        } else {
            self.uniform = Box::new(uniform);
            self.buffer = None;
        }
    }

    pub fn create_buffer(&mut self, render_resource: &RenderResource) {
        let buffer = render_resource
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: self.uniform.data(),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            });

        self.buffer = Some(Arc::new(buffer));
    }
}

impl Binding for UniformBuffer {
    fn prepare_resource(&mut self, render_resource: &RenderResource) {
        if self.buffer.is_none() {
            self.create_buffer(render_resource);
        }

        if self.updated {
            render_resource.queue.write_buffer(
                self.buffer.as_ref().unwrap(),
                0,
                self.uniform.data(),
            );

            self.updated = false;
        }
    }

    fn binding_resource(&self) -> wgpu::BindingResource {
        self.buffer.as_ref().unwrap().as_entire_binding()
    }

    fn binding_clone(&self) -> Box<dyn Binding> {
        Box::new(Self {
            uniform: self.uniform.clone(),
            updated: false,
            buffer: None,
        })
    }
}
