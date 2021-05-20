use crate::color::*;
use crate::render::*;
use bytemuck::*;
use glam::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wgpu::util::DeviceExt;

pub trait VertexAttribute: Pod {
    fn format() -> wgpu::VertexFormat;

    fn size() -> wgpu::BufferAddress {
        std::mem::size_of::<Self>() as wgpu::BufferAddress
    }

    fn to_bytes(&self) -> &[u8] {
        bytes_of(self)
    }

    fn from_bytes(bytes: &mut [u8]) -> &Self {
        from_bytes(bytes)
    }

    fn from_bytes_mut(bytes: &mut [u8]) -> &mut Self {
        from_bytes_mut(bytes)
    }
}

#[derive(Clone)]
pub struct VertexAttributeData {
    pub name: &'static str,
    pub format: wgpu::VertexFormat,
    pub data: Vec<u8>,
}

impl VertexAttributeData {
    pub fn new<V: VertexAttribute>(name: &'static str, data: Vec<V>) -> Self {
        Self {
            name,
            format: V::format(),
            data: cast_slice(&data).to_vec(),
        }
    }
}

pub struct Mesh {
    pub vertex_data: HashMap<String, VertexAttributeData>,
    pub indices: Vec<u32>,
    pub(crate) index_buffer: Mutex<Option<Arc<wgpu::Buffer>>>,
    pub(crate) vertex_buffers: Mutex<HashMap<String, Arc<wgpu::Buffer>>>,
}

impl Clone for Mesh {
    fn clone(&self) -> Self {
        Self {
            vertex_data: self.vertex_data.clone(),
            indices: self.indices.clone(),
            index_buffer: Mutex::new(None),
            vertex_buffers: Mutex::new(HashMap::new()),
        }
    }
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            vertex_data: HashMap::new(),
            indices: Vec::new(),
            index_buffer: Mutex::new(None),
            vertex_buffers: Mutex::new(HashMap::new()),
        }
    }

    pub fn set_indices(&mut self, indices: Vec<u32>) {
        self.indices = indices;
        *self.index_buffer.lock().unwrap() = None;
    }

    pub fn set_attribute<V: VertexAttribute>(&mut self, name: &'static str, data: Vec<V>) {
        let attr = VertexAttributeData::new(name, data);

        self.vertex_data.insert(name.into(), attr);
        self.vertex_buffers.lock().unwrap().remove(name);
    }

    pub fn get_attribute<V: VertexAttribute>(&self, name: &'static str) -> Option<&[V]> {
        if let Some(data) = self.vertex_data.get(name) {
            if V::format() == data.format {
                Some(cast_slice(&data.data))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_attribute_mut<V: VertexAttribute>(
        &mut self,
        name: &'static str,
    ) -> Option<&mut [V]> {
        if let Some(data) = self.vertex_data.get_mut(name) {
            if V::format() == data.format {
                Some(cast_slice_mut(&mut data.data))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn create_vertex_buffer(&self, name: &String, render_resource: &RenderResource) {
        if let Some(data) = self.vertex_data.get(name) {
            let buffer =
                render_resource
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: cast_slice(&data.data),
                        usage: wgpu::BufferUsage::VERTEX,
                    });

            self.vertex_buffers
                .lock()
                .unwrap()
                .insert(name.clone(), Arc::new(buffer));
        }
    }

    pub fn create_vertex_buffers(
        &self,
        pipeline: &PipelineLayout,
        render_resource: &RenderResource,
    ) {
        for (name, _attribute) in &pipeline.vertex_attributes {
            if !self.has_vertex_buffer(name) {
                self.create_vertex_buffer(name, render_resource);
            }
        }
    }

    pub fn has_vertex_buffer(&self, name: &String) -> bool {
        self.vertex_buffers.lock().unwrap().contains_key(name)
    }

    pub fn get_vertex_buffer(&self, name: &String) -> Option<Arc<wgpu::Buffer>> {
        self.vertex_buffers.lock().unwrap().get(name).cloned()
    }

    pub fn create_index_buffer(&self, render_resource: &RenderResource) {
        let index_buffer =
            render_resource
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: cast_slice(&self.indices),
                    usage: wgpu::BufferUsage::INDEX,
                });

        *self.index_buffer.lock().unwrap() = Some(Arc::new(index_buffer));
    }
}

impl VertexAttribute for f32 {
    fn format() -> wgpu::VertexFormat {
        wgpu::VertexFormat::Float32
    }
}

impl VertexAttribute for Vec2 {
    fn format() -> wgpu::VertexFormat {
        wgpu::VertexFormat::Float32x2
    }
}

impl VertexAttribute for Vec3 {
    fn format() -> wgpu::VertexFormat {
        wgpu::VertexFormat::Float32x3
    }
}

impl VertexAttribute for Vec4 {
    fn format() -> wgpu::VertexFormat {
        wgpu::VertexFormat::Float32x4
    }
}

impl VertexAttribute for Color {
    fn format() -> wgpu::VertexFormat {
        wgpu::VertexFormat::Float32x4
    }
}
