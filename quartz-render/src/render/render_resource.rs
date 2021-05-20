use crate::render::*;
use glam::*;
use std::sync::Arc;
use winit::window::Window;

pub enum RenderTarget {
    Swapchain,
    Texture {
        format: wgpu::TextureFormat,
        view: Arc<wgpu::TextureView>,
        extent: wgpu::Extent3d,
    },
}

pub struct RenderResource {
    pub(crate) surface: wgpu::Surface,
    pub(crate) device: Arc<wgpu::Device>,
    pub(crate) queue: wgpu::Queue,
    pub(crate) sc_desc: wgpu::SwapChainDescriptor,
    pub(crate) swap_chain: wgpu::SwapChain,
    pub(crate) size: winit::dpi::PhysicalSize<u32>,
    pub(crate) render_target: RenderTarget,
}

impl RenderResource {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

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
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        Self {
            surface,
            device: Arc::new(device),
            queue,
            sc_desc,
            swap_chain,
            size,
            render_target: RenderTarget::Swapchain,
        }
    }

    pub fn target_format(&self) -> wgpu::TextureFormat {
        match &self.render_target {
            RenderTarget::Swapchain => self.sc_desc.format,
            RenderTarget::Texture { format, .. } => *format,
        }
    }

    pub fn resize_swapchain(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = self.size.width;
        self.sc_desc.height = self.size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    #[inline(always)]
    pub fn target_width(&self) -> u32 {
        match &self.render_target {
            RenderTarget::Swapchain => self.sc_desc.width,
            RenderTarget::Texture { extent, .. } => extent.width,
        }
    }

    #[inline(always)]
    pub fn target_height(&self) -> u32 {
        match &self.render_target {
            RenderTarget::Swapchain => self.sc_desc.height,
            RenderTarget::Texture { extent, .. } => extent.height,
        }
    }

    #[inline(always)]
    pub fn target_size(&self) -> Vec2 {
        Vec2::new(self.target_width() as f32, self.target_height() as f32)
    }

    pub fn target_texture<D: TextureDimension, F: TextureFormat>(
        &mut self,
        texture: &Texture<D, F>,
    ) {
        let texture_view = texture.view();

        texture_view
            .download
            .store(true, std::sync::atomic::Ordering::SeqCst);

        self.render_target = RenderTarget::Texture {
            extent: texture.dimensions.extent(),
            format: F::format(self.target_format()),
            view: texture_view.view,
        };
    }

    pub fn target_swapchain(&mut self) {
        self.render_target = RenderTarget::Swapchain;
    }

    pub fn render(&self, mut f: impl FnMut(&mut RenderCtx)) -> Result<(), wgpu::SwapChainError> {
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("RenderCtx Encoder"),
            });

        let render = |view| {
            let mut ctx = RenderCtx {
                render_resource: self,
                render_target: view,
                encoder,
            };

            f(&mut ctx);

            self.queue.submit(std::iter::once(ctx.encoder.finish()));
        };

        match &self.render_target {
            RenderTarget::Swapchain => {
                let view = &self.swap_chain.get_current_frame()?.output.view;

                render(view);
            }
            RenderTarget::Texture { view, .. } => {
                render(view);
            }
        }

        Ok(())
    }
}
