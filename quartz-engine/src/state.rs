use crate::render::prelude::*;
use crate::tree::*;

pub struct InitCtx<'a> {
    pub tree: &'a mut Tree,
    pub render_resource: &'a RenderResource,
}

pub trait State: 'static {
    fn init(&mut self, _ctx: InitCtx) {}
}
