use glam::f32::Vec3;
use crate::hit::Hit;
use crate::ray::Ray;
use rand::distributions::{UnitSphereSurface, Distribution};

#[derive(Copy, Clone)]
pub enum Material {
    Lambert { albedo: Vec3 },
    Metal { albedo: Vec3, fuzz: f32 },
}

fn random_in_unit_sphere() -> Vec3 {
    let unit_sphere = UnitSphereSurface::new();
    let sample = unit_sphere.sample(&mut rand::thread_rng());
    Vec3::new(sample[0] as f32, sample[1] as f32, sample[2] as f32)
}

fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2.0 * Vec3::dot(v, n) * n
}

impl Material {
    pub fn scatter(self, r: &Ray, hit: &Hit) -> (Vec3, Option<Ray>) {
        match self {
            Material::Lambert { albedo } => {
                let target = hit.pos + hit.normal + random_in_unit_sphere();
                (albedo, Some(Ray::new(hit.pos, target - hit.pos)))
            },

            Material::Metal { albedo, fuzz } => {
                let reflected_dir = reflect(r.dir, hit.normal) + fuzz * random_in_unit_sphere();
                if Vec3::dot(reflected_dir, hit.normal) > 0.0 {
                    (albedo, Some(Ray::new(hit.pos, reflected_dir)))
                } else {
                    (albedo, None)
                }
            },
        }
    }
}