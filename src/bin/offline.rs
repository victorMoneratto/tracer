use glam::Vec3;
use std::ops::Div;
use std::time::Instant;
use tracer::camera::Camera;
use tracer::trace::image;

fn main() {
    let instant_before_tracing = Instant::now();

    let dimensions = (1280, 720);
    let samples = 1000;
    let depth = 100;

    let camera = &Camera::new(
        Vec3::new(0.0, 0.25, 0.0),
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::new(0.0, 1.0, 0.0),
        100.0,
        dimensions.0 as f32 / dimensions.1 as f32,
        0.025,
        1.0,
    );

    let mut buffer = image(camera, dimensions, samples, depth);

    let time_elapsed_tracing = instant_before_tracing.elapsed();
    let time_per_pixel = time_elapsed_tracing.div((dimensions.0 * dimensions.1) as u32);
    let time_per_sample = time_per_pixel.div(samples as u32);

    println!(
        "{:>6.2} seconds",
        time_elapsed_tracing.as_millis() as f32 * 1e-3
    );
    println!(
        "{:>6.2} micros/pixel",
        time_per_pixel.as_nanos() as f32 * 1e-3
    );
    println!("{:>6} nanos/sample", time_per_sample.as_nanos() as f32);

    tracer::trace::write_to_tga("output.tga", dimensions, &mut buffer).unwrap();
}
