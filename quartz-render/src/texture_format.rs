pub trait TextureFormat: Clone + Send + Sync + 'static {
    fn format(&self) -> wgpu::TextureFormat;
}

#[derive(Clone, Copy, Debug)]
pub struct TargetFormat(pub wgpu::TextureFormat);

impl TextureFormat for TargetFormat {
    fn format(&self) -> wgpu::TextureFormat {
        self.0
    }
}

macro_rules! format {
    ($ident:ident) => {
        #[derive(Clone, Copy, Debug, Default)]
        pub struct $ident;

        impl TextureFormat for $ident {
            fn format(&self) -> wgpu::TextureFormat {
                wgpu::TextureFormat::$ident
            }
        }
    };
}

format!(R32Uint);
format!(R32Sint);
format!(R32Float);
format!(Rgba8Unorm);
format!(Rgba8UnormSrgb);
format!(Depth32Float);
