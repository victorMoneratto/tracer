use glam::f32::Vec3;

pub enum Material {
    Lambert { albedo: Vec3 },
    Metal { albedo: Vec3 },
}
