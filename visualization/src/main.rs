#[macro_use]
extern crate gfx;

extern crate gfx_window_glutin;
extern crate glutin;
extern crate rand;
extern crate image;

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

use glutin::{ElementState, MouseButton};

use tokio_core::reactor::Core;
use crossbeam::Scope;

use led_control::Pixel;
use palette::pixel::RgbPixel;
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
        out: gfx::RenderTarget<ColorFormat> = "Target0",
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
    let sampler = factory.create_sampler_linear();

    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        out: main_color,
    };

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
                Resized(w, h) => {
                    gfx_window_glutin::update_views(&window, &mut data.out, &mut main_depth);
                }
                e => println!("{:?}", e), //{} ,
            }
        });

        encoder.clear(&data.out, BLACK);
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
        for (i, pixel) in self.pixels.iter().enumerate() {
            let (x, y) = (2.0 * (i as f32 / len as f32) - 1.0, 0.0);
            let i = i as u16;

            let (hx, hy) = (1.0 / len as f32, 1.0);
            let color = pixel.to_rgba();
            let color = [color.0, color.1, color.2];
            vs.extend(
                &[
                    Vertex {
                        pos: [x + hx, y - hy, 0.0],
                        color,
                    },
                    Vertex {
                        pos: [x - hx, y - hy, 0.0],
                        color,
                    },
                    Vertex {
                        pos: [x - hx, y + hy, 0.0],
                        color,
                    },
                    Vertex {
                        pos: [x + hx, y + hy, 0.0],
                        color,
                    },
                ],
            );
            let o = 4 * i;
            is.extend(&[o, o + 1, o + 2, o + 2, o + 3, o]);
        }
        (vs, is)
    }
}
