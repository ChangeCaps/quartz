pub mod bindings;
pub mod buffer;
pub mod color;
pub mod instance;
pub mod projection;
pub mod render_ctx;
pub mod render_pass;
pub mod render_pipeline;
pub mod sampler;
pub mod shader;
pub mod texture;
pub mod texture_format;
pub mod uniform;
pub mod vertex;

pub use glam;
pub use wgpu;

pub mod prelude {
    pub use crate::bindings::*;
    pub use crate::color::*;
    pub use crate::instance::*;
    pub use crate::projection::*;
    pub use crate::render_ctx::*;
    pub use crate::render_pass::*;
    pub use crate::render_pipeline::*;
    pub use crate::sampler::*;
    pub use crate::sampler::*;
    pub use crate::shader::*;
    pub use crate::texture::*;
    pub use crate::texture_format::{self as format, TextureFormat};
    pub use crate::uniform::*;
    pub use crate::vertex::*;
    pub use glam::{swizzles::*, *};
}
