#![allow(unused_imports)]
use std::time::Instant;

use glium::backend::glutin::glutin::dpi::LogicalSize;
use glium::backend::glutin::glutin::{
    ContextBuilder, Event, EventsLoop, GlProfile, GlRequest, Robustness, Window,
};
use glium::glutin::WindowBuilder;
use glium::texture::{
    srgb_texture2d::SrgbTexture2d, RawImage2d, Texture2d, UncompressedFloatFormat,
};
use glium::{implement_vertex, uniform, Display, Program, Surface, VertexBuffer};

use glium::backend::Facade;
use glium::framebuffer::SimpleFrameBuffer;
use glium::index::{IndicesSource, NoIndices};
use glium::texture::{MipmapsOption, SrgbFormat};
use glium::uniforms::{EmptyUniforms, MagnifySamplerFilter};

#[derive(Clone, Copy)]
struct Vertex {
    pub attr_pos: [f32; 2],
    pub attr_uv: [f32; 2],
}

fn main() {
    let display_size = LogicalSize::new(1280.0, 720.0);
    let resolution_scale = 0.325;
    let resolution: (u32, u32) = (
        (display_size.width * resolution_scale).round() as _,
        (display_size.height * resolution_scale).round() as _,
    );

    let (display, mut events_loop) = create_window(display_size);

    let geometry = create_fullscreen_geometry(&display);

    let composite_program = compile_composite_shader(&display);

    let composite = SrgbTexture2d::empty_with_format(
        &display,
        SrgbFormat::U8U8U8U8,
        MipmapsOption::NoMipmap,
        resolution.0,
        resolution.1,
    )
    .unwrap();

    let accumulation = SrgbTexture2d::empty_with_format(
        &display,
        SrgbFormat::U8U8U8U8,
        MipmapsOption::NoMipmap,
        resolution.0,
        resolution.1,
    )
    .unwrap();
    let mut can_accumulate = true;

    let mut closed = false;
    while !closed {
        events_loop.poll_events(|ev| match ev {
            Event::WindowEvent { event, .. } => match event {
                glium::glutin::WindowEvent::CloseRequested => closed = true,
                _ => (),
            },
            _ => (),
        });

        let instant_before_render = Instant::now();

        let render = SrgbTexture2d::with_format(
            &display,
            glium::texture::RawImage2d::from_raw_rgba(
                tracer::trace::image((resolution.0 as _, resolution.1 as _), 1, 10),
                resolution,
            ),
            SrgbFormat::U8U8U8U8,
            MipmapsOption::NoMipmap,
        )
        .unwrap();

        // composite render with accumulated image
        let mut composite_surface = SimpleFrameBuffer::new(&display, &composite).unwrap();
        composite_surface.draw(
            &geometry.0, &geometry.1,
            &composite_program,
            &uniform!(
                render_tex: &render,
                accumulation_tex: &accumulation,
                history_alpha: if can_accumulate { 0.975f32 } else { 0.0f32 }),
            &Default::default()).unwrap();
        can_accumulate = true;

        // update accumulated image
        let accumulation_surface = SimpleFrameBuffer::new(&display, &accumulation).unwrap();
        composite_surface.fill(&accumulation_surface, MagnifySamplerFilter::Linear);

        // bit to present
        let frame = display.draw();
        composite_surface.fill(&frame, MagnifySamplerFilter::Linear);
        frame.finish().unwrap();


        let instant_after_render = Instant::now();
        let render_time_in_seconds =
            (instant_after_render - instant_before_render).as_nanos() as f64 * 1e-9;

        let gl_window = display.gl_window();
        let window: &Window = gl_window.window();
        window.set_title(&format!(
            "Tracer | {:0.02}ms @ {}x{}",
            render_time_in_seconds * 1e3,
            resolution.0,
            resolution.1
        ));
    }
}

fn create_fullscreen_geometry(display: &Display) -> (VertexBuffer<Vertex>, NoIndices) {
    implement_vertex!(Vertex, attr_pos, attr_uv);
    let vertices = vec![
        Vertex {
            attr_pos: [-1.0, -1.0],
            attr_uv: [0.0, 0.0],
        },
        Vertex {
            attr_pos: [-1.0, 3.0],
            attr_uv: [0.0, 2.0],
        },
        Vertex {
            attr_pos: [3.0, -1.0],
            attr_uv: [2.0, 0.0],
        },
    ];
    let vbo = glium::VertexBuffer::new(display, &vertices).unwrap();
    let ibo = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

    (vbo, ibo)
}

fn create_window(size: LogicalSize) -> (Display, EventsLoop) {
    let wb = WindowBuilder::new()
        .with_dimensions(size)
        .with_resizable(false)
        .with_title("Tracer");
    let cb = ContextBuilder::new()
        .with_gl(GlRequest::Latest)
        .with_gl_profile(GlProfile::Core)
        .with_double_buffer(Option::Some(true))
        .with_gl_debug_flag(true)
        .with_gl_robustness(Robustness::NoError)
        .with_hardware_acceleration(Option::Some(true))
        // NOTE disabled because it seems to causes weird behaviors
        // when blitting to backbuffer (e.g. only blitting if src_rect
        // and dst_rect are equal).
        //.with_multisampling(2)
        .with_pixel_format(24, 8)
        // NOTE due to a driver bug (?), we can't get a non-srgb backbuffer
        // this hint does nothing
        //.with_srgb(true)
        .with_vsync(false)
        .with_stencil_buffer(8);
    let events_loop = EventsLoop::new();
    let display = Display::new(wb, cb, &events_loop).unwrap();
    (display, events_loop)
}

fn compile_composite_shader(display: &Display) -> Program {
    let vertex = r#"
        #version 460

        layout(location = 0) in vec2 attr_pos;
        layout(location = 1) in vec2 attr_uv;

        layout(location = 2) out vec2 uv;

        void main() {
            uv = attr_uv;
            gl_Position = vec4(attr_pos, 0.0, 1.0);
        }
    "#;
    let fragment = r#"
        #version 460

        layout(binding = 0) uniform sampler2D render_tex;
        layout(binding = 2) uniform sampler2D accumulation_tex;

        layout(location = 0) uniform float history_alpha;

        layout(location = 2) in vec2 uv;

        layout(location = 0) out vec4 color;

        void main() {
            color = mix(texture(render_tex, uv), texture(accumulation_tex, uv), history_alpha);
        }
    "#;
    glium::Program::from_source(display, vertex, fragment, None).unwrap()
}
