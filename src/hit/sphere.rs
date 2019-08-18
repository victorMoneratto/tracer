use crate::hit::{Hit, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use glam::f32::Vec3;

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub mat: Material,
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, range: [f32; 2]) -> Option<Hit> {
        // t²*dot(dir,dir) + t*2*dot(dir, oc) + dot(oc, oc)-r² = 0
        let oc = r.origin - self.center;
        let a = Vec3::dot(r.dir, r.dir);
        let b = 2.0 * Vec3::dot(r.dir, oc);
        let c = Vec3::dot(oc, oc) - self.radius * self.radius;
        let delta = b * b - 4.0 * a * c;

        if delta <= 0.0 {
            return None;
        }

        let t = (-b - delta.sqrt()) / (2.0 * a);
        if t > range[0] && t < range[1] {
            let pos = r.point_at(t);
            let normal = Vec3::normalize(pos - self.center);
            return Some(Hit { t, pos, normal });
        }

        let t = (-b + delta.sqrt()) / (2.0 * a);
        if t > range[0] && t < range[1] {
            let pos = r.point_at(t);
            let normal = (pos - self.center) / self.radius;
            return Some(Hit { t, pos, normal });
        }
        None
    }
}

impl Hittable for Vec<Sphere> {
    fn hit(&self, r: &Ray, range: [f32; 2]) -> Option<Hit> {
        let mut closest_hit = None;
        let mut range = range;
        for sphere in self {
            if let Some(hit) = sphere.hit(r, range) {
                range[1] = hit.t;
                closest_hit = Some(hit);
            }
        }
        closest_hit
    }
}
