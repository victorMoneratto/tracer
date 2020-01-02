use crate::camera::Camera;
use crate::hit::Hittable;
use crate::material::Material;
use crate::ray::Ray;
use crate::sphere::Sphere;
use glam::{Vec2, Vec3};
use rayon::prelude::*;
use std::fs::File;
use std::io;
use std::io::Write;

pub fn image(dimensions: (usize, usize), samples: i32, depth: i32) -> Vec<u8> {
    let look_at = Vec3::new(0.0, 0.0, -1.0);
    let look_from = Vec3::new(0.0, 0.25, 0.0);

    let camera = &Camera::new(
        look_from,
        look_at,
        Vec3::new(0.0, 1.0, 0.0),
        100.0,
        dimensions.0 as f32 / dimensions.1 as f32,
        0.025,
        (look_at - look_from).length(),
    );

    let world = &vec![
        Sphere::new(
            Vec3::new(0.0, 0.0, -1.0),
            0.5,
            Material::Lambert {
                albedo: Vec3::new(0.1, 0.2, 0.5),
            },
        ),
        Sphere::new(
            Vec3::new(0.0, -100.5, -1.0),
            100.0,
            Material::Lambert {
                albedo: Vec3::new(0.6, 0.6, 0.4),
            },
        ),
        Sphere::new(
            Vec3::new(1.0, 0.0, -1.0),
            0.5,
            Material::Metal {
                albedo: Vec3::new(0.8, 0.6, 0.2),
                fuzz: 0.3,
            },
        ),
        Sphere::new(
            Vec3::new(-1.0, 0.0, -1.0),
            0.5,
            Material::Dielectric {
                albedo: Vec3::new(0.9, 0.8, 0.8),
                ref_idx: 1.5,
            },
        ),
        Sphere::new(
            Vec3::new(-1.0, 0.0, -1.0),
            -0.45,
            Material::Dielectric {
                albedo: Vec3::one(),
                ref_idx: 1.5,
            },
        ),
    ];

    let buffer = (0..dimensions.1)
        .into_par_iter()
        .flat_map(|y| {
            (0..dimensions.0).into_par_iter().flat_map(move |x| {
                let mut color = Vec3::zero();
                for _ in 0..samples {
                    let offset = Vec2::new(rand::random(), rand::random());
                    let uv = Vec2::new(
                        (offset.x() + x as f32) / dimensions.0 as f32,
                        (offset.y() + y as f32) / dimensions.1 as f32,
                    );
                    color += trace(&camera.get_ray(uv), &world, depth);
                }
                color /= samples as f32;

                color = color.max(Vec3::zero());
                color = color.min(Vec3::one());

                const GAMMA: f32 = 2.2;
                color = Vec3::new(
                    color.x().powf(1.0 / GAMMA),
                    color.y().powf(1.0 / GAMMA),
                    color.z().powf(1.0 / GAMMA),
                );

                let (r, g, b, a) = (
                    (color.x() * 255.99) as u8,
                    (color.y() * 255.99) as u8,
                    (color.z() * 255.99) as u8,
                    255 as u8,
                );

                (0..4).into_par_iter().map(move |i| match i {
                    0 => r,
                    1 => g,
                    2 => b,
                    _ => a,
                } as u8)
            })
        })
        .collect();

    buffer
}

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

pub fn write_to_tga(path: &str, dimensions: (usize, usize), buffer: &mut [u8]) -> io::Result<()> {
    let mut file = File::create(path)?;

    let tga_header: Vec<u8> = vec![
        0, // ID length
        0, // no color map
        2, // uncompressed, true color
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0, // x and y origin
        (dimensions.0 & 0x00FF) as u8,
        ((dimensions.0 & 0xFF00) >> 8) as u8,
        (dimensions.1 & 0x00FF) as u8,
        ((dimensions.1 & 0xFF00) >> 8) as u8,
        32, // 32 bit bitmap
        0,
    ];
    file.write(&tga_header)?;

    // TODO for now we mutate the source buffer
    for i in (0..buffer.len()).step_by(4) {
        let rgba = (buffer[i + 2], buffer[i + 1], buffer[i + 0], buffer[i + 3]);
        buffer[i + 0] = rgba.0;
        buffer[i + 1] = rgba.1;
        buffer[i + 2] = rgba.2;
        buffer[i + 3] = rgba.3;
    }
    file.write(buffer)?;

    Ok(())
}
