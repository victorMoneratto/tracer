use crate::hit::Hit;
use crate::ray::Ray;
use glam::f32::Vec3;
use crate::math::{random_in_unit_sphere, reflect, refract, schlick};

#[derive(Copy, Clone)]
pub enum Material {
    Lambert { albedo: Vec3 },
    Metal { albedo: Vec3, fuzz: f32 },
    Dielectric { albedo: Vec3, ref_idx: f32 },
}

impl Material {
    pub fn scatter(self, r: &Ray, hit: &Hit) -> (Vec3, Option<Ray>) {
        match self {
            Material::Lambert { albedo } => {
                let target = hit.pos + hit.normal + random_in_unit_sphere();
                (albedo, Some(Ray::new(hit.pos, target - hit.pos)))
            }

            Material::Metal { albedo, fuzz } => {
                let reflected_dir = reflect(r.dir, hit.normal) + fuzz * random_in_unit_sphere();
                if Vec3::dot(reflected_dir, hit.normal) > 0.0 {
                    (albedo, Some(Ray::new(hit.pos, reflected_dir)))
                } else {
                    (albedo, None)
                }
            }
            Material::Dielectric { albedo, ref_idx } => {
                let outward_normal;
                let ni_over_nt;
                let cos;
                if Vec3::dot(r.dir, hit.normal) > 0.0 {
                    outward_normal = -hit.normal;
                    ni_over_nt = ref_idx;
                    cos = ref_idx * Vec3::dot(r.dir, hit.normal) / r.dir.length();
                } else {
                    outward_normal = hit.normal;
                    ni_over_nt = 1.0 / ref_idx;
                    cos = -Vec3::dot(r.dir, hit.normal) / r.dir.length();
                }

                if let Some(refract_dir) = refract(&r.dir, &outward_normal, ni_over_nt) {
                    let reflect_prob = schlick(cos, ref_idx);
                    if rand::random::<f32>() >= reflect_prob {
                        return (albedo, Some(Ray::new(hit.pos, refract_dir)));
                    }
                }

                (albedo, Some(Ray::new(hit.pos, reflect(r.dir, hit.normal))))
            }
        }
    }
}
