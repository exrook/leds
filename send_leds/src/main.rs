extern crate led_control;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate lednet;
extern crate tokio_core;

use std::time::Duration;
use std::thread::sleep;
use std::net::Ipv4Addr;

use clap::{App, Arg};

use led_control::{Pixel, OldEffect, OldAuxEffect, set_effect_compat, set_pixels4, setup,
                  gen_effect};

use lednet::LedServer;

use tokio_core::reactor::{Core, Timeout};

mod errors {
    error_chain!{}
}

use errors::*;

quick_main!(run);

fn run() -> Result<()> {
    let matches = App::new("set_neopixels")
        .arg(
            Arg::with_name("color")
                .number_of_values(3)
                .required(true)
                .takes_value(true)
                .index(1),
        )
        .arg(
            Arg::with_name("aux-color")
                .short("c")
                .number_of_values(3)
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("effect")
                .short("e")
                .takes_value(true)
                .required(false)
                .possible_values(
                    &[
                        "Constant",
                        "Flash",
                        "SetPix",
                        "Width",
                        "DoubleWidth",
                        "QuadWidth",
                        "Edges",
                    ],
                ),
        )
        .arg(
            Arg::with_name("aux-effect")
                .short("a")
                .takes_value(true)
                .required(false)
                .possible_values(
                    &[
                        "None",
                        "Offset",
                        "FillLeft",
                        "FillCenter",
                        "FillRight",
                        "FillEdges",
                        "FillDouble",
                    ],
                ),
        )
        .arg(
            Arg::with_name("param")
                .short("p")
                .takes_value(true)
                .required(false)
                .requires("effect"),
        )
        .arg(
            Arg::with_name("aux-param")
                .short("q")
                .takes_value(true)
                .required(false)
                .requires("aux-effect"),
        )
        .arg(
            Arg::with_name("address")
                .long("addr")
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
    let color = values_t!(matches, "color", u8)
        .map(|x| {
            Pixel {
                red: x[0],
                green: x[1],
                blue: x[2],
            }
        })
        .unwrap_or_else(|e| e.exit());
    let aux_color = values_t!(matches, "aux-color", u8).ok().map(|x| {
        Pixel {
            red: x[0],
            green: x[1],
            blue: x[2],
        }
    });
    let effect = {
        if matches.is_present("effect") {
            let param = value_t!(matches, "param", u8).unwrap_or(5);
            match value_t!(matches, "effect", String)
                .unwrap_or_else(|e| e.exit())
                .as_ref() {
                "Constant" => OldEffect::Constant,
                "Flash" => OldEffect::Width(param),
                "SetPix" => OldEffect::SetPix(param),
                "Width" => OldEffect::Width(param),
                "DoubleWidth" => OldEffect::DoubleWidth(param),
                "QuadWidth" => OldEffect::QuadWidth(param),
                "Edges" => OldEffect::Edges(param),
                _ => panic!(),
            }
        } else {
            OldEffect::Constant
        }
    };
    let aux_effect = {
        if matches.is_present("aux-effect") {
            let aux_param = value_t!(matches, "aux-param", u8).unwrap_or(5);
            match value_t!(matches, "aux-effect", String)
                .unwrap_or_else(|e| e.exit())
                .as_ref() {
                "None" => OldAuxEffect::None,
                "Offset" => OldAuxEffect::Offset(aux_param),
                "FillLeft" => OldAuxEffect::FillLeft(aux_param),
                "FillCenter" => OldAuxEffect::FillCenter(aux_param),
                "FillRight" => OldAuxEffect::FillRight(aux_param),
                "FillEdges" => OldAuxEffect::FillEdges(aux_param),
                "FillDouble" => OldAuxEffect::FillDouble(aux_param),
                _ => panic!(),
            }
        } else {
            OldAuxEffect::None
        }
    };

    let mut reactor = Core::new().chain_err(|| "Error creating tokio reactor")?;

    let server = LedServer::new(&reactor.handle(), addrs).chain_err(
        || "Error creating LedServer",
    )?;

    let mut buf = vec![Default::default(); 427];
    for (i, chunk) in buf.chunks_mut(61).enumerate() {
        let color = match i % 3 {
            0 => {
                Pixel {
                    red: 255,
                    green: 0,
                    blue: 0,
                }
            }
            1 => {
                Pixel {
                    red: 255,
                    green: 128,
                    blue: 196,
                }
            }
            2 => {
                Pixel {
                    red: 0,
                    green: 0,
                    blue: 255,
                }
            }
            _ => unreachable!(),
        };
        for p in chunk {
            *p = color;
        }
    }
    let buf = gen_effect(color, effect, aux_color, aux_effect, 427);

    assert!(buf.len() == 427);
    loop {
        server.store(buf.clone());
        let timeout = Timeout::new(Duration::from_secs(1), &reactor.handle())
            .chain_err(|| "Error creating timeout")?;
        reactor.run(timeout);
        if let Some(s) = server.load_nonlocal() {
            if *s == buf {
                return Ok(());
            }
        }
    }
    //set_pixels4(&mut serial, &buf);
    //set_effect_compat(
    //    &mut serial,
    //    Pixel {
    //        red: color[0],
    //        green: color[1],
    //        blue: color[2],
    //    },
    //    effect,
    //    aux_color,
    //    aux_effect,
    //    427,
    //).unwrap();
    Ok(())
}
