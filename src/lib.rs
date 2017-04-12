extern crate serial;

use std::io;

use std::io::prelude::*;
use serial::prelude::*;
use std::time::Duration;
use std::thread::sleep;

use std::f64::consts::PI;

#[derive(Clone,Debug)]
pub struct Pixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug)]
pub enum Effect {
    Constant,
    Flash(u8)
}

pub fn setup() -> serial::SystemPort {
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
    return port;
}

pub fn set_pixels<T: SerialPort>(port: &mut T, pixels: Vec<Pixel>) -> io::Result<()> {
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
pub fn set_pixels2<T: SerialPort>(port: &mut T, pixels: Vec<Pixel>) -> io::Result<()> {
    let bytes: Vec<u8> = pixels.into_iter().enumerate().flat_map(|(i,p)| {vec!('c' as u8,p.red,p.green,p.blue,'l' as u8,i as u8)}).collect();
    println!("{:#?}", bytes);
    try!(port.flush());
    try!(port.write_all(&bytes));
    try!(port.write(&['a' as u8]));
    try!(port.flush());
    Ok(())
}
pub fn set_pixels3<T: SerialPort>(port: &mut T, pixels: Vec<Pixel>) -> io::Result<()> {
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

pub fn set_effect<T: SerialPort>(port: &mut T, color: Pixel, effect: Effect) -> io::Result<()> {
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
