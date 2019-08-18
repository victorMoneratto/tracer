use crate::camera::Camera;
use crate::hit::sphere::Sphere;
use crate::hit::Hittable;
use crate::material::Material;
use crate::ray::Ray;
use glam::f32::Vec3;
use glam::Vec2;

mod camera;
mod hit;
mod material;
mod ray;

const NY: i32 = 240;
const NX: i32 = 2 * NY;
const SAMPLES: i32 = 200;

fn scene(r: &Ray, world: &Vec<Sphere>, depth: i32) -> Vec3 {
    if let Some(hit) = world.hit(r, [1e-3, std::f32::MAX]) {
        if depth > 0 {
            let (attenuation, scattered) = hit.mat.scatter(r, &hit);
            if let Some(ray) = scattered {
                return attenuation * scene(&ray, world, depth-1);
            }
        }
        return Vec3::zero();
    }

    let dir = Vec3::normalize(r.dir);
    let t = dir.y() * 0.5 + 0.5;
    (1.0 - t) * Vec3::new(1.0, 1.0, 1.0) + t * Vec3::new(0.5, 0.7, 1.0)
}

fn main() {
    println!("P3");
    println!("{} {}", NX, NY);
    println!("255");

    let camera = Camera {
        origin: Vec3::new(0.0, 0.0, 0.0),
        lower_left: Vec3::new(-2.0, -1.0, -1.0),
        horizontal: Vec3::new(4.0, 0.0, 0.0),
        vertical: Vec3::new(0.0, 2.0, 0.0),
    };

    let world = vec![
        Sphere { center: Vec3::new(0.0, 0.0, -1.0), radius: 0.5, mat: Material::Lambert { albedo: Vec3::new(0.1, 0.2, 0.5), }, },
        Sphere { center: Vec3::new(0.0, -100.5, -1.0), radius: 100.0, mat: Material::Lambert { albedo: Vec3::new(0.8, 0.8, 0.0) }, },
        Sphere { center: Vec3::new(1.0, 0.0, -1.0), radius: 0.5, mat: Material::Metal { albedo: Vec3::new(0.8, 0.6, 0.2), fuzz: 0.3 }, },
        Sphere { center: Vec3::new(-1.0, 0.0, -1.0), radius: 0.5, mat: Material::Dielectric { albedo: Vec3::new(0.7, 0.9, 0.5), ref_idx: 1.5 }, },
        Sphere { center: Vec3::new(-1.0, 0.0, -1.0), radius: -0.45, mat: Material::Dielectric { albedo: Vec3::one(), ref_idx: 1.5 }, },
    ];

    for y in (0..NY).rev() {
        for x in 0..NX {
            let mut color = Vec3::zero();
            for _ in 0..SAMPLES {
                let offset = Vec2::new(rand::random(), rand::random());
                let uv = Vec2::new(
                    (offset.x() + x as f32) / NX as f32,
                    (offset.y() + y as f32) / NY as f32,
                );
                color += scene(&camera.get_ray(uv), &world, 50);
            }
            color /= SAMPLES as f32;

            color = color.max(Vec3::zero());
            color = color.min(Vec3::one());

            const GAMMA: f32 = 2.2;
            color = Vec3::new(
                color.x().powf(1.0 / GAMMA),
                color.y().powf(1.0 / GAMMA),
                color.z().powf(1.0 / GAMMA),
            );

            println!(
                "{} {} {}",
                (color.x() * 255.99) as u8,
                (color.y() * 255.99) as u8,
                (color.z() * 255.99) as u8
            )
        }
    }
}
