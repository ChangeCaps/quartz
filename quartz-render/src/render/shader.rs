use crate::render::*;
use std::path::Path;

pub struct Shader {
    pub(crate) vs_spirv: shaderc::CompilationArtifact,
    pub(crate) fs_spirv: shaderc::CompilationArtifact,
}

impl Shader {
    pub fn load(vs_path: impl AsRef<Path>, fs_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let vs_src = std::fs::read_to_string(vs_path)?;
        let fs_src = std::fs::read_to_string(fs_path)?;

        Self::from_glsl(&vs_src, &fs_src)
    }

    pub fn from_glsl(vs_src: &str, fs_src: &str) -> anyhow::Result<Self> {
        let mut compiler = shaderc::Compiler::new()
            .ok_or(anyhow::Error::msg("Failed to create shaderc compiler"))?;
        let vs_spirv = compiler.compile_into_spirv(
            vs_src,
            shaderc::ShaderKind::Vertex,
            "vertex shader",
            "main",
            None,
        )?;

        let fs_spirv = compiler.compile_into_spirv(
            fs_src,
            shaderc::ShaderKind::Fragment,
            "fragment shader",
            "main",
            None,
        )?;

        Ok(Self { vs_spirv, fs_spirv })
    }

    pub fn to_modules(
        &self,
        render_resource: &RenderResource,
    ) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
        let vs_data = wgpu::util::make_spirv(self.vs_spirv.as_binary_u8());
        let fs_data = wgpu::util::make_spirv(self.fs_spirv.as_binary_u8());

        let vs_module =
            render_resource
                .device
                .create_shader_module(&wgpu::ShaderModuleDescriptor {
                    label: Some("Vertex Shader"),
                    source: vs_data,
                    flags: wgpu::ShaderFlags::default(),
                });
        let fs_module =
            render_resource
                .device
                .create_shader_module(&wgpu::ShaderModuleDescriptor {
                    label: Some("Fragment Shader"),
                    source: fs_data,
                    flags: wgpu::ShaderFlags::default(),
                });

        (vs_module, fs_module)
    }
}
