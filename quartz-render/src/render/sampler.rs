use crate::render::*;
use std::sync::Arc;
pub use wgpu::AddressMode;
pub use wgpu::FilterMode;

pub struct SamplerDescriptor {
    pub address_mode: wgpu::AddressMode,
    pub filter: wgpu::FilterMode,
}

impl Default for SamplerDescriptor {
    fn default() -> Self {
        Self {
            address_mode: wgpu::AddressMode::Repeat,
            filter: wgpu::FilterMode::Linear,
        }
    }
}

#[derive(Clone)]
pub struct Sampler {
    pub(crate) sampler: Arc<wgpu::Sampler>,
}

impl Sampler {
    pub fn new(descriptor: &SamplerDescriptor, render_resource: &RenderResource) -> Self {
        let sampler = render_resource
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                label: None,
                address_mode_u: descriptor.address_mode,
                address_mode_v: descriptor.address_mode,
                address_mode_w: descriptor.address_mode,
                mag_filter: descriptor.filter,
                min_filter: descriptor.filter,
                mipmap_filter: descriptor.filter,
                lod_min_clamp: 0.0,
                lod_max_clamp: 0.0,
                compare: None,
                anisotropy_clamp: None,
                border_color: None,
            });

        Self {
            sampler: Arc::new(sampler),
        }
    }
}

impl Binding for Sampler {
    fn binding_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::Sampler(&self.sampler)
    }

    fn binding_clone(&self) -> Box<dyn Binding> {
        Box::new(Clone::clone(self))
    }
}
