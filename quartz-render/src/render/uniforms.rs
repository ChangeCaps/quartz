use crate::render::*;
use glam::*;
use bytemuck::*;
use std::sync::Arc;
use wgpu::util::DeviceExt;

pub trait Uniform {
    fn alignment() -> wgpu::BufferAddress;

    fn size() -> wgpu::BufferAddress;

    fn data(&self) -> Vec<u8>;
}

fn append_aligned<T: Uniform>(data: &mut Vec<u8>, uniform: &T, alignment: wgpu::BufferAddress) {
    data.append(&mut uniform.data());

    let remaining_bytes = ((data.len() - 1) / alignment as usize + 1) * alignment as usize;

    data.append(&mut vec![0; remaining_bytes]);
}

pub struct UniformBuffer<T: Uniform, const L: u32> {
    uniforms: Vec<T>,
}

impl<T: Uniform, const L: u32> std::ops::Index<u32> for UniformBuffer<T, L> {
    type Output = T;

    fn index(&self, index: u32) -> &T {
        &self.uniforms[index as usize]
    }
}

impl<T: Uniform, const L: u32> std::ops::IndexMut<u32> for UniformBuffer<T, L> {
    fn index_mut(&mut self, index: u32) -> &mut T {
        &mut self.uniforms[index as usize]
    }
}

impl<T: Uniform, const L: u32> IntoIterator for UniformBuffer<T, L> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.uniforms.into_iter()
    }
}

impl<T: Uniform, const L: u32> UniformBuffer<T, L> {
    pub fn new() -> Self {
        Self {
            uniforms: Vec::with_capacity(L as usize),
        }
    }

    pub fn push(&mut self, uniform: T) -> Result<(), ()> {
        if self.uniforms.len() < L as usize {
            self.uniforms.push(uniform);

            Ok(())
        } else {
            Err(())
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        self.uniforms.pop()
    }

    pub fn remove(&mut self, index: u32) -> T {
        self.uniforms.remove(index as usize)
    }

    pub fn len(&self) -> u32 {
        self.uniforms.len() as u32
    }

    pub fn clear(&mut self) {
        self.uniforms.drain(..);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.uniforms.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.uniforms.iter_mut()
    }
}

impl<T: Uniform, const L: u32> Uniform for UniformBuffer<T, L> {
    fn alignment() -> u64 {
        16
    }

    fn size() -> u64 {
        ((T::size() * L as u64 - 1) / 16 + 1) * 16 + 16
    }

    fn data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::size() as usize);

        let len = data.len() as u32;
        append_aligned(&mut data, &len, 16);

        for uniform in &self.uniforms {
            append_aligned(&mut data, uniform, 16);
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
    pub fn new<T: Uniform>(uniform: &T) -> Self {
        Self {
            data: uniform.data(),
            updated: false,
            buffer: None,
        }
    }

    pub fn set_uniform<T: Uniform>(&mut self, uniform: &T) {
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

impl Uniform for f32 {
    fn alignment() -> wgpu::BufferAddress {
        4
    }

    fn size() -> wgpu::BufferAddress {
        4
    }

    fn data(&self) -> Vec<u8> {
        bytes_of(self).to_vec()
    }
}

impl Uniform for u32 {
    fn alignment() -> wgpu::BufferAddress {
        4
    }

    fn size() -> wgpu::BufferAddress {
        4
    }

    fn data(&self) -> Vec<u8> {
        bytes_of(self).to_vec()
    }
}

impl Uniform for i32 {
    fn alignment() -> wgpu::BufferAddress {
        4
    }

    fn size() -> wgpu::BufferAddress {
        4
    }

    fn data(&self) -> Vec<u8> {
        bytes_of(self).to_vec()
    }
}

impl Uniform for Vec2 {
    fn alignment() -> wgpu::BufferAddress {
        8
    }

    fn size() -> wgpu::BufferAddress {
        8
    }

    fn data(&self) -> Vec<u8> {
        bytes_of(self).to_vec()
    }
}

impl Uniform for Vec3 {
    fn alignment() -> wgpu::BufferAddress {
        16
    }

    fn size() -> wgpu::BufferAddress {
        12
    }

    fn data(&self) -> Vec<u8> {
        bytes_of(self).to_vec()
    }
}

impl Uniform for crate::color::Color {
    fn alignment() -> wgpu::BufferAddress {
        16
    }

    fn size() -> wgpu::BufferAddress {
        16
    }

    fn data(&self) -> Vec<u8> {
        bytes_of(self).to_vec()
    }
}

impl Uniform for Vec4 {
    fn alignment() -> wgpu::BufferAddress {
        16
    }

    fn size() -> wgpu::BufferAddress {
        16
    }

    fn data(&self) -> Vec<u8> {
        bytes_of(self).to_vec()
    }
}

impl Uniform for Mat4 {
    fn alignment() -> wgpu::BufferAddress {
        16
    }

    fn size() -> wgpu::BufferAddress {
        64
    }

    fn data(&self) -> Vec<u8> {
        bytes_of(self).to_vec()
    }
}