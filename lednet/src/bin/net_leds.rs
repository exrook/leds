extern crate lednet;
extern crate vamp;
extern crate portaudio;
extern crate palette;
extern crate led_control;

extern crate tokio_core;
extern crate futures;
extern crate multinet;

extern crate clap;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate atomic_box;

#[macro_use]
extern crate error_chain;

use std::ffi::CString;
use std::thread::sleep;
use std::thread;
use std::time::Duration;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::net::Ipv4Addr;

use palette::{Hsv, RgbHue, Lch, LabHue};
use palette::pixel::Srgb;
use palette::IntoColor;

use led_control::{Pixel, Effect, AuxEffect, set_pixels4};

use clap::{Arg, App};

use futures::{Future, Stream, Sink};
use tokio_core::reactor::Core;
use multinet::{Server, ControlPacket, AssembledDataPacket, RecievedPacket, ChannelID};

use lednet::{LedServer, Message};

mod errors {
    error_chain!{}
}

use errors::*;

const NUM_LEDS: usize = 427;

quick_main!(run);

fn run() -> Result<()> {
    let matches = App::new("LED Server")
        .arg(
            Arg::with_name("address")
                .short("a")
                .takes_value(true)
                .multiple(true)
                .help("Joins the multicast group on the specified interfaces")
                .default_value("0.0.0.0"),
        )
        .get_matches();
    let addrs: Vec<Ipv4Addr> = matches
        .values_of_lossy("address")
        .unwrap()
        .into_iter()
        .map(|s| s.parse())
        .collect::<std::result::Result<_, _>>()
        .chain_err(|| "Invalid IP address specified")?;
    let mut reactor = Core::new().unwrap();
    let ledserver = LedServer::new(&reactor.handle(), addrs).chain_err(
        || "Error starting led server",
    )?;
    let mut serial = led_control::setup("/dev/ttyACM0");
    sleep(Duration::from_secs(1));
    loop {
        let msg = ledserver.load();
        //let Message {
        //    color,
        //    effect,
        //    aux_color,
        //    aux_effect,
        //} = (*thread_message_box.load()).clone();
        //set_effect_compat(&mut serial, color, effect, aux_color, aux_effect, 427).unwrap();
        set_pixels4(&mut serial, &msg).unwrap();
        reactor.turn(Some(Duration::from_millis(5)));
        //sleep(Duration::from_millis(20));
    }

    Ok(())
}
