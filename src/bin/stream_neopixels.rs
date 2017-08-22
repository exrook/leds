extern crate set_neopixels;
#[macro_use]
extern crate clap;
extern crate atomic_box;

use std::time::Duration;
use std::thread::{spawn,sleep};
use std::thread;
use std::io::BufRead;
use std::sync::Arc;

use clap::{App,Arg};

use set_neopixels::{Pixel,Effect,AuxEffect,set_effect,setup};
use atomic_box::AtomicBox;

fn main() {
    let matches = App::new("set_neopixels")
        .arg(Arg::with_name("serial").value_name("port").required(true).takes_value(true).index(1))
        .get_matches();
    let port = matches.value_of("serial").unwrap().to_owned();
    let b: Arc<AtomicBox<Option<(Pixel, Effect, Option<Pixel>, AuxEffect)>>> = Arc::new(AtomicBox::new(None));
    let b2 = b.clone();
    let handle = std::thread::spawn(move || {
        let mut serial = setup(&port);
        thread::park();
        loop {
            match Arc::try_unwrap(*(b.swap(None))) {
                Ok(Some(l)) => 
            set_effect(
                &mut serial,
                l.0, // color
                l.1,// effect
                l.2, // aux_color
                l.3 // aux_effect
            ).unwrap(),
                _ => {}
            }
        }
    });
    sleep(Duration::from_secs(1));
    let stdin = std::io::stdin();
    for l in stdin.lock().lines().filter_map(|x| x.ok()).filter_map(string_to_opts) {
        println!("{:?}", l);
        b2.store(Some(l));
        handle.thread().unpark();
        println!("unparked");
    }
    println!("lul");
}

fn string_to_opts<T: AsRef<str>>(s: T) -> Option<(Pixel, Effect, Option<Pixel>, AuxEffect)> {
    let v: Vec<_> = s.as_ref().split_whitespace().collect();
    if v.len() != 10 {
        return None
    }
    let color = match v[0..3].iter().map(|x|x.parse().ok()).collect::<Option<Vec<_>>>() {
        Some(c) => {
            Pixel {
                red: c[0],
                green: c[1],
                blue: c[2]
            }
        }
        None => return None
    };
    let param = match v[4].parse().ok() {
        Some(s) => s,
        None => return None
    };
    let effect = match v[3] {
        "Constant" => Effect::Constant,
        "Flash" => Effect::Width(param),
        "SetPix" => Effect::SetPix(param),
        "Width" => Effect::Width(param),
        "DoubleWidth" => Effect::DoubleWidth(param),
        "QuadWidth" => Effect::QuadWidth(param),
        "Edges" => Effect::Edges(param),
        _ => return None
    };
    let aux_color = match v[5..8].iter().map(|x|x.parse().ok()).collect::<Option<Vec<_>>>() {
        Some(c) => {
            Pixel {
                red: c[0],
                green: c[1],
                blue: c[2]
            }
        }
        None => return None
    };
    let aux_param = match v[9].parse().ok() {
        Some(s) => s,
        None => return None
    };
    let aux_effect = match v[8] {
        "None" => AuxEffect::None,
        "Offset" => AuxEffect::Offset(aux_param),
        "FillLeft" => AuxEffect::FillLeft(aux_param),
        "FillCenter" => AuxEffect::FillCenter(aux_param),
        "FillRight" => AuxEffect::FillRight(aux_param),
        "FillEdges" => AuxEffect::FillEdges(aux_param),
        "FillDouble" => AuxEffect::FillDouble(aux_param),
        _ => return None
    };
    Some((color, effect, Some(aux_color), aux_effect))
}
