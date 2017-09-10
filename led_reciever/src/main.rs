extern crate lednet;
extern crate led_control;

extern crate tokio_core;
extern crate clap;

#[macro_use]
extern crate error_chain;

use std::thread::sleep;
use std::time::Duration;
use std::net::Ipv4Addr;

use led_control::set_pixels4;

use clap::{Arg, App};

use tokio_core::reactor::Core;

use lednet::LedServer;

mod errors {
    error_chain!{}
}

use errors::*;

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
    sleep(Duration::from_millis(500));

    loop {
        let msg = ledserver.load();
        set_pixels4(&mut serial, &msg).chain_err(
            || "Error setting pixels",
        )?;
        reactor.turn(Some(Duration::from_millis(5)));
    }
}
