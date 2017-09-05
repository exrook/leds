extern crate serial;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

use std::io;

use std::io::prelude::*;
use serial::prelude::*;
use std::time::Duration;
use std::thread::sleep;
use std::ffi::OsStr;

use std::f64::consts::PI;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct Pixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Pixel {
    pub fn from_slice(s: &[u8]) -> Self {
        Pixel {
            red: s[0],
            green: s[1],
            blue: s[2],
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone)]
pub enum Effect {
    Constant,
    Flash(u8),
    SetPix(u8),
    Width(u8),
    DoubleWidth(u8),
    QuadWidth(u8),
    Edges(u8),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone)]
pub enum AuxEffect {
    None,
    Offset(u8),
    FillLeft(u8),
    FillCenter(u8),
    FillRight(u8),
    FillEdges(u8),
    FillDouble(u8),
}

pub fn setup<T: AsRef<OsStr> + ?Sized>(port: &T) -> serial::SystemPort {
    let mut port = serial::open(port).unwrap();
    port.reconfigure(&|settings| {
        try!(settings.set_baud_rate(serial::BaudOther(230400)));
        //try!(settings.set_baud_rate(serial::BaudOther(1000000)));
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
    let mut bytes: Vec<u8> = pixels
        .into_iter()
        .flat_map(|p| vec![p.red, p.green, p.blue, 0])
        .collect();
    {
        let last = bytes.len() - 1;
        bytes[last] = 255;
    }
    try!(port.flush());
    try!(port.write_all(
        &vec![255, 255, 255, 255, 108, 101, 100, 122],
    ));
    let mut resp = String::new();
    try!(port.read_to_string(&mut resp));
    println!("{}", resp);
    try!(port.write_all(&bytes));
    try!(port.flush());
    Ok(())
}
pub fn set_pixels2<T: SerialPort>(port: &mut T, pixels: Vec<Pixel>) -> io::Result<()> {
    let bytes: Vec<u8> = pixels
        .into_iter()
        .enumerate()
        .flat_map(|(i, p)| {
            vec!['c' as u8, p.red, p.green, p.blue, 'l' as u8, i as u8]
        })
        .collect();
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
    for (i, pixel) in pixels.iter().enumerate() {
        //println!("Writing pixel {}", i);
        //try!(port.write_all(&chunk));
        try!(port.write_all(&vec![pixel.red, pixel.green, pixel.blue]));
        sleep(Duration::from_millis(10));
        //sleep(Duration::new(0,300000));
    }
    Ok(())
}

pub fn set_pixels4<T: SerialPort>(port: &mut T, pixels: &[Pixel]) -> io::Result<()> {
    port.write_all(&pixels
        .into_iter()
        .flat_map(|p| vec![p.red, p.green, p.blue])
        .collect::<Vec<u8>>())
}

pub fn set_effect<T: SerialPort>(
    port: &mut T,
    color: Pixel,
    effect: Effect,
    color2: Option<Pixel>,
    aux_effect: AuxEffect,
) -> io::Result<()> {
    let mut bytes = Vec::new();
    bytes.append(&mut vec![color.red, color.green, color.blue]);
    bytes.append(&mut match effect {
        Effect::Constant => vec![0, 0],
        Effect::Flash(rate) => vec![1, rate],
        Effect::SetPix(num) => vec![2, num],
        Effect::Width(width) => vec![3, width],
        Effect::DoubleWidth(width) => vec![4, width],
        Effect::QuadWidth(width) => vec![5, width],
        Effect::Edges(width) => vec![6, width],
    });
    bytes.append(&mut match color2 {
        Some(color2) => vec![color2.red, color2.green, color2.blue],
        None => vec![0, 0, 0],
    });
    bytes.append(&mut match aux_effect {
        AuxEffect::None => vec![0, 0],
        AuxEffect::Offset(amount) => vec![1, amount],
        AuxEffect::FillLeft(len) => vec![2, len],
        AuxEffect::FillCenter(len) => vec![3, len],
        AuxEffect::FillRight(len) => vec![4, len],
        AuxEffect::FillEdges(len) => vec![5, len],
        AuxEffect::FillDouble(len) => vec![6, len],
    });
    println!("Setting effect {:#?}, color: {:?}", effect, color);
    try!(port.write_all(&bytes));
    try!(port.flush());
    Ok(())
}
