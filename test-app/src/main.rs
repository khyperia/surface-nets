#[macro_use]
extern crate gfx;
extern crate cgmath;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate noise;
extern crate surface_nets;

mod camera;

use cgmath::{Deg, Matrix, Matrix4, SquareMatrix};
use gfx::state::{RasterMethod, Rasterizer};
use gfx::traits::FactoryExt;
use gfx::{Device, Primitive};
use glutin::{ElementState, Event, KeyboardInput, WindowEvent};

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 3] = "a_Pos",
        norm: [f32; 3] = "a_Norm",
    }

    constant Locals {
        projection: [[f32; 4]; 4] = "u_Projection",
        modelview: [[f32; 4]; 4] = "u_ModelView",
        normalmat: [[f32; 4]; 4] = "u_NormalMat",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        projection: gfx::Global<[[f32; 4]; 4]> = "u_Projection",
        modelview: gfx::Global<[[f32; 4]; 4]> = "u_ModelView",
        normalmat: gfx::Global<[[f32; 4]; 4]> = "u_NormalMat",
        locals: gfx::ConstantBuffer<Locals> = "Locals",
        out_color: gfx::RenderTarget<ColorFormat> = "Target0",
        out_depth: gfx::DepthTarget<DepthFormat> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

impl Vertex {
    fn new(p: [f32; 3], n: [f32; 3]) -> Vertex {
        Vertex {
            pos: [p[0], p[1], p[2]],
            norm: [n[0], n[1], n[2]],
        }
    }
}

// wikipedia to the rescue!
// https://en.wikipedia.org/wiki/Blinn%E2%80%93Phong_shading_model
static VERTEX_SRC: &'static [u8] = b"
#version 140

in vec3 a_Pos;
in vec3 a_Norm;

uniform Locals {
    mat4 u_Projection;
    mat4 u_ModelView;
    mat4 u_NormalMat;
};

out vec3 vertPos;
out vec3 normalInterp;

void main() {
    gl_Position = u_Projection * u_ModelView * vec4(a_Pos, 1.0);
    vec4 vertPos4 = u_ModelView * vec4(a_Pos, 1.0);
    vertPos = vec3(vertPos4) / vertPos4.w;
    normalInterp = vec3(u_NormalMat * vec4(a_Norm, 0.0));
}
";

static FRAGMENT_SRC: &'static [u8] = b"
#version 140

in vec3 vertPos;
in vec3 normalInterp;

const vec3 lightPos = vec3(1.0,1.0,1.0);
const vec3 ambientColor = vec3(0.1, 0.15, 0.2);
const vec3 diffuseColor = vec3(0.4, 0.3, 0.5);
const vec3 specColor = vec3(1.0, 0.8, 0.8);
const float shininess = 16.0;

void main() {
    vec3 normal = normalize(normalInterp);
    vec3 lightDir = normalize(lightPos - vertPos);

    float lambertian = max(dot(lightDir, normal), 0.0);
    float specular = 0.0;

    if (lambertian > 0.0) {
        vec3 viewDir = normalize(-vertPos);
        vec3 halfDir = normalize(lightDir + viewDir);
        float specAngle = max(dot(halfDir, normal), 0.0);
        specular = pow(specAngle, shininess);
    }
    vec3 colorLinear = ambientColor + diffuseColor * lambertian + specColor * specular;
    gl_FragColor = vec4(colorLinear, 1.0);
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

    let mut keyboard_input = camera::CameraControl::new();

    let (mut pso, mut pso_alt) = {
        let program = factory.link_program(VERTEX_SRC, FRAGMENT_SRC).unwrap();
        let rasterizer = Rasterizer::new_fill().with_cull_back();
        let pso = factory
            .create_pipeline_from_program(
                &program,
                Primitive::TriangleList,
                rasterizer,
                pipe::new(),
            ).unwrap();
        let mut rasterizer_alt = Rasterizer::new_fill().with_cull_back();
        rasterizer_alt.method = RasterMethod::Line(1);
        let pso_alt = factory
            .create_pipeline_from_program(
                &program,
                Primitive::TriangleList,
                rasterizer_alt,
                pipe::new(),
            ).unwrap();
        (pso, pso_alt)
    };
    let (verts, inds) = get_data();
    let (vertex_buffer, slice) =
        factory.create_vertex_buffer_with_slice(&verts as &[_], &inds as &[_]);
    let mut projection = cgmath::perspective(Deg(45.0f32), 1.0, 1.0, 1000.0);
    let mut data = pipe::Data {
        locals: factory.create_constant_buffer(1),
        projection: projection.into(),
        modelview: Matrix4::identity().into(),
        normalmat: Matrix4::identity().into(),
        vbuf: vertex_buffer,
        out_color: main_color,
        out_depth: main_depth,
    };

    loop {
        let mut running = true;
        events_loop.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => running = false,
                    WindowEvent::Resized(size) => {
                        projection = cgmath::perspective(
                            Deg(45.0f32),
                            (size.width / size.height) as f32,
                            1.0,
                            1000.0,
                        );
                        window.resize(size.to_physical(window.get_hidpi_factor()));
                        gfx_window_glutin::update_views(
                            &window,
                            &mut data.out_color,
                            &mut data.out_depth,
                        );
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(virtual_keycode),
                                state,
                                ..
                            },
                        ..
                    } => match state {
                        ElementState::Pressed => {
                            keyboard_input.key_down(virtual_keycode);
                            if virtual_keycode == glutin::VirtualKeyCode::F {
                                std::mem::swap(&mut pso, &mut pso_alt);
                            }
                        }
                        ElementState::Released => keyboard_input.key_up(virtual_keycode),
                    },
                    //x => println!("{:?}", x),
                    _ => (),
                }
            }
        });

        if !running {
            break;
        }

        keyboard_input.step();
        let view_matrix =
            Matrix4::look_at_dir(keyboard_input.pos, keyboard_input.look, keyboard_input.up);

        // draw a frame
        let locals = Locals {
            projection: projection.into(),
            modelview: view_matrix.into(),
            normalmat: view_matrix.invert().unwrap().transpose().into(),
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

fn get_data() -> (Vec<Vertex>, Vec<u32>) {
    let simplex = noise::OpenSimplex::new();
    use noise::NoiseFn;
    let grid_size = 200;
    let sdf = move |x, y, z| {
        let x = x as f64 / 5.0;
        let y = y as f64 / 5.0;
        let z = z as f64 / 5.0;
        //let bias_source = (y - 5.0) / 10.0;
        //let bias = bias_source;
        simplex.get([x, y, z]) as f32
        //+ bias as f32
        //
        //let x = x as f32 - grid_size as f32 / 2.0;
        //let y = y as f32 - grid_size as f32 / 2.0;
        //let z = z as f32 - grid_size as f32 / 2.0;
        //(x * x + y * y + z * z).sqrt() - grid_size as f32 / 2.1
        //
        //let sum = x + y + z;
        //if sum & 1 == 0 {
        //    -1.0
        //} else {
        //    1.0
        //}
        //-(x as f32) + y as f32 / 2.0 + z as f32 / 3.0 + 5.5
        //let x = x as f32 - grid_size as f32 / 2.0;
        //let y = y as f32 - grid_size as f32 / 2.0;
        //let z = z as f32 - grid_size as f32 / 2.0;
        //x * x + y * y - z
    };
    let (verts, normals, indicies) = surface_nets::surface_net(grid_size, &sdf, true);
    println!(
        "{} verts, {} inds ({} triangles)",
        verts.len(),
        indicies.len(),
        indicies.len() / 3
    );
    (
        verts
            .into_iter()
            .zip(normals.into_iter())
            .map(|(pos, norm)| Vertex::new(pos, norm))
            .collect(),
        indicies.into_iter().map(|i| i as u32).collect(),
    )
}
