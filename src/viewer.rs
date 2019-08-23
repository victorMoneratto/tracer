use glium::backend::glutin::glutin::dpi::LogicalSize;
use glium::glutin::WindowBuilder;
use glium::backend::glutin::glutin::{ContextBuilder, EventsLoop, Event};
use glium::{Display, Surface, implement_vertex};

#[derive(Clone, Copy)]
struct Vertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}

fn main() {
    let mut events_loop = EventsLoop::new();
    let wb = WindowBuilder::new()
        .with_dimensions(LogicalSize::new(600.0, 600.0))
        .with_resizable(false)
        .with_title("Tracer");
    let cb = ContextBuilder::new();
    let display = Display::new(wb, cb, &events_loop).unwrap();

    implement_vertex!(Vertex, pos, uv);
    let vertices = vec![
        Vertex{ pos: [1.0, 1.0], uv: [1.0, 1.0]},
        Vertex{ pos: [1.0, -1.0], uv: [1.0, 0.0]},
        Vertex{ pos: [-1.0, 1.0], uv: [0.0, 1.0]},
        Vertex{ pos: [-1.0, -1.0], uv: [0.0, 0.0]},
    ];
    let vbo = glium::VertexBuffer::new(&display, &vertices).unwrap();
    let ibo = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

    let vertex_shader_src = r#"
        #version 150

        in vec2 pos;
        in vec2 uv;

        out v2f {
            vec2 uv;
        } o;

        void main() {
            o.uv = uv;
            gl_Position = vec4(pos, 0.0, 1.0);
        }
    "#;

    let frag_shader_src = r#"
        #version 150
        in v2f { vec2 uv; } i;
        out vec4 color;

        void main() {
            color = vec4(i.uv, 0.0, 1.0);
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, frag_shader_src, None).unwrap();


    let mut closed = false;
    while !closed {
        events_loop.poll_events(|ev| {
            match ev {
                Event::WindowEvent { event, ..} => match event {
                    glium::glutin::WindowEvent::CloseRequested => closed = true,
                    _ => (),
                }
                _ => (),
            }
        });

        let mut rt = display.draw();
        rt.clear_color(0.0, 0.0, 0.0, 1.0);
        rt.draw(&vbo, &ibo, &program, &glium::uniforms::EmptyUniforms,
                &Default::default()).unwrap();

        rt.finish().unwrap();

    }
}