use glam::f32::Vec3;
use crate::hit::Hit;
use crate::ray::Ray;
use rand::distributions::{UnitSphereSurface, Distribution};

#[derive(Copy, Clone)]
pub enum Material {
    Lambert { albedo: Vec3 },
    Metal { albedo: Vec3, fuzz: f32 },
    Dielectric { albedo: Vec3, ref_idx: f32 }
}

fn random_in_unit_sphere() -> Vec3 {
    let unit_sphere = UnitSphereSurface::new();
    let sample = unit_sphere.sample(&mut rand::thread_rng());
    Vec3::new(sample[0] as f32, sample[1] as f32, sample[2] as f32)
}

fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2.0 * Vec3::dot(v, n) * n
}

fn refract(v: &Vec3, n: &Vec3, ni_over_nt: f32) -> Option<Vec3> {
    let v_norm = v.normalize();
    let dot = Vec3::dot(v_norm, *n);
    let delta = 1.0 - ni_over_nt * ni_over_nt * (1.0 - dot * dot);

    if delta > 0.0 {
        Some(ni_over_nt * (v_norm - *n * dot) - *n * delta.sqrt())
    } else {
        None
    }
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
            Material::Dielectric { albedo, ref_idx } => {
                let outward_normal;
                let ni_over_nt;
                if Vec3::dot(r.dir, hit.normal) > 0.0 {
                    outward_normal = -hit.normal;
                    ni_over_nt = ref_idx;
                } else {
                    outward_normal = hit.normal;
                    ni_over_nt = 1.0 / ref_idx;
                }

                if let Some(refract_dir) = refract(&r.dir, &outward_normal, ni_over_nt) {
                    return (albedo, Some(Ray::new(hit.pos, refract_dir)))
                }

                (albedo, Some(Ray::new(hit.pos, reflect(r.dir, hit.normal))))
            }
        }
    }
}