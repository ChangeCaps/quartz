use glam::*;

pub struct WindowDescriptor {
    pub(crate) size: Vec2,
    pub cursor_grabbed: bool,
    pub cursor_visible: bool,
}

impl Default for WindowDescriptor {
    fn default() -> Self {
        Self {
            size: Vec2::ZERO,
            cursor_grabbed: false,
            cursor_visible: true,
        }
    }
}

impl WindowDescriptor {
    pub fn aspect_ratio(&self) -> f32 {
        self.size.x / self.size.y
    }

    pub fn size(&self) -> Vec2 {
        self.size
    }
}
