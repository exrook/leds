extern crate lednet;
extern crate vamp;
extern crate portaudio;
extern crate palette;
extern crate led_control;

extern crate tokio_core;
extern crate futures;
extern crate multinet;

extern crate bincode;
extern crate serde;
extern crate rand;

#[macro_use]
extern crate error_chain;
extern crate clap;

extern crate probability;

use std::ffi::CString;
use std::thread::sleep;
use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::sync::Arc;
use std::net::Ipv4Addr;
use std::panic;
use std::f32::consts::PI;

use portaudio::PortAudio;
use portaudio::stream::{Parameters, InputSettings};

use palette::{Hsv, RgbHue, Rgb, Hue, IntoColor, FromColor};
use palette::gradient::Gradient;

use vamp::{PluginLoader, RealTime};

use led_control::{Effect, AuxEffect};

use tokio_core::reactor::Core;

use clap::{App, Arg};

use lednet::LedServer;

use probability::distribution::{Continuous, Distribution, Gaussian};

mod errors {
    error_chain!{}
}

use errors::*;

quick_main!(run);

fn run() -> Result<()> {
    let matches = App::new("LED Sender")
        .arg(
            Arg::with_name("address")
                .short("a")
                .takes_value(true)
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

    let pa = PortAudio::new().unwrap();
    let devs = pa.devices().unwrap();

    let mut p = None;
    let mut latency = 0.0;
    for d in devs {
        let d = d.unwrap();
        let (index, info) = d;
        println!("Device #{}", index.0);
        println!("{}", info.name);
        println!(
            "In Channels: {} Out Channels: {}\n",
            info.max_input_channels,
            info.max_output_channels
        );
        if info.name == "pulse" {
            p = Some(index);
            latency = info.default_low_input_latency;
        }
    }
    let p = p.unwrap();

    let pl = PluginLoader::get_instance();
    let mut pl = pl.lock().unwrap();
    println!("{:#?}", pl.list_plugins());
    let mut plug = pl.load_plugin(
        CString::new("vamp-example-plugins:amplitudefollower").unwrap(),
        44200.0,
        0x03,
    ).unwrap();
    println!("{:#?}", plug.get_name());
    let block_size = match plug.get_preferred_block_size() {
        0 => 1024,
        x => x,
    };
    println!("Using block size: {:#?}", block_size);

    let params: Parameters<f32> = Parameters::new(p, 1, true, latency);
    let in_settings = InputSettings::new(params, 44100.0, block_size as u32);

    match plug.initialise(1, 1024, block_size) {
        Ok(()) => println!("Initialized successfully"),
        Err(()) => panic!("Couldn't initialise plugin"),
    }
    plug.reset();

    let mut stream = pa.open_blocking_stream(in_settings).unwrap();
    stream.start().unwrap();
    sleep(Duration::from_millis(10));

    let conv = Arc::new(AtomicUsize::new(0));
    let conv2 = conv.clone();

    let crashed = Arc::new(AtomicBool::new(false));
    let crashed2 = crashed.clone();
    thread::spawn(move || {
        match panic::catch_unwind(move || -> Result<()> {

            let mut reactor = Core::new().unwrap();

            let ledserver = LedServer::new(&reactor.handle(), addrs).chain_err(
                || "Error starting led server",
            )?;

            let mut color = Hsv::new(RgbHue::from(0.0), 1.0, 0.0);
            let mut color2 = Hsv::new(RgbHue::from(0.0), 1.0, 0.0);
            let mut last_points = vec![0.0; 10];
            let mut count = 0;
            let mut width = 0.0;
            let mut offset: u8 = 0;

            let mut pixels = vec![Rgb::new(0f32, 0f32, 0f32); 427];

            let mut counter = 0.0;

            loop {
                sleep(Duration::from_millis(20));
                let out = conv2.load(Ordering::Relaxed) as f32;
                let pow_f = out * (1.0 / 2048.0);
                let sum: f32 = last_points.iter().sum();
                let avg = sum / (last_points.len() as f32);
                let diff_points: Vec<_> = last_points
                    .windows(2)
                    .map(|v| (v[1] - v[0] as f32).abs())
                    .collect();
                let diff_avg: f32 = (diff_points.iter().sum::<f32>()) /
                    ((diff_points.len() - 1) as f32);
                println!("sum: {:#?}", sum);
                println!("avg: {:#?}", avg);
                println!("diff_avg: {:#?}", diff_avg);

                color.hue = color.hue + RgbHue::from((16.0 * pow_f + 1.0).log2());
                //color2.hue = color2.hue+RgbHue::from((16.0*pow_f+1.0).log2());
                color.value = pow_f + color.value * 0.1;
                width = color.value.max(width * 0.5);

                let diff = (pow_f - last_points[count]).max(0.0);

                println!("Above avg diff: {}", diff > diff_avg);
                let points_above: f32 = diff_points
                    .iter()
                    .map(|x| {
                        if diff > *x {
                            return 1.0;
                        }
                        0.0
                    })
                    .sum();
                println!("Points above diff: {}", points_above);
                color.saturation = (color.saturation + 0.25).min(1.0);
                if points_above > (diff_points.len() as f32 - 1.0) {
                    // Add in something to flash brighter if the last point was darker than average
                    color.value = (avg * 3.0).max(1.0).min(color.value);
                    color.hue = color.hue + RgbHue::from(avg * 4.0 * 90.0);
                    color2.hue = color2.hue + RgbHue::from(avg * 1.0 * 90.0);
                    color.saturation = 0.0;
                    width = (2.0 * avg).max(color.value).min(1.0);
                    //color.value *= 0.8;
                }
                color2.hue = color2.hue + RgbHue::from(1.0);
                color2.saturation = 1.0f32;
                color2.value = 0.9 - (color.value * 0.8);
                color2.value *= 0.5;

                offset = (offset + 1) % 150;

                for p in pixels.iter_mut() {
                    *p = Rgb::new(0.0, 0.0, 0.0);
                }

                let america_gradient = Gradient::new(vec![
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(328.0), 0.5, 1.0),
                    Hsv::new(RgbHue::from(328.0), 0.5, 1.0),
                    Hsv::new(RgbHue::from(328.0), 0.5, 1.0),
                    Hsv::new(RgbHue::from(240.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(240.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(240.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                ]);
                let rainbow_gradient = Gradient::new(vec![
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(180.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(360.0), 1.0, 1.0),
                ]);
                let reverse_double_gradient = Gradient::new(vec![
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(10.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                ]);
                let double_rainbow_gradient = Gradient::new(vec![
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(90.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(180.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(270.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(300.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(270.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(180.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(90.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                ]);
                let gmu_gradient = Gradient::new(vec![
                    Hsv::new(RgbHue::from(120.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(120.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(52.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(52.0), 1.0, 1.0),
                ]);
                let orange_gradient = Gradient::new(vec![
                    Hsv::new(RgbHue::from(30.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(30.0), 1.0, 1.0),
                ]);
                let red_blue = Gradient::new(vec![
                    Hsv::new(RgbHue::from(300.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(300.0), 1.0, 1.0),
                ]);
                let red_green = Gradient::new(vec![
                    Hsv::new(RgbHue::from(120.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(120.0), 1.0, 1.0),
                ]);
                let blue_blue = Gradient::new(vec![
                    Hsv::new(RgbHue::from(280.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(200.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(280.0), 1.0, 1.0),
                ]);
                let red_blue = Gradient::new(vec![
                    Hsv::new(RgbHue::from(280.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(280.0), 1.0, 1.0),
                ]);
                let red_pink = Gradient::new(vec![
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(340.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(340.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                    Hsv::new(RgbHue::from(0.0), 1.0, 1.0),
                ]);
                let mut america_gradient_vals = america_gradient.take(61 * 3).cycle();
                let chunks_len = pixels.len() / 61;
                for (i, chunk) in pixels.chunks_mut(61).enumerate() {
                    let america = match i % 3 {
                        0 => Rgb::new(1.0, 0.0, 0.0),
                        1 => Rgb::new(1.0, 0.5, 0.75),
                        2 => Rgb::new(0.0, 0.0, 1.0),
                        _ => unreachable!(),
                    } * avg;
                    let mexico = match i % 2 {
                        0 => Rgb::new(1.0, 0.0, 0.0),
                        1 => Rgb::new(1.0, 0.5, 0.0),
                        _ => unreachable!(),
                    } * color.value;
                    let double_rainbow = match i % 2 {
                        0 => color,
                        1 => color.shift_hue(RgbHue::from(180.0)),
                        _ => unreachable!(),
                    };
                    let quad_rainbow = match i % 4 {
                        0 => color,
                        1 => color.shift_hue(RgbHue::from(90.0)),
                        2 => color.shift_hue(RgbHue::from(180.0)),
                        3 => color.shift_hue(RgbHue::from(270.0)),
                        _ => unreachable!(),
                    };
                    let width = (width * 0.85).powf(0.5) * 0.8 + 0.2;
                    let effect = Effect::Width(width);
                    let aux_effect = AuxEffect::FillEdges((1.0 - width).max(0.0));
                    let mode = 2;
                    let mut america_gradient_vals: Vec<_> =
                        america_gradient_vals.by_ref().take(chunk.len()).collect();
                    match mode {
                        0 => {
                            gen_effect_pixels(
                                //Some(america),
                                double_rainbow_gradient
                                    .take(61)
                                    .skip((61.0 / 2.0 - (width * 61.0 / 2.0)).ceil() as usize)
                                    .map(|mut c| {
                                        //c.value = color.value;
                                        c.value = avg;
                                        c.value = (c.value * 0.8 + 0.2);
                                        //c.saturation = color.saturation;
                                        c
                                    })
                                    .collect::<Vec<_>>(),
                                effect,
                                Some(vec![Rgb::new(0.0, 0.0, 0.0)]),
                                aux_effect,
                                chunk,
                            )
                        }
                        1 => {
                            let mut america_gradient_vals = america_gradient.take(chunk.len());
                            for ((n, p), grad) in chunk.into_iter().enumerate().zip(
                                america_gradient_vals.by_ref(),
                            )
                            {
                                let n = i * 61 + n;
                                *p = grad.into();
                            }
                        }
                        2 => {
                            let distr = Gaussian::new(0.5, width as f64 / 4.0);
                            let c = rainbow_gradient.get(i as f32 / chunks_len as f32);
                            gen_effect_function(chunk, |pos, len, old_c: Rgb| {
                                let mut c = rainbow_gradient.get(
                                    ((i as f32 / chunks_len as f32) + (pos / chunks_len as f32) +
                                         counter) %
                                        1.0,
                                );
                                //let mut c = c;
                                //let mut c = Hsv::new(RgbHue::from(0.0), 1.0, 1.0);
                                //c.value *= distr.density(pos as f64) as f32 *
                                //    (color.value.powf(0.5));
                                //c.value *=
                                //    (0.5 - (distr.distribution(pos as f64) as f32 - 0.5).abs()) *
                                //        2.0 *
                                //        (color.value.powf(0.5));
                                let cos_curve = (PI * 2.0 * (pos - 0.5)).cos() * 0.5 + 0.5;
                                let scale = (cos_curve - (1.0 - color.value)) *
                                    (1.0 - color.value) *
                                    1.5 + 0.1;
                                c.value *= scale.max(0.0);
                                c.value = c.value.max((1.0 - (pos * 2.0 - 1.0).abs()) * 0.04);
                                //println!(
                                //    "color.value: {}, pos: {}, cos_curve: {}, scale: {}, c.value: {}",
                                //    color.value,
                                //    pos,
                                //    cos_curve,
                                //    scale.max(0.0),
                                //    c.value
                                //);
                                c
                            })
                        }
                        _ => unimplemented!(),
                    };
                }
                counter = (counter + 0.01 * (color.value.sqrt().sqrt())) % 1.0;
                ledserver.store(pixels.clone().into_iter().map(|p| p.to_pixel()).collect());

                count = (count + 1) % last_points.len();
                last_points[count] = pow_f;
                reactor.turn(Some(Duration::from_millis(5)));
            }
        }) {
            Ok(_) => {}
            Err(e) => {
                crashed.store(true, Ordering::Release);
                panic::resume_unwind(e)
            }
        }
    });
    loop {
        if crashed2.load(Ordering::Acquire) {
            return Err("Other thread crashed".into());
        }
        let data = vec![stream.read(block_size as u32).unwrap().to_vec()];
        let time = RealTime::new(0, 0);
        let feat = plug.process(data, time);
        //println!("{:#?}", feat.get(&0).unwrap()[0].values[0]);
        let amplitude = feat.get(&0).unwrap()[0].values[0];
        conv.store((amplitude * 2048.0) as usize, Ordering::Relaxed);
    }
}

fn gen_effect_function<
    COut: Into<CVec>,
    CIn: From<CVec>,
    F: FnMut(f32, usize, CIn) -> COut,
    CVec: Clone,
>(
    v: &mut [CVec],
    mut f: F,
) {
    let len = v.len();
    for (i, c) in v.iter_mut().enumerate() {
        *c = f(i as f32 / len as f32, len, CIn::from(c.clone())).into();
    }
}

fn gen_effect_pixels<
    G1: IntoIterator<Item = C1, IntoIter = I1>,
    I1: Clone + Iterator<Item = C1>,
    C1: IntoColor<f32>,
    G2: IntoIterator<Item = C2, IntoIter = I2>,
    I2: Clone + Iterator<Item = C2>,
    C2: IntoColor<f32> + Default,
>(
    color: G1,
    effect: Effect,
    color2: Option<G2>,
    aux_effect: AuxEffect,
    pixels: &mut [Rgb],
) {
    let count = pixels.len();
    let mut color = color.into_iter().cycle().map(|c| c.into_rgb());
    let mut color2 = color2.map(|c| c.into_iter().cycle().map(IntoColor::into_rgb));
    match effect {
        Effect::Constant => {
            for p in pixels.iter_mut() {
                *p = color.next().unwrap();
            }
        }
        Effect::Flash(_) => unimplemented!("Flash doesn't make sense with the new system"),
        Effect::Width(width) => {
            println!(
                "Width: {:?}, Floor: {:?}, Original Width: {:?}",
                width * 0.5 * count as f32,
                (width * 0.5 * count as f32).floor(),
                width.min(1.0)
            );
            println!("count: {:?}, count/2: {:?}", count, count / 2);
            let width = (width.min(1.0) * 0.5 * count as f32).floor();
            for p in pixels[((count / 2) - width as usize)..((count / 2) + width as usize)]
                .iter_mut()
            {
                *p = color.next().unwrap();
            }
        }
        Effect::DoubleWidth(width) => {
            let width = width * count as f32;
            for p in pixels[((count / 4) - width as usize)..((count / 4) + width as usize)]
                .iter_mut()
            {
                *p = color.next().unwrap();
            }
            for p in pixels[((count * 3 / 4) - width as usize)..
                                ((count * 3 / 4) + width as usize)]
                .iter_mut()
            {
                *p = color.next().unwrap();
            }
        }
        Effect::QuadWidth(width) => {
            let width = width * count as f32;
            for p in pixels[((count / 8) - width as usize)..((count / 8) + width as usize)]
                .iter_mut()
            {
                *p = color.next().unwrap();
            }
            for p in pixels[((count * 3 / 8) - width as usize)..
                                ((count * 3 / 8) + width as usize)]
                .iter_mut()
            {
                *p = color.next().unwrap();
            }
            for p in pixels[((count * 5 / 8) - width as usize)..
                                ((count * 5 / 8) + width as usize)]
                .iter_mut()
            {
                *p = color.next().unwrap();
            }
            for p in pixels[((count * 7 / 8) - width as usize)..
                                ((count * 7 / 8) + width as usize)]
                .iter_mut()
            {
                *p = color.next().unwrap();
            }
        }
        Effect::Edges(width) => {
            let width = width * count as f32;
            for p in pixels[..width as usize].iter_mut() {
                *p = color.next().unwrap();
            }
            for p in pixels[count - width as usize..].iter_mut() {
                *p = color.next().unwrap();
            }
        }
    };

    //let color2 = color2.unwrap_or_else(Some(C2::default()).into_iter().cycle());
    match aux_effect {
        AuxEffect::None => {}
        AuxEffect::Offset(amount) => {
            let p2 = pixels.to_owned();
            for (i, p) in pixels.iter_mut().enumerate() {
                *p = p2[(i + (amount as usize)) % count];
            }
        }
        AuxEffect::FillLeft(len) => {
            let len = len * count as f32;
            for p in pixels[..len as usize].iter_mut() {
                *p = color2
                    .as_mut()
                    .map(|x| x.next().unwrap())
                    .unwrap_or_default();
            }
        }
        AuxEffect::FillCenter(len) => {
            let len = len * count as f32;
            for p in pixels[(count / 2) - len as usize..(count / 2) + len as usize].iter_mut() {
                *p = color2
                    .as_mut()
                    .map(|x| x.next().unwrap())
                    .unwrap_or_default();
            }
        }
        AuxEffect::FillRight(len) => {
            let len = len * count as f32;
            for p in pixels[count - len as usize..].iter_mut() {
                *p = color2
                    .as_mut()
                    .map(|x| x.next().unwrap())
                    .unwrap_or_default();
            }
        }
        AuxEffect::FillEdges(len) => {
            println!(
                "len: {:?}, Floor: {:?}, Original len: {:?}",
                len * 0.5 * count as f32,
                (len * 0.5 * count as f32).floor(),
                len.min(1.0)
            );
            println!("count: {:?}, count/2: {:?}", count, count / 2);
            let len = len.min(1.0) * 0.5 * count as f32;
            for p in pixels[..len as usize].iter_mut() {
                *p = color2
                    .as_mut()
                    .map(|x| x.next().unwrap())
                    .unwrap_or_default();
            }
            for p in pixels[count - len as usize..].iter_mut() {
                *p = color2
                    .as_mut()
                    .map(|x| x.next().unwrap())
                    .unwrap_or_default();
            }
        }
        AuxEffect::FillDouble(len) => {
            let len = len * count as f32;
            // fill left
            for p in pixels[..len as usize].iter_mut() {
                *p = color2
                    .as_mut()
                    .map(|x| x.next().unwrap())
                    .unwrap_or_default();
            }
            // fill center
            for p in pixels[(count / 2) - len as usize..(count / 2) + len as usize].iter_mut() {
                *p = color2
                    .as_mut()
                    .map(|x| x.next().unwrap())
                    .unwrap_or_default();
            }
            // fill right
            for p in pixels[count - len as usize..].iter_mut() {
                *p = color2
                    .as_mut()
                    .map(|x| x.next().unwrap())
                    .unwrap_or_default();
            }
        }
    };
}
