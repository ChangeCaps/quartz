use crate::color::*;
use crate::prelude::*;
use format::*;
use futures::executor::block_on;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, RwLock,
};

pub trait TextureDimension: Clone + 'static {
    type Data;

    fn init_data(&self) -> Self::Data;
    fn data_to_bytes(data: &Self::Data, format: &wgpu::TextureFormat) -> Vec<u8>;
    fn bytes_to_data(&self, data: &mut Self::Data, bytes: &[u8], format: &wgpu::TextureFormat);
    fn get_dimension(&self) -> wgpu::TextureDimension;
    fn extent(&self) -> wgpu::Extent3d;
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct D1 {
    pub width: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct D2 {
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct D2Array {
    pub width: u32,
    pub height: u32,
    pub layers: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct D3 {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl D1 {
    pub const fn new(width: u32) -> Self {
        Self { width }
    }
}

impl TextureDimension for D1 {
    type Data = Vec<Color>;

    fn init_data(&self) -> Self::Data {
        vec![Color::ZERO; self.width as usize]
    }

    fn data_to_bytes(data: &Self::Data, format: &wgpu::TextureFormat) -> Vec<u8> {
        data.iter()
            .map(|color| color.into_bytes(format))
            .flatten()
            .collect()
    }

    fn bytes_to_data(&self, data: &mut Self::Data, bytes: &[u8], format: &wgpu::TextureFormat) {
        let info = format.describe();
        let block_size = info.block_size as usize;

        data.iter_mut().enumerate().for_each(|(x, color)| {
            let index = x * block_size;

            *color = Color::from_bytes(&bytes[index..index + block_size], format);
        });
    }

    fn get_dimension(&self) -> wgpu::TextureDimension {
        wgpu::TextureDimension::D1
    }

    fn extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.width,
            height: 1,
            depth_or_array_layers: 1,
        }
    }
}

impl D2 {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl TextureDimension for D2 {
    type Data = Vec<Vec<Color>>;

    fn init_data(&self) -> Self::Data {
        vec![vec![Color::ZERO; self.height as usize]; self.width as usize]
    }

    fn data_to_bytes(data: &Self::Data, format: &wgpu::TextureFormat) -> Vec<u8> {
        data.iter()
            .map(|data| data.iter().map(|color| color.into_bytes(format)).flatten())
            .flatten()
            .collect()
    }

    fn bytes_to_data(&self, data: &mut Self::Data, bytes: &[u8], format: &wgpu::TextureFormat) {
        let info = format.describe();
        let block_size = info.block_size as usize;
        let row_size = data_width(self.width) as usize * block_size;

        data.iter_mut().enumerate().for_each(|(x, data)| {
            data.iter_mut().enumerate().for_each(|(y, color)| {
                let index = y * row_size + x * block_size;

                *color = Color::from_bytes(&bytes[index..index + block_size], format);
            });
        });
    }

    fn get_dimension(&self) -> wgpu::TextureDimension {
        wgpu::TextureDimension::D2
    }

    fn extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        }
    }
}

impl D3 {
    pub const fn new(width: u32, height: u32, depth: u32) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }
}

impl TextureDimension for D3 {
    type Data = Vec<Vec<Vec<Color>>>;

    fn init_data(&self) -> Self::Data {
        vec![
            vec![vec![Color::ZERO; self.depth as usize]; self.height as usize];
            self.width as usize
        ]
    }

    fn data_to_bytes(data: &Self::Data, format: &wgpu::TextureFormat) -> Vec<u8> {
        data.iter()
            .map(|data| {
                data.iter()
                    .map(|data| data.iter().map(|color| color.into_bytes(format)).flatten())
                    .flatten()
            })
            .flatten()
            .collect()
    }

    fn bytes_to_data(&self, data: &mut Self::Data, bytes: &[u8], format: &wgpu::TextureFormat) {
        let info = format.describe();
        let block_size = info.block_size as usize;
        let row_size = data_width(self.width) as usize * block_size;
        let image_size = row_size * self.height as usize;

        data.iter_mut().enumerate().for_each(|(x, data)| {
            data.iter_mut().enumerate().for_each(|(y, data)| {
                data.iter_mut().enumerate().for_each(|(z, color)| {
                    let index = z * image_size + y * row_size + x * block_size;

                    *color = Color::from_bytes(&bytes[index..index + block_size], format);
                });
            });
        });
    }

    fn get_dimension(&self) -> wgpu::TextureDimension {
        wgpu::TextureDimension::D3
    }

    fn extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: self.depth,
        }
    }
}

impl D2Array {
    pub const fn new(width: u32, height: u32, layers: u32) -> Self {
        Self {
            width,
            height,
            layers,
        }
    }
}

impl TextureDimension for D2Array {
    type Data = Vec<Vec<Vec<Color>>>;

    fn init_data(&self) -> Self::Data {
        vec![
            vec![vec![Color::ZERO; self.layers as usize]; self.height as usize];
            self.width as usize
        ]
    }

    fn data_to_bytes(data: &Self::Data, format: &wgpu::TextureFormat) -> Vec<u8> {
        data.iter()
            .map(|data| {
                data.iter()
                    .map(|data| data.iter().map(|color| color.into_bytes(format)).flatten())
                    .flatten()
            })
            .flatten()
            .collect()
    }

    fn bytes_to_data(&self, data: &mut Self::Data, bytes: &[u8], format: &wgpu::TextureFormat) {
        let info = format.describe();
        let block_size = info.block_size as usize;
        let row_size = data_width(self.width) as usize * block_size;
        let image_size = row_size * self.height as usize;

        data.iter_mut().enumerate().for_each(|(x, data)| {
            data.iter_mut().enumerate().for_each(|(y, data)| {
                data.iter_mut().enumerate().for_each(|(z, color)| {
                    let index = z * image_size + y * row_size + x * block_size;

                    *color = Color::from_bytes(&bytes[index..index + block_size], format);
                });
            });
        });
    }

    fn get_dimension(&self) -> wgpu::TextureDimension {
        wgpu::TextureDimension::D2
    }

    fn extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: self.layers,
        }
    }
}

pub struct TextureDescriptor<D: TextureDimension, F: TextureFormat> {
    pub dimension: D,
    pub format: F,
}

impl<D: TextureDimension, F: TextureFormat + Default> TextureDescriptor<D, F> {
    pub fn default_settings(dimension: D) -> Self {
        Self {
            dimension,
            format: Default::default(),
        }
    }
}

#[inline(always)]
fn data_width(width: u32) -> u32 {
    ((width - 1) / wgpu::COPY_BYTES_PER_ROW_ALIGNMENT + 1) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
}

pub struct Texture<D: TextureDimension, F: TextureFormat = Rgba8UnormSrgb> {
    pub(crate) texture: wgpu::Texture,
    pub(crate) staging_buffer: Mutex<Option<wgpu::Buffer>>,
    pub(crate) view: Arc<wgpu::TextureView>,
    pub(crate) data: RwLock<Option<D::Data>>,
    pub(crate) download: Arc<AtomicBool>,
    pub(crate) format: F,
    pub dimensions: D,
}

impl<D: TextureDimension, F: TextureFormat> Texture<D, F> {
    pub fn new(texture_descriptor: &TextureDescriptor<D, F>, instance: &Instance) -> Self {
        let texture = instance.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_descriptor.dimension.extent(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: texture_descriptor.dimension.get_dimension(),
            format: texture_descriptor.format.format(),
            usage: wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::RENDER_ATTACHMENT
                | wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::COPY_DST,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: None,
            dimension: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        Self {
            texture: texture,
            view: Arc::new(view),
            staging_buffer: Mutex::new(None),
            data: RwLock::new(None),
            dimensions: texture_descriptor.dimension.clone(),
            download: Arc::new(AtomicBool::new(true)),
            format: texture_descriptor.format.clone(),
        }
    }

    fn create_staging_buffer(&self, size: u64, instance: &Instance) {
        let mut staging_buffer = self.staging_buffer.lock().unwrap();

        if staging_buffer.is_none() {
            let buffer = instance.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Texture Staging Buffer"),
                size,
                mapped_at_creation: false,
                usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
            });

            *staging_buffer = Some(buffer)
        }
    }

    fn image_data_layout(&self, instance: &Instance) -> wgpu::ImageDataLayout {
        use std::num::NonZeroU32;

        let extent = self.dimensions.extent();
        let info = self.format.format().describe();

        let bytes_per_row = NonZeroU32::new(info.block_size as u32 * extent.width).unwrap();
        let rows_per_image = NonZeroU32::new(extent.height).unwrap();

        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(bytes_per_row),
            rows_per_image: Some(rows_per_image),
        }
    }

    pub fn download_data(&self, instance: &Instance) {
        use std::num::NonZeroU32;

        if !self.download.load(Ordering::SeqCst) {
            return;
        }

        self.download.store(false, Ordering::SeqCst);

        let extent = self.dimensions.extent();
        let format = self.format.format();
        let info = format.describe();

        let block_size = info.block_size as u32;
        let data_width = data_width(extent.width);
        let bytes_per_row = block_size * data_width;
        let rows_per_image = extent.height;

        self.create_staging_buffer(bytes_per_row as u64 * rows_per_image as u64, instance);
        let staging_buffer = self.staging_buffer.lock().unwrap();
        let staging_buffer = staging_buffer.as_ref().unwrap();
        let mut encoder = instance
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Texture Read"),
            });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: staging_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(NonZeroU32::new(bytes_per_row).unwrap()),
                    rows_per_image: Some(NonZeroU32::new(rows_per_image).unwrap()),
                },
            },
            extent,
        );

        instance.queue.submit(std::iter::once(encoder.finish()));

        let slice = staging_buffer.slice(..);
        let future = slice.map_async(wgpu::MapMode::Read);
        instance.device.poll(wgpu::Maintain::Wait);
        block_on(future).unwrap();

        {
            let mapped = slice.get_mapped_range();

            let mut data = self.data.write().unwrap();

            if data.is_none() {
                *data = Some(self.dimensions.init_data());
            }

            self.dimensions
                .bytes_to_data(data.as_mut().unwrap(), &*mapped, &format);
        }

        staging_buffer.unmap();
    }

    pub fn read<T>(&self, instance: &Instance, mut f: impl FnMut(&D::Data) -> T) -> T {
        self.download_data(instance);
        let data = self.data.read().unwrap();
        f(data.as_ref().unwrap())
    }

    pub fn write<T>(&mut self, instance: &Instance, mut f: impl FnMut(&mut D::Data) -> T) -> T {
        self.download_data(instance);

        let mut data = self.data.write().unwrap();
        let res = f(data.as_mut().unwrap());

        let extent = self.dimensions.extent();

        instance.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &D::data_to_bytes(data.as_ref().unwrap(), &self.format.format()),
            self.image_data_layout(instance),
            extent,
        );

        res
    }

    pub fn view(&self) -> TextureView<'static, F> {
        TextureView {
            view: ViewInner::Owned(self.view.clone()),
            download: Some(self.download.clone()),
            extent: self.dimensions.extent(),
            format: self.format.clone(),
        }
    }
}

impl<F: TextureFormat> Texture<D2Array, F> {
    pub fn layer_view(&self, layer: u32) -> TextureView<'static, F> {
        let view = self.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Texture Array Layer View"),
            format: None,
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: layer,
            array_layer_count: None,
        });

        TextureView {
            view: ViewInner::Owned(Arc::new(view)),
            download: Some(self.download.clone()),
            extent: self.dimensions.extent(),
            format: self.format.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ViewInner<'a> {
    Owned(Arc<wgpu::TextureView>),
    Borrowed(&'a wgpu::TextureView),
}

impl<'a> ViewInner<'a> {
    pub(crate) fn view(&self) -> &wgpu::TextureView {
        match self {
            Self::Owned(view) => &view,
            Self::Borrowed(view) => view,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextureView<'a, F: TextureFormat = TargetFormat> {
    pub(crate) view: ViewInner<'a>,
    pub(crate) download: Option<Arc<AtomicBool>>,
    pub(crate) extent: wgpu::Extent3d,
    pub(crate) format: F,
}

impl<'a, F: TextureFormat> TextureView<'a, F> {
    pub fn width(&self) -> u32 {
        self.extent.width
    }

    pub fn height(&self) -> u32 {
        self.extent.height
    }

    pub fn extent(&self) -> wgpu::Extent3d {
        self.extent
    }

    pub fn format(&self) -> F {
        self.format.clone()
    }
}

impl<'a, F: TextureFormat> TextureView<'a, F> {
    pub(crate) fn view(&self) -> &wgpu::TextureView {
        self.view.view()
    }
}

impl<F: TextureFormat> Binding for TextureView<'static, F> {
    fn binding_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::TextureView(self.view.view())
    }

    fn binding_clone(&self) -> Box<dyn Binding> {
        Box::new(Clone::clone(self))
    }
}

pub type Texture1d<F = Rgba8UnormSrgb> = Texture<D1, F>;
pub type Texture2d<F = Rgba8UnormSrgb> = Texture<D2, F>;
pub type Texture3d<F = Rgba8UnormSrgb> = Texture<D3, F>;
pub type Texture2dArray<F = Rgba8UnormSrgb> = Texture<D2Array, F>;
