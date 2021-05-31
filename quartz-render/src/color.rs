use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(
    Clone, Copy, Default, Debug, PartialEq, Pod, Zeroable, serde::Serialize, serde::Deserialize,
)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const ZERO: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_bytes(bytes: &[u8], format: &wgpu::TextureFormat) -> Self {
        match format {
            wgpu::TextureFormat::Rgba8UnormSrgb => Self::rgba(
                bytes[0] as f32 / 255.0,
                bytes[1] as f32 / 255.0,
                bytes[2] as f32 / 255.0,
                bytes[3] as f32 / 255.0,
            ),
            _ => panic!("format not supported"),
        }
    }

    pub fn into_bytes(&self, format: &wgpu::TextureFormat) -> Vec<u8> {
        match format {
            wgpu::TextureFormat::Rgba8UnormSrgb => vec![
                (self.r * 255.0).round() as u8,
                (self.g * 255.0).round() as u8,
                (self.b * 255.0).round() as u8,
                (self.a * 255.0).round() as u8,
            ],
            _ => panic!("format not supported"),
        }
    }

    pub fn lerp(self, other: Self, lerp: f32) -> Self {
        Self {
            r: other.r * lerp + self.r * (1.0 - lerp),
            g: other.g * lerp + self.g * (1.0 - lerp),
            b: other.b * lerp + self.b * (1.0 - lerp),
            a: other.a * lerp + self.a * (1.0 - lerp),
        }
    }
}

impl Into<wgpu::Color> for Color {
    fn into(self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: self.a as f64,
        }
    }
}
