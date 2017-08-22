extern crate set_neopixels;
#[macro_use]
extern crate clap;

use std::time::{Duration,self};
use std::thread::sleep;

use clap::{App,Arg};

use set_neopixels::{Pixel,Effect,AuxEffect,set_pixels4,setup};

fn main() {
    let matches = App::new("set_neopixels")
        .arg(Arg::with_name("serial").value_name("port").required(true).takes_value(true).index(1))
        .arg(Arg::with_name("color").number_of_values(3).required(true).takes_value(true).index(2))
        .arg(Arg::with_name("aux-color").short("c").number_of_values(3).takes_value(true).required(false))
        .arg(Arg::with_name("effect").short("e").takes_value(true).required(false)
             .possible_values(&["Constant", "Flash", "SetPix", "Width", "DoubleWidth", "QuadWidth", "Edges"]))
        .arg(Arg::with_name("aux-effect").short("a").takes_value(true).required(false)
             .possible_values(&["None", "Offset", "FillLeft", "FillCenter", "FillRight", "FillEdges", "FillDouble"]))
        .arg(Arg::with_name("param").short("p").takes_value(true).required(false).requires("effect"))
        .arg(Arg::with_name("aux-param").short("q").takes_value(true).required(false).requires("aux-effect"))
        .get_matches();
    let port = matches.value_of("serial").unwrap();
    let color = values_t!(matches, "color", u8).unwrap_or_else(|e| e.exit());
    let aux_color = values_t!(matches, "aux-color", u8).ok().map(|x|Pixel{ red: x[0], green: x[1], blue: x[2] });
    let effect = {
        if matches.is_present("effect") {
            let param = value_t!(matches,"param", u8).unwrap_or(5);
            match value_t!(matches, "effect", String).unwrap_or_else(|e| e.exit()).as_ref() {
                "Constant" => Effect::Constant,
                "Flash" => Effect::Width(param),
                "SetPix" => Effect::SetPix(param),
                "Width" => Effect::Width(param),
                "DoubleWidth" => Effect::DoubleWidth(param),
                "QuadWidth" => Effect::QuadWidth(param),
                "Edges" => Effect::Edges(param),
                _ => panic!()
            }
        } else {
            Effect::Constant
        }
    };
    let aux_effect = {
        if matches.is_present("aux-effect") {
            let aux_param = value_t!(matches,"aux-param", u8).unwrap_or(5);
            match value_t!(matches, "aux-effect", String).unwrap_or_else(|e| e.exit()).as_ref() {
                "None" => AuxEffect::None,
                "Offset" => AuxEffect::Offset(aux_param),
                "FillLeft" => AuxEffect::FillLeft(aux_param),
                "FillCenter" => AuxEffect::FillCenter(aux_param),
                "FillRight" => AuxEffect::FillRight(aux_param),
                "FillEdges" => AuxEffect::FillEdges(aux_param),
                "FillDouble" => AuxEffect::FillDouble(aux_param),
                _ => panic!()
            }
        } else {
            AuxEffect::None
        }
    };
    let mut serial = setup(port);
    sleep(Duration::from_secs(1));
    let mut v = vec![Pixel::from_slice(&color); 300];
    let mut i = 0;
    let mut tm = time::Instant::now();
    loop {
        println!("TIME TAKEN: {:?}", tm.elapsed());
        tm = time::Instant::now();
        for (n, p) in v.iter_mut().enumerate() {
            p.red = (n%255) as u8;
            p.green = i;
        }
        i = i.wrapping_add(1);
        set_pixels4(&mut serial, &v).unwrap();
        sleep(Duration::from_millis(25));
    }
}

