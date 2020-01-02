use std::ops::Div;
use std::time::Instant;
use tracer::trace::image;

fn main() {
    let instant_before_tracing = Instant::now();

    let dimensions = (1280, 720);
    let samples = 1000;
    let depth = 100;
    let mut buffer = image(dimensions, samples, depth);

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
