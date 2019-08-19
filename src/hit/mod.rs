use crate::material::Material;
use crate::ray::Ray;
use glam::f32::Vec3;

pub mod sphere;

pub struct Hit {
    pub t: f32,
    pub pos: Vec3,
    pub normal: Vec3,
    pub mat: Material,
}

pub trait Hittable {
    fn hit(&self, r: &Ray, range: [f32; 2]) -> Option<Hit>;
}
