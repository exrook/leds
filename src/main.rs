extern crate serial;

use std::io;

use std::io::prelude::*;
use serial::prelude::*;
use std::time::Duration;
use std::thread::sleep;

use std::f64::consts::PI;

#[derive(Clone,Debug)]
struct Pixel {
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Debug)]
enum Effect {
    Constant,
    Flash(u8)
}

fn main() {
    let mut port = serial::open("/dev/ttyACM0").unwrap();
    port.reconfigure(&|settings| {
        try!(settings.set_baud_rate(serial::BaudOther(230400)));
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    }).unwrap();;
    port.set_dtr(false);
    port.set_timeout(Duration::from_millis(1000));
    println!("Timeout: {:#?}", port.timeout());
    sleep(Duration::from_millis(2000));
    //let mut buf = String::new();
    //port.read_to_string(&mut buf).unwrap();
    //println!("String: {}", buf);
    //port.write(&vec!(255,255,255,255,108,101,100,122)).unwrap(); // write header
    //port.write(&vec!(255,0,0,0)).unwrap();
    //port.write(&vec!(0,255,0,0)).unwrap();
    //port.write(&vec!(0,0,255,0)).unwrap();
    //port.write(&vec!(0,255,0,0)).unwrap();
    //port.write(&vec!(0,255,255,0)).unwrap();
    //port.write(&vec!(0,0,255,255)).unwrap();
    //port.flush().unwrap();
    //let mut p = Vec::new();
    //for i in 0..300 as u64 {
    //   println!("creating pixel {}", i);
    //   p.push(Pixel{
    //       //red: (f64::cos(i as f64*(PI/30.0))*-127.0 + 128.0) as u8,
    //       red: 0,
    //       green: 0,
    //       blue: 255,
    //   });
    //}
    //println!("{}", p.len());
    //set_pixels(&mut port, vec!(Pixel{red: 0,green: 255, blue: 0}));
    //loop {
    //    set_effect(
    //        &mut port,
    //        Pixel { 
    //            red: 128,
    //            green: 0,
    //            blue: 0,
    //        },
    //        Effect::Constant
    //    );
    //    sleep(Duration::from_millis(3000));
    //    set_effect(
    //        &mut port,
    //        Pixel { 
    //            red: 255,
    //            green: 255,
    //            blue: 255,
    //        },
    //        Effect::Flash(100)
    //    );
    //    sleep(Duration::from_millis(3000));
    //}
    set_effect(
        &mut port,
        Pixel { 
            red: 0,
            green: 255,
            blue: 0,
        },
        Effect::Constant
    );
    let mut c = 0;
    //loop {
    //    println!("{:#?}",set_pixels3(&mut port, p.clone()));
    //    for (i,px) in p.iter_mut().enumerate() {
//  //          if i == c {
//  //              px.red = 255;
//  //          } else {
//  //              px.red = 0;
//  //          }
    //        //px.red = ((px.red as u32 + 1)%128) as u8;
    //        //px.red = ((px.red as i32 + 128)%255) as u8;
    //        //px.red = (f64::cos(i as f64*(PI/30.0))*-127.0 + 128.0) as u8;
    //        //i = i + 1.0;
    //        //let (r,g,b) = (px.red,px.green,px.blue);
    //        //if (px.red < 10) {
    //        //    px.red = 128;
    //        //    px.green = 0;
    //        //}
    //        //px.red = r/2;
    //        //px.green = ((((128-r as i32)-(g as i32*2)).abs() % 255) as u8);
    //    }
    //    c = (c + 1)%300; // this isn't actually causing the shifting effect it seems
    //    println!("{}", c);
    //    //sleep(Duration::from_millis(000));
    //}
    //println!("{:#?}",set_pixels3(&mut port, p));
}

fn set_pixels<T: SerialPort>(port: &mut T, pixels: Vec<Pixel>) -> io::Result<()> {
    let mut bytes: Vec<u8> = pixels.into_iter().flat_map(|p| {vec!(p.red,p.green,p.blue,0)}).collect();
    {
        let last = bytes.len()-1;
        bytes[last] = 255;
    }
    try!(port.flush());
    try!(port.write_all(&vec!(255,255,255,255,108,101,100,122)));
    let mut resp = String::new();
    try!(port.read_to_string(&mut resp));
    println!("{}", resp);
    try!(port.write_all(&bytes));
    try!(port.flush());
    Ok(())
}
fn set_pixels2<T: SerialPort>(port: &mut T, pixels: Vec<Pixel>) -> io::Result<()> {
    let bytes: Vec<u8> = pixels.into_iter().enumerate().flat_map(|(i,p)| {vec!('c' as u8,p.red,p.green,p.blue,'l' as u8,i as u8)}).collect();
    println!("{:#?}", bytes);
    try!(port.flush());
    try!(port.write_all(&bytes));
    try!(port.write(&['a' as u8]));
    try!(port.flush());
    Ok(())
}
fn set_pixels3<T: SerialPort>(port: &mut T, pixels: Vec<Pixel>) -> io::Result<()> {
    //let mut bytes: Vec<u8> = pixels.into_iter().flat_map(|p| {vec!(p.red,p.green,p.blue)}).collect();
    //println!("Size: {}, Size/3: {}", bytes.len(), bytes.len()/3);
    println!("Num pixels: {}", pixels.len());
    try!(port.flush());
    //let chunks = bytes.chunks(3);
    //for (i,chunk) in chunks.enumerate() {
    for (i,pixel) in pixels.iter().enumerate() {
        //println!("Writing pixel {}", i);
        //try!(port.write_all(&chunk));
        try!(port.write_all(&vec!(pixel.red,pixel.green,pixel.blue)));
        sleep(Duration::from_millis(10));
        //sleep(Duration::new(0,300000));
    }
    Ok(())
}

fn set_effect<T: SerialPort>(port: &mut T, color: Pixel, effect: Effect) -> io::Result<()> {
    println!("Setting effect {:#?}, color: {:?}", effect, color);
    try!(port.write_all(&vec!(color.red,color.green,color.blue)));
    println!("Writing: {:?}", &vec!(color.red,color.green,color.blue));
    match effect {
        Effect::Constant => {
            try!(port.write_all(&vec!(0,0)));
            println!("Writing: {:?}", &vec!(0,0));
        }
        Effect::Flash(rate) => {
            try!(port.write_all(&vec!(1,rate)));
            println!("Writing: {:?}", &vec!(1,rate));
        }
    }
    try!(port.flush());
    Ok(())
}
