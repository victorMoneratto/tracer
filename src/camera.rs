use crate::ray::Ray;
use glam::f32::Vec3;
use glam::Vec2;

pub struct Camera {
    pub origin: Vec3,
    pub lower_left: Vec3,
    pub horizontal: Vec3,
    pub vertical: Vec3,
}

impl Camera {
    pub fn get_ray(&self, uv: Vec2) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left - self.origin + uv.x() * self.horizontal + uv.y() * self.vertical,
        )
    }
}
