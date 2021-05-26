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

format!(Rgba8UnormSrgb);
format!(Rgba8Unorm);
format!(Depth32Float);
