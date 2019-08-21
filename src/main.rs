use crate::camera::Camera;
use crate::hit::sphere::Sphere;
use crate::hit::Hittable;
use crate::material::Material;
use crate::ray::Ray;
use glam::f32::Vec3;
use glam::Vec2;
use std::error::Error;
use std::fs::File;
use std::io::Write as IOWrite;
use std::fmt::Write;
use std::time::Instant;
use std::ops::Div;

mod camera;
mod hit;
mod material;
mod ray;
mod math;

const VIEW_DIMENSIONS : [i32; 2] = [854, 480];
const SAMPLES: i32 = 200;

fn trace(r: &Ray, world: &Vec<Sphere>, depth: i32) -> Vec3 {
    if let Some(hit) = world.hit(r, [1e-3, std::f32::MAX]) {
        if depth > 0 {
            let (attenuation, scattered) = hit.mat.scatter(r, &hit);
            if let Some(ray) = scattered {
                return attenuation * trace(&ray, world, depth - 1);
            }
        }
        return Vec3::zero();
    }

    let dir = r.dir.normalize();
    let t = dir.y() * 0.5 + 0.5;
    (1.0 - t) * Vec3::new(1.0, 1.0, 1.0) + t * Vec3::new(0.5, 0.7, 1.0)
}

fn main() -> Result<(), Box<dyn Error>> {
    let lookat = Vec3::new(0.0, 0.0, -1.0);
    let lookfrom = Vec3::new(3.0, 3.0, 2.0);

    let camera = Camera::new(
        lookfrom,
        lookat,
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
        VIEW_DIMENSIONS[0] as f32 / VIEW_DIMENSIONS[1] as f32,
        0.25,
        (lookat-lookfrom).length()
    );

    let world = vec![
        Sphere::new(
            Vec3::new(0.0, 0.0, -1.0), 0.5, Material::Lambert { albedo: Vec3::new(0.1, 0.2, 0.5), },
        ),
        Sphere::new(
            Vec3::new(0.0, -100.5, -1.0), 100.0, Material::Lambert { albedo: Vec3::new(0.6, 0.6, 0.4), },
        ),
        Sphere::new(
            Vec3::new(1.0, 0.0, -1.0), 0.5, Material::Metal { albedo: Vec3::new(0.8, 0.6, 0.2), fuzz: 0.3, },
        ),
        Sphere::new(
            Vec3::new(-1.0, 0.0, -1.0), 0.5, Material::Dielectric { albedo: Vec3::new(0.9, 0.8, 0.8), ref_idx: 1.5, },
        ),
        Sphere::new(
            Vec3::new(-1.0, 0.0, -1.0), -0.45, Material::Dielectric { albedo: Vec3::one(), ref_idx: 1.5, },
        ),
    ];

    let mut buffer = vec![0u8; (4 * VIEW_DIMENSIONS[0] * VIEW_DIMENSIONS[1]) as usize];

    let instant_before_tracing = Instant::now();
    for y in (0..VIEW_DIMENSIONS[1]).rev() {
        for x in 0..VIEW_DIMENSIONS[0] {
            let mut color = Vec3::zero();
            for _ in 0..SAMPLES
                {
                let offset = Vec2::new(rand::random(), rand::random());
                let uv = Vec2::new(
                    (offset.x() + x as f32) / VIEW_DIMENSIONS[0] as f32,
                    (offset.y() + y as f32) / VIEW_DIMENSIONS[1] as f32,
                );
                color += trace(&camera.get_ray(uv), &world, 50);
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

            let base: usize = 4 * (y * VIEW_DIMENSIONS[0] + x) as usize;
            let (r, g, b, a) = (
                (color.x() * 255.99) as u8,
                (color.y() * 255.99) as u8,
                (color.z() * 255.99) as u8,
                255 as u8);
            buffer[base + 0] = b;
            buffer[base + 1] = g;
            buffer[base + 2] = r;
            buffer[base + 3] = a;
        }
    }

    let time_elapsed_tracing = instant_before_tracing.elapsed();
    let time_per_pixel = time_elapsed_tracing.div((VIEW_DIMENSIONS[0] * VIEW_DIMENSIONS[1]) as u32);
    let time_per_sample = time_per_pixel.div(SAMPLES as u32);

    println!("{:>6.2} seconds", time_elapsed_tracing.as_millis() as f32 * 1e-3);
    println!("{:>6.2} micros/pixel", time_per_pixel.as_nanos() as f32 * 1e-3);
    println!("{:>6} nanos/sample", time_per_sample.as_nanos() as f32);

    let tga_header : Vec<u8> = vec![
            0, // ID length
            0, // no color map
            2, // uncompressed, true color
            0, 0, 0, 0, 0,
            0, 0, 0, 0, // x and y origin
            (VIEW_DIMENSIONS[0] & 0x00FF) as u8,
            ((VIEW_DIMENSIONS[0] & 0xFF00) >> 8) as u8,
            (VIEW_DIMENSIONS[1] & 0x00FF) as u8,
            ((VIEW_DIMENSIONS[1] & 0xFF00) >> 8) as u8,
            32, // 32 bit bitmap
            0
    ];

    let mut file = File::create("output.tga")?;
    file.write(tga_header.as_ref())?;
    file.write(buffer.as_ref())?;
    Ok(())
}
