extern crate vamp_host;
extern crate portaudio;
extern crate palette;
extern crate set_neopixels;

extern crate tokio_core;
extern crate futures;
extern crate multi_net;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate atomic_box;

use std::ffi::CString;
use std::thread::sleep;
use std::thread;
use std::time::Duration;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use palette::{Hsv, RgbHue, Lch, LabHue};
use palette::pixel::Srgb;
use palette::IntoColor;

use set_neopixels::{Pixel, Effect, AuxEffect, set_effect};

use futures::{Future, Stream, Sink};
use tokio_core::reactor::Core;
use multi_net::{Server, ControlPacket, AssembledDataPacket, RecievedPacket, ChannelID};

mod message;
use message::Message;

fn main() {
    let message_box = Arc::new(atomic_box::AtomicBox::new(Message {
        color: Pixel {
            red: 0,
            green: 0,
            blue: 0,
        },
        effect: Effect::Constant,
        aux_effect: AuxEffect::None,
        aux_color: None,
    }));
    let thread_message_box = message_box.clone();
    thread::spawn(move || {
        let mut serial = set_neopixels::setup("/dev/ttyACM0");
        sleep(Duration::from_secs(1));
        loop {
            let Message {
                color,
                effect,
                aux_color,
                aux_effect,
            } = (*thread_message_box.load()).clone();
            set_effect(&mut serial, color, effect, aux_color, aux_effect).unwrap();
            sleep(Duration::from_millis(20));
        }
    });

    let mut reactor = Core::new().unwrap();

    let (s, handle) = Server::new(reactor.handle(), &[0, 0, 0, 0u8].into()).unwrap();

    reactor.handle().spawn(s.map_err(
        |e| panic!("Error polling server: {:?}", e),
    ));

    reactor
        .run(handle.for_each(|p| {
            println!("Recieved: {:?}", p);
            println!("\n\n\n\n\n\n");
            match p {
                RecievedPacket::Data(data) => {
                    match data.channel {
                        ChannelID(42) => {
                            match bincode::deserialize(&data.data) {
                                Ok(msg) => {
                                    let msg: Message = msg;
                                    println!("Message: {:?}", msg);
                                    message_box.store(msg)
                                }
                                Err(_) => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            Ok(())
        }))
        .unwrap();
}
