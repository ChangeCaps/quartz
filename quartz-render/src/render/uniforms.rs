use crate::render::*;
use bytemuck::*;
use std::sync::Arc;
use wgpu::util::DeviceExt;

pub trait Uniform {
    fn size() -> wgpu::BufferAddress;

    fn data(&self) -> Vec<u8>;
}

impl<T> Uniform for T
where
    T: Pod,
{
    fn size() -> wgpu::BufferAddress {
        std::mem::size_of::<Self>() as u64
    }

    fn data(&self) -> Vec<u8> {
        bytes_of(self).to_vec()
    }
}

pub struct UniformBuffer<T: Uniform, const L: u64> {
    uniforms: Vec<T>,
}

impl<T: Uniform, const L: u64> Uniform for UniformBuffer<T, L> {
    fn size() -> u64 {
        T::size() * L + 4
    }

    fn data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::size() as usize);

        data.append(&mut bytes_of(&(L as u32)).to_vec());

        for uniform in &self.uniforms {
            data.append(&mut uniform.data());
        }

        let remaining_bytes = Self::size() as usize - data.len();

        data.append(&mut vec![0; remaining_bytes]);

        data
    }
}

#[derive(Clone)]
pub struct UniformBinding {
    pub data: Vec<u8>,
    pub updated: bool,
    pub(crate) buffer: Option<Arc<wgpu::Buffer>>,
}

impl UniformBinding {
    pub fn new<T: Uniform>(uniform: T) -> Self {
        Self {
            data: uniform.data(),
            updated: false,
            buffer: None,
        }
    }

    pub fn set_uniform<T: Uniform>(&mut self, uniform: T) {
        self.data = uniform.data();

        if T::size() == self.data.len() as u64 {
            self.updated = true;
        } else {
            self.buffer = None;
        }
    }

    pub fn create_buffer(&mut self, render_resource: &RenderResource) {
        let buffer = render_resource
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: &self.data,
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            });

        self.buffer = Some(Arc::new(buffer));
    }
}

impl Binding for UniformBinding {
    fn prepare_resource(&mut self, render_resource: &RenderResource) {
        if self.buffer.is_none() {
            self.create_buffer(render_resource);
        }

        if self.updated {
            render_resource.queue.write_buffer(
                self.buffer.as_ref().unwrap(),
                0,
                &self.data,
            );

            self.updated = false;
        }
    }

    fn binding_resource(&self) -> wgpu::BindingResource {
        self.buffer.as_ref().unwrap().as_entire_binding()
    }

    fn binding_clone(&self) -> Box<dyn Binding> {
        Box::new(Self {
            data: self.data.clone(),
            updated: false,
            buffer: None,
        })
    }
}
