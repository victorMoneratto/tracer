#![allow(unused_imports)]
#![feature(clamp)]

use std::time::Instant;

use glium::backend::glutin::glutin::{
    dpi::LogicalSize, ContextBuilder, DeviceEvent, ElementState, Event, EventsLoop, GlProfile,
    GlRequest, KeyboardInput, MouseCursor, Robustness, VirtualKeyCode, Window, WindowEvent,
};
use glium::glutin::WindowBuilder;
use glium::texture::{
    srgb_texture2d::SrgbTexture2d, RawImage2d, Texture2d, UncompressedFloatFormat,
};
use glium::{implement_vertex, uniform, Display, Program, Surface, VertexBuffer};

use glam::{deg, Quat, Vec3};
use glium::backend::glutin::glutin::dpi::LogicalPosition;
use glium::backend::Facade;
use glium::framebuffer::SimpleFrameBuffer;
use glium::index::{IndicesSource, NoIndices};
use glium::texture::{MipmapsOption, SrgbFormat};
use glium::uniforms::{EmptyUniforms, MagnifySamplerFilter};
use glium::RawUniformValue::Vec2;
use image::math::utils::clamp;
use std::convert::identity;
use tracer::camera::Camera;

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
    let gl_window = display.gl_window();
    let window: &Window = gl_window.window();
    window.grab_cursor(true).unwrap();
    window.hide_cursor(true);
    window.set_cursor_position(LogicalPosition::new(
        display_size.width * 0.5,
        display_size.height * 0.5,
    )).unwrap();

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

    let instant_start = Instant::now();
    let mut instant_last_frame = Instant::now();

    let mut mouse_input: Vec3;
    let mut movement_input = Vec3::zero();

    let mut camera_angles = Vec3::zero();
    let mut camera_origin = Vec3::zero();

    let mut closed = false;
    while !closed {
        mouse_input = Vec3::zero();

        events_loop.poll_events(|ev| match ev {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => closed = true,
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta: mouse_delta } => {
                    let mouse_delta = Vec3::new(mouse_delta.0 as _, mouse_delta.1 as _, 0.0f32);
                    mouse_input += mouse_delta;
                }
                DeviceEvent::Key(KeyboardInput {
                    virtual_keycode: Some(key),
                    state,
                    ..
                }) => {
                    let sign = match state {
                        ElementState::Pressed => 1.0f32,
                        ElementState::Released => -1.0f32,
                    };

                    match key {
                        VirtualKeyCode::W => movement_input += sign * Vec3::unit_z(),
                        VirtualKeyCode::S => movement_input += -sign * Vec3::unit_z(),

                        VirtualKeyCode::A => movement_input += sign * Vec3::unit_x(),
                        VirtualKeyCode::D => movement_input += -sign * Vec3::unit_x(),

                        VirtualKeyCode::Q => movement_input += -sign * Vec3::unit_y(),
                        VirtualKeyCode::E => movement_input += sign * Vec3::unit_y(),

                        // System Input
                        VirtualKeyCode::Escape => closed = true,
                        _ => {}
                    }
                    movement_input = movement_input.min(Vec3::one()).max(-Vec3::one());
                }
                _ => {}
            },
            _ => {
                //println!("{:?}", ev);
            }
        });

        let mut can_accumulate = true;

        let instant_this_frame = Instant::now();
        let _frame_time = (instant_this_frame - instant_start).as_nanos() as f32 * 1e-9f32;
        let dt = (instant_this_frame - instant_last_frame).as_nanos() as f32 * 1e-9f32;
        instant_last_frame = instant_this_frame;

        let camera_angles_delta = Vec3::new(-10.0, 10.0, 0.0) * mouse_input * dt;
        camera_angles += camera_angles_delta;
        camera_angles.set_y(camera_angles.y().clamp(-85.0, 85.0));
        let camera_rotation = Quat::from_rotation_ypr(
            deg(camera_angles.x()),
            deg(camera_angles.y()),
            deg(camera_angles.z()),
        );

        let camera_movement_delta = 1.0 * movement_input * dt;
        camera_origin += camera_rotation * camera_movement_delta;

        if camera_angles_delta.length_squared() > 0.0
            || camera_movement_delta.length_squared() > 0.0
        {
            can_accumulate = false;
        }

        let camera = &Camera::new(
            camera_origin,
            camera_origin + camera_rotation * Vec3::unit_z(),
            Vec3::new(0.0, 1.0, 0.0),
            100.0,
            display_size.width as f32 / display_size.height as f32,
            0.025,
            1.0,
        );

        let instant_before_render = Instant::now();

        let render = SrgbTexture2d::with_format(
            &display,
            glium::texture::RawImage2d::from_raw_rgba(
                tracer::trace::image(camera, (resolution.0 as _, resolution.1 as _), 1, 50),
                resolution,
            ),
            SrgbFormat::U8U8U8U8,
            MipmapsOption::NoMipmap,
        )
        .unwrap();

        // composite render with accumulated image
        let mut composite_surface = SimpleFrameBuffer::new(&display, &composite).unwrap();
        composite_surface
            .draw(
                &geometry.0,
                &geometry.1,
                &composite_program,
                &uniform!(
                render_tex: &render,
                accumulation_tex: &accumulation,
                history_alpha: if can_accumulate { 0.9f32 } else { 0.0f32 }
                ),
                &Default::default(),
            )
            .unwrap();

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
