pub trait TextureFormat: Send + Sync + 'static {
    fn format(target_format: wgpu::TextureFormat) -> wgpu::TextureFormat;
}

pub struct TargetFormat;

impl TextureFormat for TargetFormat {
    fn format(target_format: wgpu::TextureFormat) -> wgpu::TextureFormat {
        target_format
    }
}

macro_rules! format {
    ($ident:ident) => {
        pub struct $ident;

        impl TextureFormat for $ident {
            fn format(_target_format: wgpu::TextureFormat) -> wgpu::TextureFormat {
                wgpu::TextureFormat::$ident
            }
        }
    };
}

format!(Rgba8UnormSrgb);
format!(Rgba8Unorm);
format!(Depth32Float);
