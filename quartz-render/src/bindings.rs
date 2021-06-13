use crate::instance::*;
use std::{collections::HashMap, ops::Deref, sync::Arc};
use serde::de;
use wgpu::util::DeviceExt;

pub trait Bindable {
    fn new_binding(&self) -> Result<Binding, ()>;
    fn set(&self, binding: &mut Binding) -> Result<bool, ()>;
}

/// Implemented for anything that can be bound
#[derive(Clone, Debug)]
pub enum Binding {
    Texture {
        view: Arc<wgpu::TextureView>,
        dimension: wgpu::TextureViewDimension,
        multisampled: bool,
    },
    Sampler {
        sampler: Arc<wgpu::Sampler>,
    },
    UniformBlock {
        data: Vec<u8>,
        buffer: Option<Arc<wgpu::Buffer>>,
    },
}

impl Binding {
    pub fn sampler(sampler: Arc<wgpu::Sampler>) -> Self {
        Self::Sampler { sampler }
    }

    pub fn uniform_block(size: u64) -> Self {
        let data = vec![0; size as usize];

        Self::UniformBlock { data, buffer: None }
    }

    pub fn prepare(&mut self, instance: &Instance) {
        match self {
            Binding::UniformBlock { data, buffer } => {
                if let Some(buffer) = buffer {
                    instance.queue.write_buffer(buffer, 0, data);
                } else {
                    let new_buffer =
                        instance
                            .device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Uniform Buffer Binding"),
                                contents: data,
                                usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::UNIFORM,
                            });

                    *buffer = Some(Arc::new(new_buffer));
                };
            }
            _ => {}
        }
    }

    pub fn binding_resource(&self) -> wgpu::BindingResource {
        let binding_resource = match self {
            Binding::Texture { view, .. } => wgpu::BindingResource::TextureView(view),
            Binding::Sampler { sampler, .. } => wgpu::BindingResource::Sampler(sampler),
            Binding::UniformBlock { buffer, .. } => buffer.as_ref().unwrap().as_entire_binding(),
        };

        binding_resource
    }
}

#[derive(Default, Debug)]
pub struct Bindings {
    pub(crate) bindings: HashMap<u32, HashMap<u32, (Binding, bool)>>,
    pub(crate) bind_groups: HashMap<u32, Arc<wgpu::BindGroup>>,
    pub(crate) layout: HashMap<u32, wgpu::BindGroupLayout>,
}

impl Bindings {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            bind_groups: HashMap::new(),
            layout: HashMap::new(),
        }
    }

    pub fn bind(&mut self, set: u32, group: u32, bindable: &impl Bindable) {
        let bindings = self.bindings.entry(set).or_default();

        if let Some((binding, changed)) = bindings.get_mut(&group) {
            *changed |= bindable.set(binding).unwrap();
        } else {
            bindings.insert(group, (bindable.new_binding().unwrap(), true));
        }
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn generate_groups(&mut self, instance: &Instance) {
        for (set, group) in &mut self.bindings {
            let mut recreate = false;

            for (_index, (binding, changed)) in group.iter_mut() {
                if *changed {
                    binding.prepare(instance);
                }

                recreate |= *changed;
                *changed = false;
            }

            if !recreate {
                continue;
            }

            let mut entries = Vec::new();
            let mut layout_entries = Vec::new();

            for (index, (binding, _changed)) in group {
                layout_entries.push(wgpu::BindGroupLayoutEntry {
                    binding: *index,
                    count: None,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: match binding {
                        Binding::Sampler { .. } => wgpu::BindingType::Sampler {
                            filtering: true,
                            comparison: false,
                        },
                        Binding::Texture {
                            dimension,
                            multisampled,
                            ..
                        } => wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: *dimension,
                            multisampled: *multisampled,
                        },
                        Binding::UniformBlock { .. } => wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                });

                entries.push(wgpu::BindGroupEntry {
                    binding: *index,
                    resource: binding.binding_resource(),
                });
            }

            let layout_desc = wgpu::BindGroupLayoutDescriptor {
                label: Some("Binding group layout"),
                entries: &layout_entries,
            };

            let layout = instance.device.create_bind_group_layout(&layout_desc);

            let desc = wgpu::BindGroupDescriptor {
                label: Some("Bind group"),
                entries: &entries,
                layout: &layout,
            };

			let bind_group = instance.device.create_bind_group(&desc);

			self.layout.insert(*set, layout);
			self.bind_groups.insert(*set, Arc::new(bind_group));
        }
    }
}
