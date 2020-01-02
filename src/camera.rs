use crate::math::random_in_unit_disk;
use crate::ray::Ray;
use glam::f32::Vec3;
use glam::Vec2;

pub struct Camera {
    pub origin: Vec3,
    pub lower_left: Vec3,
    pub horizontal: Vec3,
    pub vertical: Vec3,
    pub u: Vec3,
    pub v: Vec3,
    pub w: Vec3,
    pub lens_radius: f32,
}

impl Camera {
    pub fn new(
        origin: Vec3,
        lookat: Vec3,
        up: Vec3,
        vertical_fov: f32,
        aspect: f32,
        aperture: f32,
        focus_dist: f32,
    ) -> Camera {
        let theta = vertical_fov.to_radians();
        let half_height = (theta / 2.0).tan();
        let half_width = aspect * half_height;

        let w = (origin - lookat).normalize();
        let u = Vec3::cross(up, w).normalize();
        let v = Vec3::cross(w, u);

        let lens_radius = aperture / 2.0;

        Camera {
            origin,
            lower_left: origin
                - (focus_dist * half_width * u)
                - (focus_dist * half_height * v)
                - (focus_dist * w),
            horizontal: 2.0 * half_width * focus_dist * u,
            vertical: 2.0 * half_height * focus_dist * v,
            lens_radius,
            u,
            v,
            w,
        }
    }

    pub fn get_ray(&self, uv: Vec2) -> Ray {
        let sample_in_radius = self.lens_radius * random_in_unit_disk().extend(0.0);
        let offset = self.u * sample_in_radius.x() + self.v * sample_in_radius.y();
        Ray::new(
            self.origin + offset,
            (self.lower_left - self.origin - offset)
                + (uv.x() * self.horizontal)
                + (uv.y() * self.vertical),
        )
    }
}
