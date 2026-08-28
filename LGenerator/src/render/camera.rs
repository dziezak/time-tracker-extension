use glam::{Mat4, Vec3};

pub struct Camera3D {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect: f32,
    pub fov: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera3D {
    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = Mat4::perspective_rh(self.fov.to_radians(), self.aspect, self.znear, self.zfar);
        proj * view
    }

    pub fn update_orbit(&mut self, time: f32, radius: f32) {
        self.eye.x = time.cos() * radius;
        self.eye.z = time.sin() * radius;
        self.eye.y = 10.0 + (time * 0.5).sin() * 3.0; // Delikatne unoszenie góra/dół
    }
}