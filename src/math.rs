use glam::f32::Vec3;
use glam::Vec2;
use rand::distributions::{Distribution, UnitCircle, UnitSphereSurface};

pub fn random_in_unit_sphere() -> Vec3 {
    let unit_sphere = UnitSphereSurface::new();
    let sample = unit_sphere.sample(&mut rand::thread_rng());
    Vec3::new(sample[0] as f32, sample[1] as f32, sample[2] as f32)
}

pub fn random_in_unit_disk() -> Vec2 {
    let unit_disk = UnitCircle::new();
    let sample = unit_disk.sample(&mut rand::thread_rng());
    Vec2::new(sample[0] as f32, sample[1] as f32)
}

pub fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2.0 * Vec3::dot(v, n) * n
}

pub fn refract(v: &Vec3, n: &Vec3, ni_over_nt: f32) -> Option<Vec3> {
    let v_norm = v.normalize();
    let dot = Vec3::dot(v_norm, *n);
    let delta = 1.0 - ni_over_nt * ni_over_nt * (1.0 - dot * dot);

    if delta > 0.0 {
        Some(ni_over_nt * (v_norm - *n * dot) - *n * delta.sqrt())
    } else {
        None
    }
}

pub fn schlick(cos: f32, ref_idx: f32) -> f32 {
    let r0_sqrt = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0 = r0_sqrt * r0_sqrt;
    return r0 + (1.0 - r0) * (1.0 - cos).powi(5);
}
