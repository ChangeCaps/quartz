use crate::prelude::*;
use std::sync::Arc;

pub struct Instance {
    pub(crate) device: Arc<wgpu::Device>,
    pub(crate) queue: wgpu::Queue,
}

pub struct SwapChain {
    pub(crate) surface: wgpu::Surface,
    pub(crate) desc: wgpu::SwapChainDescriptor,
    pub(crate) swap_chain: wgpu::SwapChain,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl Instance {
    pub async fn new(
        window: &impl raw_window_handle::HasRawWindowHandle,
        width: u32,
        height: u32,
    ) -> (Instance, SwapChain) {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface).unwrap(), // FIXME: unwrap
            width: width,
            height: height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let instance = Instance {
            device: Arc::new(device),
            queue,
        };

        let swap_chain = SwapChain {
            surface,
            desc: sc_desc,
            swap_chain,
            width,
            height,
        };

        (instance, swap_chain)
    }

    pub fn render(&self) -> RenderCtx<'_> {
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("RenderCtx Encoder"),
            });

        RenderCtx {
            instance: self,
            encoder: Some(encoder),
        }
    }
}

impl SwapChain {
    pub fn resize(&mut self, width: u32, height: u32, instance: &Instance) {
        self.width = width;
        self.height = height;
        self.desc.width = self.width;
        self.desc.height = self.height;
        self.swap_chain = instance.device.create_swap_chain(&self.surface, &self.desc);
    }

    pub fn format(&self) -> format::TargetFormat {
        format::TargetFormat(self.desc.format)
    }

    pub fn next_frame(
        &self,
        mut f: impl FnMut(TextureView<'_, format::TargetFormat>),
    ) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?;

        let view = TextureView {
            view: ViewInner::Borrowed(&frame.output.view),
            download: None,
            extent: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            _marker: Default::default(),
        };

        f(view);

        Ok(())
    }
}
