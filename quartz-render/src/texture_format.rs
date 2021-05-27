use crate::color::*;
use bytemuck::{bytes_of, from_bytes};

pub trait TextureData: Default + Clone {
    fn from_bytes<F: TextureFormat>(bytes: &[u8], format: F) -> Self;
    fn to_bytes<F: TextureFormat>(&self, format: F) -> Vec<u8>;
}

pub trait TextureFormat: Clone + Send + Sync + 'static {
    type Data: TextureData;

    fn format(&self) -> wgpu::TextureFormat;
}

#[derive(Clone, Copy, Debug)]
pub struct TargetFormat(pub wgpu::TextureFormat);

impl TextureFormat for TargetFormat {
    type Data = Color;

    fn format(&self) -> wgpu::TextureFormat {
        self.0
    }
}

macro_rules! format {
    ($ident:ident) => {
        #[derive(Clone, Copy, Debug, Default)]
        pub struct $ident;

        impl TextureFormat for $ident {
            type Data = Color;

            fn format(&self) -> wgpu::TextureFormat {
                wgpu::TextureFormat::$ident
            }
        }
    };
    ($ident:ident: $data:path) => {
        #[derive(Clone, Copy, Debug, Default)]
        pub struct $ident;

        impl TextureFormat for $ident {
            type Data = $data;

            fn format(&self) -> wgpu::TextureFormat {
                wgpu::TextureFormat::$ident
            }
        }
    };
}

format!(R32Uint: u32);
format!(R32Sint: i32);
format!(R32Float: f32);
format!(Rgba8Unorm);
format!(Rgba8UnormSrgb);
format!(Depth32Float: f32);

impl TextureData for u32 {
    fn from_bytes<F: TextureFormat>(bytes: &[u8], _format: F) -> Self {
        *from_bytes(bytes)
    }

    fn to_bytes<F: TextureFormat>(&self, _format: F) -> Vec<u8> {
        bytes_of(self).to_vec()
    }
}

impl TextureData for i32 {
    fn from_bytes<F: TextureFormat>(bytes: &[u8], _format: F) -> Self {
        *from_bytes(bytes)
    }

    fn to_bytes<F: TextureFormat>(&self, _format: F) -> Vec<u8> {
        bytes_of(self).to_vec()
    }
}

impl TextureData for f32 {
    fn from_bytes<F: TextureFormat>(bytes: &[u8], _format: F) -> Self {
        *from_bytes(bytes)
    }

    fn to_bytes<F: TextureFormat>(&self, _format: F) -> Vec<u8> {
        bytes_of(self).to_vec()
    }
}

impl TextureData for Color {
    fn from_bytes<F: TextureFormat>(bytes: &[u8], format: F) -> Self {
        Color::from_bytes(bytes, &format.format())
    }

    fn to_bytes<F: TextureFormat>(&self, format: F) -> Vec<u8> {
        self.into_bytes(&format.format())
    }
}
