extern crate vamp;
extern crate portaudio;
extern crate palette;
extern crate led_control;

use std::ffi::CString;
use std::thread::sleep;
use std::thread;
use std::time::Duration;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use portaudio::PortAudio;
use portaudio::stream::{Parameters, InputSettings};

use palette::{Hsv, RgbHue};
use palette::pixel::Srgb;
use palette::IntoColor;

use vamp::{PluginLoader, Plugin, RealTime};

use led_control::{Pixel, Effect, set_effect};

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
    sleep(Duration::from_millis(100));

    let conv = Arc::new(AtomicUsize::new(0));
    let conv2 = conv.clone();
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
