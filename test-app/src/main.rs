#[macro_use]
extern crate gfx;
extern crate cgmath;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate noise;
extern crate surface_nets;

use cgmath::{Deg, Matrix4, Point3, Vector3};
use gfx::state::{Rasterizer, RasterMethod};
use gfx::traits::FactoryExt;
use gfx::{Device, Primitive};
use glutin::{Event, WindowEvent};
use std::time::Instant;
//use std::error::Error;
//use std::fs::File;
//use std::io::Write;


pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 4] = "a_Pos",
    }

    constant Locals {
        transform: [[f32; 4]; 4] = "u_Transform",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::Global<[[f32; 4]; 4]> = "u_Transform",
        locals: gfx::ConstantBuffer<Locals> = "Locals",
        out_color: gfx::RenderTarget<ColorFormat> = "Target0",
        out_depth: gfx::DepthTarget<DepthFormat> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

impl Vertex {
    fn new(p: [f32; 3]) -> Vertex {
        Vertex {
            pos: [p[0], p[1], p[2], 1.0],
        }
    }
}

struct Timer {
    start: Instant,
}

impl Timer {
    fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    fn elapsed(&self) -> f64 {
        let now = Instant::now();
        let duration = now - self.start;
        duration.as_secs() as f64 + f64::from(duration.subsec_nanos()) / 1_000_000_000.0
    }
}

static VERTEX_SRC: &'static [u8] = b"
#version 140

in vec4 a_Pos;

uniform Locals {
    mat4 u_Transform;
};

out vec4 v_Pos;

void main() {
    v_Pos = a_Pos;
    gl_Position = u_Transform * v_Pos;
}
";

static FRAGMENT_SRC: &'static [u8] = b"
#version 140

in vec4 v_Pos;

void main() {
    float r = sin(v_Pos.x / 5.0) * 0.25 + 0.5;
    float g = sin(v_Pos.y / 5.0) * 0.25 + 0.5;
    float b = sin(v_Pos.z / 5.0) * 0.25 + 0.5;
    gl_FragColor = vec4(r, g, b, 1.0);
}
";

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window_config = glutin::WindowBuilder::new().with_title("Triangle example".to_string());
    //.with_dimensions(1024, 768);

    let context = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 6)))
        .with_vsync(true);
    let (window, mut device, mut factory, main_color, main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(window_config, context, &events_loop);
    let mut encoder = gfx::Encoder::from(factory.create_command_buffer());

    let pso = {
        let program = factory.link_program(VERTEX_SRC, FRAGMENT_SRC).unwrap();
        let mut rasterizer = Rasterizer::new_fill().with_cull_back();
        //rasterizer.method = RasterMethod::Line(1);
        factory
            .create_pipeline_from_program(
                &program,
                Primitive::TriangleList,
                rasterizer,
                pipe::new(),
            ).unwrap()
    };
    let (verts, inds) = get_data();
    let (vertex_buffer, slice) =
        factory.create_vertex_buffer_with_slice(&verts as &[_], &inds as &[_]);
    let mut perspective = cgmath::perspective(Deg(45.0f32), 1.0, 1.0, 1000.0);
    let mut data = pipe::Data {
        locals: factory.create_constant_buffer(1),
        transform: (perspective * default_view(0.0)).into(),
        vbuf: vertex_buffer,
        out_color: main_color,
        out_depth: main_depth,
    };

    let timer = Timer::new();
    loop {
        let mut running = true;
        events_loop.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => running = false,
                    WindowEvent::Resized(size) => {
                        perspective = cgmath::perspective(Deg(45.0f32), (size.width / size.height) as f32, 1.0, 1000.0);
                        window.resize(size.to_physical(window.get_hidpi_factor()));
                        gfx_window_glutin::update_views(
                            &window,
                            &mut data.out_color,
                            &mut data.out_depth,
                        );
                    }
                    //x => println!("{:?}", x),
                    _ => (),
                }
            }
        });

        if !running {
            break;
        }

        // draw a frame
        let locals = Locals {
            transform: (perspective * default_view(timer.elapsed() as f32)).into(),
        };
        encoder.update_constant_buffer(&data.locals, &locals);
        encoder.clear(&data.out_color, [0.5, 0.5, 0.5, 1.0]);
        encoder.clear_depth(&data.out_depth, 1.0);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}

fn default_view(time: f32) -> Matrix4<f32> {
    let distance = 300.0;
    let rotation_rate = 0.1;
    let center = 100.0;
    let x = (time * rotation_rate).cos() * distance + center;
    let y = (time * rotation_rate).sin() * distance + center;
    let z = center + distance / 2.0;
    Matrix4::look_at(
        Point3::new(x, y, z),
        Point3::new(center, center, center),
        Vector3::unit_z(),
    )
}

fn get_data() -> (Vec<Vertex>, Vec<u32>) {
    let simplex = noise::OpenSimplex::new();
    use noise::NoiseFn;
    let (verts, indicies) = surface_nets::surface_net(
        200,
        &move |x, y, z| {
            let x = x as f64 / 5.0;
            let y = y as f64 / 5.0;
            let z = z as f64 / 5.0;
            //let bias_source = (y - 5.0) / 10.0;
            //let bias = bias_source;
            simplex.get([x,y,z]) as f32 //+ bias as f32
            //
            // let x = x as f32 - 5.0;
            // let y = y as f32 - 5.0;
            // let z = z as f32 - 5.0;
            // (x * x + y * y + z * z).sqrt() - 5.0
            //
            //let sum = x + y + z;
            //match sum & 1 == 0 {
            //    false => -1.0,
            //    true => 1.0,
            //}
            //x as f32 - 5.5
        },
        true,
    );
    println!("{} verts, {} inds ({} triangles)", verts.len(), indicies.len(), indicies.len() / 3);
    (
        verts
            .into_iter()
            .map(|(x, y, z)| Vertex::new([x, y, z]))
            .collect(),
        indicies.into_iter().map(|i| i as u32).collect(),
    )
}
