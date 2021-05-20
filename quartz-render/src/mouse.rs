use crate::input::*;
use glam::*;
pub use winit::event::MouseButton;

#[derive(Default)]
pub struct MouseInput {
    pub input: Input<MouseButton>,
    pub prev_position: Vec2,
    pub position: Vec2,
    pub delta: Vec2,
}

impl MouseInput {
    pub fn pre_update(&mut self) {
        self.delta = self.position - self.prev_position;
    }

    pub fn post_update(&mut self) {
        self.prev_position = self.position;
        self.input.update();
    }
}
