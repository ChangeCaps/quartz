use glam::*;

pub struct PerspectiveProjection {
    pub aspect: f32,
    pub fov: f32,
    pub far: f32,
    pub near: f32,
}

impl Default for PerspectiveProjection {
    fn default() -> Self {
        Self {
            aspect: 1.0,
            fov: std::f32::consts::PI / 2.0,
            far: 1000.0,
            near: 1.0,
        }
    }
}

impl PerspectiveProjection {
    pub fn matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }
}

pub struct OrthographicProjection {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub far: f32,
    pub near: f32,
}

impl Default for OrthographicProjection {
    fn default() -> Self {
        Self {
            left: -5.0,
            right: 5.0,
            bottom: -5.0,
            top: 5.0,
            far: 1000.0,
            near: 0.0,
        }
    }
}

impl OrthographicProjection {
    pub fn matrix(&self) -> Mat4 {
        Mat4::orthographic_rh(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        )
    }
}
