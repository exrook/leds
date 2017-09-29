#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate led_control;
extern crate lednet;
extern crate tokio_core;
#[macro_use]
extern crate error_chain;
extern crate palette;

use std::thread;
use std::path::{Path, PathBuf};
use rocket::response::NamedFile;
use rocket::request::{Form, State};

use led_control::{Pixel, OldEffect, OldAuxEffect, set_effect_compat, set_pixels4, setup,
                  gen_effect};
use palette::Rgb;

use lednet::LedServer;

use tokio_core::reactor::{Core, Timeout};

mod errors {
    error_chain!{}
}
use errors::*;

quick_main!(run);

fn run() -> Result<()> {
    println!("Hello, world!");

    let mut reactor = Core::new().chain_err(|| "Error creating tokio reactor")?;

    let ledserver = LedServer::new(&reactor.handle(), &[[0, 0, 0, 0].into()])
        .chain_err(|| "Error creating LedServer")?;
    thread::spawn(move || {
        let rocket = rocket::ignite()
            .mount("/", routes![index, static_files])
            .mount("/set", routes![set_lights])
            .mount("/set_color", routes![set_color])
            .manage(ledserver)
            .launch();
    });
    loop {
        reactor.turn(None)
    }
    Ok(())
}

#[get("/")]
fn index() -> Option<NamedFile> {
    static_files("index.html".into())
}

#[get("/<file..>")]
fn static_files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}

#[derive(FromForm)]
struct SetColor {
    r: f32,
    g: f32,
    b: f32,
}

#[post("/", data = "<c>")]
fn set_lights(c: Form<SetColor>, led: State<LedServer>) {
    let c = c.into_inner();
    let color = Rgb::new(c.r, c.g, c.b).to_pixel();
    let v = vec![color; 427];
    led.store(v.clone());
    thread::sleep_ms(10);
    led.store(v.clone());
    thread::sleep_ms(10);
    led.store(v)
}

#[derive(FromForm)]
struct HexColor {
    color: String,
}

#[post("/", data = "<c>")]
fn set_color(c: Form<HexColor>, led: State<LedServer>) -> Result<()> {
    let c = c.into_inner().color;
    let r = u8::from_str_radix(&c.get(1..3).chain_err(|| "Couldn't parse string")?, 16)
        .chain_err(|| "Couldn't parse string")?;
    let g = u8::from_str_radix(&c.get(3..5).chain_err(|| "Couldn't parse string")?, 16)
        .chain_err(|| "Couldn't parse string")?;
    let b = u8::from_str_radix(&c.get(5..7).chain_err(|| "Couldn't parse string")?, 16)
        .chain_err(|| "Couldn't parse string")?;
    let color = Pixel {
        red: r,
        green: g,
        blue: b,
    };
    let v = vec![color; 427];
    led.store(v.clone());
    thread::sleep_ms(10);
    led.store(v.clone());
    thread::sleep_ms(10);
    led.store(v);
    Ok(())
}
