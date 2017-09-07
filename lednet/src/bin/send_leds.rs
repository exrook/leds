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
#[macro_use]
extern crate serde_derive;
extern crate rand;

use std::ffi::CString;
use std::thread::sleep;
use std::thread;
use std::time::Duration;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use portaudio::PortAudio;
use portaudio::stream::{Parameters, InputSettings};

use palette::{Hsv, RgbHue, Lch, LabHue};
use palette::pixel::Srgb;
use palette::IntoColor;

use vamp::{PluginLoader, Plugin, RealTime};

use led_control::{Pixel, Effect, AuxEffect, gen_effect};

use lednet::Message;

use futures::{Future, Stream, Sink};
use tokio_core::reactor::Core;
use multinet::{Server, ControlPacket, AssembledDataPacket, RecievedPacket, ChannelID, ServerHandle};

use bincode::Infinite;

const num_leds: usize = 427;

fn main() {
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
    thread::spawn(move || {
        let mut reactor = Core::new().unwrap();

        let (s, mut handle) = Server::new(reactor.handle(), &[0, 0, 0, 0u8].into()).unwrap();

        reactor.handle().spawn(s.map_err(
            |e| panic!("Error polling server: {:?}", e),
        ));

        let mut last = 0.0;
        let mut color = Hsv::new(RgbHue::from(0.0), 1.0, 0.0);
        let mut color2 = Hsv::new(RgbHue::from(0.0), 1.0, 0.0);
        //let mut color2 = Lch::new(50.0f32,128.0,LabHue::from(0.0));
        let mut last_points = vec![0.0; 10];
        let mut count = 0;
        let mut width = 0.0;
        let mut offset: u8 = 0;
        let mut effect = Effect::Constant;
        loop {
            sleep(Duration::from_millis(20));
            let out = conv2.load(std::sync::atomic::Ordering::Relaxed) as f32;
            let pow_f = (out * (1.0 / 2048.0));
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

            let mut diff = (pow_f - last_points[count]).max(0.0);

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
                width = (2.0 * avg).max(color.value).min(1.0)
            }
            //color.hue = RgbHue::from(0.0);
            //color.saturation = 0.0;
            //color.value = color.value * 0.5;

            //color2.hue = RgbHue::from({
            //    let current = color2.hue.to_positive_degrees() + 360.0;
            //    let target = color.hue.to_positive_degrees() + 360.0 + 180.0;
            //    let diff = (target - current);
            //    (current - 360.0) + (diff * 0.7)
            //});
            //color2.hue = color.hue + RgbHue::from(180.0);
            color2.hue = color2.hue + RgbHue::from(1.0);
            //color.hue = RgbHue::from(5f32);
            //color2.hue = RgbHue::from(5f32);
            //color.value = color.value * 0.05;
            //color2.hue = RgbHue::from(345f32);
            color2.saturation = 1.0f32;
            //color2.hue = RgbHue::from(0.0);
            //color2.hue = color2.hue + RgbHue::from(0.1);
            //color2.value = 0.2 - (color.value*0.1);
            //color2.hue = RgbHue::from(5f32);
            color2.value = 0.9 - (color.value * 0.8);
            //color2.value *= 0.015;
            //color2.value = color2.value * 0.1;
            //color2.saturation = 0.0;
            //color2.l = 1.0 as f32;

            let rgb = color.into_rgb();
            let rgb2 = color2.into_rgb();
            effect = {
                //if avg > 0.5 {
                //Effect::Width((width * (125.0 / 1.0)) as u8 + 25)
                Effect::Width((width * (125.0 / 1.0)) as u8 + 25)
                //} else if avg > 0.25 || (match effect { Effect::DoubleWidth(_) => true, _ => false } && avg > 0.2) {
                //Effect::DoubleWidth((width * (70.0 / 1.0)) as u8 + 5)
                //} else {
                //Effect::QuadWidth((width*(35.5/1.0)).min(33.0) as u8 + 2)
                //  //Effect::Edges((width*(140.0/1.0)) as u8 + 10)
                //}
            };
            offset = (offset + 1) % 150;
            println!("{:#?}", color);
            println!("{:#?}", rgb);
            send_effect(
                &mut handle,
                Pixel {
                    red: (rgb.red.max(0.0) * 255.0) as u8,
                    green: (rgb.green.max(0.0) * 255.0) as u8,
                    blue: (rgb.blue.max(0.0) * 255.0) as u8,
                },
                effect,
                Some(Pixel {
                    red: (rgb2.red.max(0.0) * 255.0) as u8,
                    green: (rgb2.green.max(0.0) * 255.0) as u8,
                    blue: (rgb2.blue.max(0.0) * 255.0) as u8,
                }),
                //AuxEffect::None
                //AuxEffect::Offset(37 + (color.hue.to_positive_degrees()*(75.0/360.0)) as u8)
                //AuxEffect::Offset(255),
                //AuxEffect::Offset(50),
                AuxEffect::FillEdges(150 - (width * (125.0 / 1.0)) as u8),
                //AuxEffect::FillDouble(61 - (width * 60.0 / 1.0) as u8),
                num_leds,
            );
            last = out;
            count = (count + 1) % last_points.len();
            last_points[count] = pow_f;
            reactor.turn(Some(Duration::from_millis(10)));
        }
    });
    loop {
        let data = vec![stream.read(block_size as u32).unwrap().to_vec()];
        let time = RealTime::new(0, 0);
        let feat = plug.process(data, time);
        //println!("{:#?}", feat.get(&0).unwrap()[0].values[0]);
        let amplitude = feat.get(&0).unwrap()[0].values[0];
        conv.store(
            (amplitude * 2048.0) as usize,
            std::sync::atomic::Ordering::Relaxed,
        );
    }
}

fn send_effect(
    handle: &mut ServerHandle,
    color: Pixel,
    effect: Effect,
    aux_color: Option<Pixel>,
    aux_effect: AuxEffect,
    count: usize,
) {
    let msg = gen_effect(color, effect, aux_color, aux_effect, count);
    let data = bincode::serialize(&msg, Infinite).unwrap();
    handle
        .start_send(ControlPacket::SendData(AssembledDataPacket::new(
            data,
            ChannelID::new(42),
            34
            //rand::random(),
        )))
        .unwrap();
}
