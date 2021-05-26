use crate::color::*;
use crate::prelude::*;
use bytemuck::*;
use glam::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wgpu::util::DeviceExt;

pub trait VertexAttribute: Pod + Zeroable {
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

#[derive(Clone, Serialize, Deserialize)]
pub struct VertexAttributeData {
    pub name: String,
    pub format: wgpu::VertexFormat,
    pub data: Vec<u8>,
}

impl VertexAttributeData {
    pub fn new<V: VertexAttribute>(name: String, data: Vec<V>) -> Self {
        Self {
            name,
            format: V::format(),
            data: cast_slice(&data).to_vec(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Mesh {
    pub(crate) vertex_data: HashMap<String, VertexAttributeData>,
    pub(crate) indices: Vec<u32>,
    #[serde(skip)]
    pub(crate) index_buffer: Mutex<Option<Arc<wgpu::Buffer>>>,
    #[serde(skip)]
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

    pub fn vertex_data(&self) -> &HashMap<String, VertexAttributeData> {
        &self.vertex_data
    }

    pub fn vertex_data_mut(&mut self) -> &mut HashMap<String, VertexAttributeData> {
        &mut self.vertex_data
    }

    pub fn set_indices(&mut self, indices: Vec<u32>) {
        self.indices = indices;
        *self.index_buffer.lock().unwrap() = None;
    }

    pub fn indices(&self) -> &Vec<u32> {
        &self.indices
    }

    pub fn indices_mut(&mut self) -> &mut Vec<u32> {
        *self.index_buffer.lock().unwrap() = None;
        &mut self.indices
    }

    pub fn indices_mut_unmarked(&mut self) -> &mut Vec<u32> {
        &mut self.indices
    }

    pub fn invalidate_index_buffer(&self) {
        *self.index_buffer.lock().unwrap() = None;
    }

    pub fn add_attribute<V: VertexAttribute>(&mut self, name: &str) {
        if !self.vertex_data.contains_key(name) {
            self.set_attribute::<V>(name, vec![]);
        }
    }

    pub fn set_attribute<V: VertexAttribute>(&mut self, name: &str, data: Vec<V>) {
        let attr = VertexAttributeData::new(name.into(), data);

        self.vertex_data.insert(name.into(), attr);
        self.vertex_buffers.lock().unwrap().remove(name);
    }

    pub fn get_attribute<V: VertexAttribute>(&self, name: &str) -> Option<&[V]> {
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

    pub fn get_attribute_mut<V: VertexAttribute>(&mut self, name: &str) -> Option<&mut [V]> {
        if let Some(data) = self.vertex_data.get_mut(name) {
            self.vertex_buffers.lock().unwrap().remove(name);

            if V::format() == data.format {
                Some(cast_slice_mut(&mut data.data))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_attribute_mut_unmarked<V: VertexAttribute>(
        &mut self,
        name: &str,
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

    pub fn invalidate_vertex_buffer(&self, name: &str) {
        self.vertex_buffers.lock().unwrap().remove(name);
    }

    pub fn create_vertex_buffer(&self, name: &String, instance: &Instance) {
        if let Some(data) = self.vertex_data.get(name) {
            let buffer = instance
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

    pub fn create_vertex_buffers(&self, pipeline: &PipelineLayout, instance: &Instance) {
        for (name, _attribute) in &pipeline.vertex_attributes {
            if !self.has_vertex_buffer(name) {
                self.create_vertex_buffer(name, instance);
            }
        }
    }

    pub fn has_vertex_buffer(&self, name: &String) -> bool {
        self.vertex_buffers.lock().unwrap().contains_key(name)
    }

    pub fn get_vertex_buffer(&self, name: &String) -> Option<Arc<wgpu::Buffer>> {
        self.vertex_buffers.lock().unwrap().get(name).cloned()
    }

    pub fn create_index_buffer(&self, instance: &Instance) {
        let index_buffer = instance
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
