#[macro_use]
extern crate gfx;

extern crate gfx_window_glutin;
extern crate glutin;
extern crate rand;
extern crate image;
extern crate nalgebra;

extern crate tokio_core;
#[macro_use]
extern crate error_chain;
extern crate crossbeam;

extern crate led_control;
extern crate palette;
extern crate lednet;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::time::Duration;

use gfx::traits::FactoryExt;
use gfx::Device;
use gfx::state::{Depth, Comparison};

use glutin::{ElementState, MouseButton};

use nalgebra::{Matrix4, Matrix3, Vector3, Point3, Unit};

use tokio_core::reactor::Core;
use crossbeam::Scope;

use led_control::Pixel;
use palette::pixel::RgbPixel;
use palette::{Rgb, Hsv};
use lednet::LedServer;

mod errors {
    error_chain!{}
}
use errors::*;

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

gfx_defines! {
    vertex Vertex {
        pos: [f32; 3] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::Global<[[f32; 4]; 4]> = "u_Proj",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
        out_depth: gfx::DepthTarget<DepthFormat> = Depth {
            fun: Comparison::LessEqual,
            write: true
        },
    }
}

const WHITE: [f32; 3] = [1.0, 1.0, 1.0];

quick_main!(|| -> Result<()> { crossbeam::scope(run) });

fn run(scope: &Scope) -> Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let (chan, led_server) = channel();
    let t_running = running.clone();
    scope.spawn(move || {
        let running = t_running;
        let mut reactor = match || -> Result<(Core, LedServer)> {
            let r = Core::new().chain_err(|| "Unable to start tokio core")?;
            let l = LedServer::new(&r.handle(), [[0, 0, 0, 0].into()])
                .chain_err(|| "Unable to create led_server")?;
            Ok((r, l))
        }() {
            Ok((r, led)) => {
                chan.send(Ok(led));
                r
            }
            Err(e) => {
                chan.send(Err(e)).unwrap();
                panic!()
            }
        };
        while running.load(Ordering::Relaxed) {
            reactor.turn(Some(Duration::from_millis(100)));
        }
    });
    let mut led_server = led_server
        .recv()
        .chain_err(|| "Error recieving led_server")?
        .chain_err(|| "Error starting led_server")?;
    let mut led_strip = LedStrip::new();
    let mut frame = Frame::new();
    let events_loop = glutin::EventsLoop::new();
    let builder = glutin::WindowBuilder::new()
        .with_title("LED Visualization")
        .with_dimensions(800, 800)
        .with_vsync();
    let (window, mut device, mut factory, main_color, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder, &events_loop);

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let pso = factory
        .create_pipeline_simple(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/shaders/rect_150.glslv"
            )),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/shaders/rect_150.glslf"
            )),
            pipe::new(),
        )
        .unwrap();
    let (verticies, indicies) = led_strip.get_verticies_indicies();
    let (vertex_buffer, mut slice) =
        factory.create_vertex_buffer_with_slice(&verticies, &*indicies);

    let proj_mat = Matrix4::new_perspective(1.0, 70.0, 0.1, 100.0);
    let ortho_mat = Matrix4::new_orthographic(-1.0, 1.0, -1.0, 1.0, 0.1, 100.0);
    let look_mat = Matrix4::look_at_rh(
        &Point3::new(0.0, 0.0, 3.0),
        &Point3::new(0.0, 0.0, 0.0),
        &Vector3::new(0.0, 1.0, 0.0),
    );
    let mut pitch = 0.0;
    let mut yaw = 0.0;
    let rot_mat = Matrix4::from_euler_angles(0.0, 0.0, 0.0);

    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        transform: (proj_mat * look_mat).into(),
        out: main_color,
        out_depth: main_depth,
    };

    let mut mouse_pressed = false;
    let mut start_mouse = (0, 0);
    let mut look_offset = (0.0, 0.0);

    #[derive(Clone, Copy)]
    enum Projection {
        Flat,
        Perspective,
        Orthographic,
    }

    let mut proj_mode = Projection::Flat;

    while running.load(Ordering::Relaxed) {
        events_loop.poll_events(|glutin::Event::WindowEvent {
             window_id: _,
             event,
         }| {
            use glutin::WindowEvent::*;
            match event {
                KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape), _) |
                Closed => running.store(false, Ordering::Release),
                KeyboardInput(ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Space), _) => {
                    proj_mode = match proj_mode {
                        Projection::Flat => Projection::Perspective,
                        Projection::Perspective => Projection::Orthographic,
                        Projection::Orthographic => Projection::Flat,
                    };
                }
                Resized(w, h) => {
                    gfx_window_glutin::update_views(&window, &mut data.out, &mut data.out_depth);
                }
                MouseInput(ElementState::Pressed, MouseButton::Right) => mouse_pressed = true,
                MouseInput(ElementState::Released, MouseButton::Right) => {
                    mouse_pressed = false;
                    pitch = pitch + look_offset.1;
                    yaw = yaw + look_offset.0;
                    look_offset = (0.0, 0.0)
                }
                MouseMoved(x, y) => {
                    if mouse_pressed {
                        look_offset = (
                            (x - start_mouse.0) as f32 * 0.005,
                            (y - start_mouse.1) as f32 * 0.005,
                        );
                    } else {
                        start_mouse = (x, y)
                    }
                }
                e => println!("{:?}", e), //{} ,
            }
        });
        let rot_mat = Matrix4::from_euler_angles(pitch + look_offset.1, yaw + look_offset.0, 0.0);
        match proj_mode {
            Projection::Perspective => {
                yaw += 0.01;
                data.transform = (proj_mat * look_mat * rot_mat).into();
            }
            Projection::Flat => data.transform = Matrix4::identity().into(),
            Projection::Orthographic => data.transform = (ortho_mat * look_mat * rot_mat).into(),
        }

        led_strip.update(led_server.load());

        encoder.clear(&data.out, BLACK);
        encoder.clear_depth(&data.out_depth, 1000.0);

        let (vs, is) = led_strip.get_verticies_indicies();
        let (vbuf, sl) = factory.create_vertex_buffer_with_slice(&vs, &*is);
        data.vbuf = vbuf;
        slice = sl;
        encoder.draw(&slice, &pso, &data);

        let (vs, is) = frame.get_verticies_indicies();
        let (vbuf, sl) = factory.create_vertex_buffer_with_slice(&vs, &*is);
        data.vbuf = vbuf;
        slice = sl;
        encoder.draw(&slice, &pso, &data);

        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup()
    }
    Ok(())
}

#[derive(Debug)]
struct LedStrip {
    pixels: Arc<Vec<Pixel>>,
    ratio: f32,
}

impl LedStrip {
    pub fn new() -> Self {
        Self {
            pixels: vec![].into(),
            ratio: 1.0,
        }
    }

    pub fn update(&mut self, pixels: Arc<Vec<Pixel>>) {
        self.pixels = pixels;
    }

    pub fn get_verticies_indicies(&self) -> (Vec<Vertex>, Vec<u16>) {
        let (mut vs, mut is) = (vec![], vec![]);
        let len = self.pixels.len();
        for (i, pxs) in self.pixels.windows(2).enumerate() {
            let p1 = pxs[0];
            let p2 = pxs[1];
            let (x, y) = (2.0 * (i as f32 / len as f32) - 1.0, 0.0);
            let i = i as u16;

            let (hx, hy) = (1.0 / len as f32, 1.0);
            let hsv1: Hsv<f32> = Rgb::from_pixel(&p1).into();
            let colorL = p1.to_rgba();
            let colorL = [colorL.0, colorL.1, colorL.2];
            let hsv2: Hsv<f32> = Rgb::from_pixel(&p2).into();
            let colorR = p2.to_rgba();
            let colorR = [colorR.0, colorR.1, colorR.2];
            //let colorR = [0.0, 0.0, 0.0];
            vs.extend(
                &[
                    Vertex {
                        pos: [x + hx, y - hy, hsv2.value],
                        color: colorR,
                    },
                    Vertex {
                        pos: [x - hx, y - hy, hsv1.value],
                        color: colorL,
                    },
                    Vertex {
                        pos: [x - hx, y + hy, hsv1.value],
                        color: colorL,
                    },
                    Vertex {
                        pos: [x + hx, y + hy, hsv2.value],
                        color: colorR,
                    },
                    Vertex {
                        pos: [x + hx, y - hy, 0.0],
                        color: colorR,
                    },
                    Vertex {
                        pos: [x - hx, y - hy, 0.0],
                        color: colorL,
                    },
                    Vertex {
                        pos: [x - hx, y + hy, 0.0],
                        color: colorL,
                    },
                    Vertex {
                        pos: [x + hx, y + hy, 0.0],
                        color: colorR,
                    },
                ],
            );
            let o = 8 * i;
            is.extend(&[o + 0, o + 1, o + 2, o + 2, o + 3, o + 0]);
            is.extend(&[o + 4, o + 0, o + 3, o + 3, o + 7, o + 4]);
            is.extend(&[o + 5, o + 1, o + 0, o + 0, o + 4, o + 5]);
            is.extend(&[o + 6, o + 2, o + 1, o + 1, o + 5, o + 6]);
            is.extend(&[o + 7, o + 3, o + 2, o + 2, o + 6, o + 7]);
            is.extend(&[o + 6, o + 5, o + 4, o + 4, o + 7, o + 6]);
        }
        (vs, is)
    }
}

struct Frame {
    pos: [f32; 3],
    width: f32,
    height: f32,
    depth: f32,
    linewidth: f32,
    color: [f32; 3],
}
impl Frame {
    pub fn new() -> Self {
        Self {
            pos: [0.0, 0.0, 0.0],
            width: 1.0,
            height: 1.0,
            depth: 1.0,
            linewidth: 0.05,
            color: [1.0, 1.0, 1.0],
        }
    }
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn generate_line(
        point1: [f32; 3],
        point2: [f32; 3],
        color: [f32; 3],
        linewidth: f32,
        o: u16,
    ) -> (Vec<Vertex>, Vec<u16>) {
        let trans: Matrix3<f32> = [[0.0,-1.0,0.0],[0.0,0.0,-1.0],[1.0,0.0,0.0]].into();
        let p1: Vector3<f32> = point1.into();
        let p2: Vector3<f32> = point2.into();
        let dir = p2 - p1;
        let mut normal_1 = trans*dir;
        let normal_2 = dir.cross(&normal_1);
        let normal_1 = Unit::new_normalize(normal_1).unwrap()*linewidth;
        let normal_2 = Unit::new_normalize(normal_2).unwrap()*linewidth;

        let vert = vec![
            Vertex {
                pos: (p1 + normal_1 - normal_2).into(),
                color
            },
            Vertex {
                pos: (p1 - normal_1 - normal_2).into(),
                color
            },
            Vertex {
                pos: (p1 - normal_1 + normal_2).into(),
                color
            },
            Vertex {
                pos: (p1 + normal_1 + normal_2).into(),
                color
            },
            Vertex {
                pos: (p2 + normal_1 - normal_2).into(),
                color
            },
            Vertex {
                pos: (p2 - normal_1 - normal_2).into(),
                color
            },
            Vertex {
                pos: (p2 - normal_1 + normal_2).into(),
                color
            },
            Vertex {
                pos: (p2 + normal_1 + normal_2).into(),
                color
            },
        ];
        let o = o * 8;
        let is = vec![
            o + 0, o + 1, o + 2, o + 2, o + 3, o + 0,
            o + 4, o + 0, o + 3, o + 3, o + 7, o + 4,
            o + 5, o + 1, o + 0, o + 0, o + 4, o + 5,
            o + 6, o + 2, o + 1, o + 1, o + 5, o + 6,
            o + 7, o + 3, o + 2, o + 2, o + 6, o + 7,
            o + 6, o + 5, o + 4, o + 4, o + 7, o + 6,
        ];

        (vert, is)
    }
    pub fn get_verticies_indicies(&self) -> (Vec<Vertex>, Vec<u16>) {
        let (mut vert, mut is) = (vec![], vec![]);

        let (v, i) =
            Self::generate_line([1.0, 1.0, 0.0], [-1.0, 1.0, 0.0], [1.0, 1.0, 1.0], 0.001, 0);
        vert.extend(v);
        is.extend(i);
        let (v, i) = Self::generate_line(
            [-1.0, 1.0, 0.0],
            [-1.0, -1.0, 0.0],
            [1.0, 1.0, 1.0],
            0.001,
            1,
        );
        vert.extend(v);
        is.extend(i);
        let (v, i) = Self::generate_line(
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 1.0],
            0.001,
            2,
        );
        vert.extend(v);
        is.extend(i);
        let (v, i) =
            Self::generate_line([1.0, -1.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0], 0.001, 3);
        vert.extend(v);
        is.extend(i);

        let (v, i) =
            Self::generate_line([1.0, 1.0, 1.0], [-1.0, 1.0, 1.0], [1.0, 1.0, 1.0], 0.001, 4);
        vert.extend(v);
        is.extend(i);
        let (v, i) = Self::generate_line(
            [-1.0, 1.0, 1.0],
            [-1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            0.001,
            5,
        );
        vert.extend(v);
        is.extend(i);
        let (v, i) = Self::generate_line(
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            0.001,
            6,
        );
        vert.extend(v);
        is.extend(i);
        let (v, i) =
            Self::generate_line([1.0, -1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 1.0], 0.001, 7);
        vert.extend(v);
        is.extend(i);
        //let (v, i) = Self::generate_line([1.0, 1.0, 0.0], [1.0, 1.0, 1.0], [1.0, 1.0, 1.0], 0.1, 2);
        //vert.extend(v);
        //is.extend(i);
        (vert, is)
    }
}
