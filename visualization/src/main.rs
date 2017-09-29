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

use nalgebra::{Matrix4, Vector3, Point3};

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
    }

    let mut proj_mode = Projection::Flat;

    while running.load(Ordering::Relaxed) {
        led_strip.update(led_server.load());
        let (vs, is) = led_strip.get_verticies_indicies();
        let (vbuf, sl) = factory.create_vertex_buffer_with_slice(&vs, &*is);

        data.vbuf = vbuf;
        slice = sl;

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
                        Projection::Perspective => Projection::Flat,
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
                data.transform = (proj_mat * look_mat * rot_mat).into();
            }
            Projection::Flat => data.transform = Matrix4::identity().into(),
        }

        encoder.clear(&data.out, BLACK);
        encoder.clear_depth(&data.out_depth, 1000.0);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup()
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum Cursor {
    Plain((f32, f32), [f32; 3]),
    Growing((f32, f32), f32, [f32; 3]),
}

impl Cursor {
    fn to_square(self) -> Square {
        use Cursor::*;
        match self {
            Plain(xy, color) => Square {
                pos: xy,
                size: 0.05,
                color,
            },
            Growing(xy, size, color) => Square {
                pos: xy,
                size,
                color,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Square {
    pub pos: (f32, f32),
    pub size: f32,
    pub color: [f32; 3],
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
            vs.extend(
                &[
                    Vertex {
                        pos: [x + hx, y - hy, hsv1.value],
                        color: colorR,
                    },
                    Vertex {
                        pos: [x - hx, y - hy, hsv2.value],
                        color: colorL,
                    },
                    Vertex {
                        pos: [x - hx, y + hy, hsv2.value],
                        color: colorL,
                    },
                    Vertex {
                        pos: [x + hx, y + hy, hsv1.value],
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
