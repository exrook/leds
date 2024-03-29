extern crate serial;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;
#[cfg(feature = "palette")]
extern crate palette;
#[cfg(feature = "palette")]
extern crate num_traits;

use std::io;

use std::io::prelude::*;
use serial::prelude::*;
use std::time::Duration;
use std::thread::sleep;
use std::ffi::OsStr;

use std::f64::consts::PI;

#[cfg(feature = "palette")]
use palette::pixel::RgbPixel;
#[cfg(feature = "palette")]
use num_traits::Float;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Default, Copy, Eq, PartialEq)]
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

#[cfg(feature = "palette")]
impl<T: Float> RgbPixel<T> for Pixel {
    fn from_rgba(red: T, green: T, blue: T, alpha: T) -> Self {
        let (red, green, blue): (u8, u8, u8) = RgbPixel::from_rgba(red, green, blue, alpha);
        Pixel { red, green, blue }
    }
    fn to_rgba(&self) -> (T, T, T, T) {
        let Pixel { red, green, blue } = *self;
        (red, green, blue).to_rgba()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone)]
pub enum Effect {
    Constant,
    Flash(f32),
    Width(f32),
    DoubleWidth(f32),
    QuadWidth(f32),
    Edges(f32),
}

#[derive(Debug, Copy, Clone)]
pub enum OldEffect {
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
    Offset(f32),
    FillLeft(f32),
    FillCenter(f32),
    FillRight(f32),
    FillEdges(f32),
    FillDouble(f32),
}

#[derive(Debug, Copy, Clone)]
pub enum OldAuxEffect {
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
    port.set_dtr(true);
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
    //    .chunks(2000)
    //{
    //    port.write_all(c)?;
    //    sleep(Duration::from_millis(200));
    //    println!("CHUNK");
    //}
    //println!("DONEEEEEEEEEEEEEEEEEEEEE: {:?}", pixels.len());
    //Ok(())
}

pub fn set_effect_compat<T: SerialPort>(
    port: &mut T,
    color: Pixel,
    effect: OldEffect,
    color2: Option<Pixel>,
    aux_effect: OldAuxEffect,
    count: usize,
) -> io::Result<()> {
    set_pixels4(port, &gen_effect(color, effect, color2, aux_effect, count))
}
pub fn gen_effect(
    color: Pixel,
    effect: OldEffect,
    color2: Option<Pixel>,
    aux_effect: OldAuxEffect,
    count: usize,
) -> Vec<Pixel> {
    let mut pixels = vec![Default::default(); count];
    pixels[0] = Pixel {
        red: 0,
        green: 0,
        blue: 0,
    };
    match effect {
        OldEffect::Constant => {
            for p in pixels.iter_mut() {
                *p = color;
            }
        }
        OldEffect::Flash(rate) => unimplemented!("Flash doesn't make sense with the new system"),
        OldEffect::SetPix(num) => {
            pixels[num as usize] = color;
        }
        OldEffect::Width(width) => {
            for p in pixels[((count / 2) - width as usize)..((count / 2) + width as usize)]
                .iter_mut()
            {
                *p = color;
            }
        }
        OldEffect::DoubleWidth(width) => {
            for p in pixels[((count / 4) - width as usize)..((count / 4) + width as usize)]
                .iter_mut()
            {
                *p = color;
            }
            for p in pixels[((count * 3 / 4) - width as usize)..
                                ((count * 3 / 4) + width as usize)]
                .iter_mut()
            {
                *p = color;
            }
        }
        OldEffect::QuadWidth(width) => {
            for p in pixels[((count / 8) - width as usize)..((count / 8) + width as usize)]
                .iter_mut()
            {
                *p = color;
            }
            for p in pixels[((count * 3 / 8) - width as usize)..
                                ((count * 3 / 8) + width as usize)]
                .iter_mut()
            {
                *p = color;
            }
            for p in pixels[((count * 5 / 8) - width as usize)..
                                ((count * 5 / 8) + width as usize)]
                .iter_mut()
            {
                *p = color;
            }
            for p in pixels[((count * 7 / 8) - width as usize)..
                                ((count * 7 / 8) + width as usize)]
                .iter_mut()
            {
                *p = color;
            }
        }
        OldEffect::Edges(width) => {
            for p in pixels[..width as usize].iter_mut() {
                *p = color;
            }
            for p in pixels[count - width as usize..].iter_mut() {
                *p = color;
            }
        }
    };

    let color2 = color2.unwrap_or_else(Pixel::default);
    match aux_effect {
        OldAuxEffect::None => {}
        OldAuxEffect::Offset(amount) => {
            let p2 = pixels.clone();
            for (i, p) in pixels.iter_mut().enumerate() {
                *p = p2[(i + (amount as usize)) % count];
            }
        }
        OldAuxEffect::FillLeft(len) => {
            for p in pixels[..len as usize].iter_mut() {
                *p = color2
            }
        }
        OldAuxEffect::FillCenter(len) => {
            for p in pixels[(count / 2) - len as usize..(count / 2) + len as usize].iter_mut() {
                *p = color2
            }
        }
        OldAuxEffect::FillRight(len) => {
            for p in pixels[count - len as usize..].iter_mut() {
                *p = color2
            }
        }
        OldAuxEffect::FillEdges(len) => {
            for p in pixels[..len as usize].iter_mut() {
                *p = color2
            }
            for p in pixels[count - len as usize..].iter_mut() {
                *p = color2
            }
        }
        OldAuxEffect::FillDouble(len) => {
            // fill left
            for p in pixels[..len as usize].iter_mut() {
                *p = color2
            }
            // fill center
            for p in pixels[(count / 2) - len as usize..(count / 2) + len as usize].iter_mut() {
                *p = color2
            }
            // fill right
            for p in pixels[count - len as usize..].iter_mut() {
                *p = color2
            }
        }
    };
    pixels
}

pub fn set_effect<T: SerialPort>(
    port: &mut T,
    color: Pixel,
    effect: OldEffect,
    color2: Option<Pixel>,
    aux_effect: OldAuxEffect,
) -> io::Result<()> {
    let mut bytes = Vec::new();
    bytes.append(&mut vec![color.red, color.green, color.blue]);
    bytes.append(&mut match effect {
        OldEffect::Constant => vec![0, 0],
        OldEffect::Flash(rate) => vec![1, rate],
        OldEffect::SetPix(num) => vec![2, num],
        OldEffect::Width(width) => vec![3, width],
        OldEffect::DoubleWidth(width) => vec![4, width],
        OldEffect::QuadWidth(width) => vec![5, width],
        OldEffect::Edges(width) => vec![6, width],
    });
    bytes.append(&mut match color2 {
        Some(color2) => vec![color2.red, color2.green, color2.blue],
        None => vec![0, 0, 0],
    });
    bytes.append(&mut match aux_effect {
        OldAuxEffect::None => vec![0, 0],
        OldAuxEffect::Offset(amount) => vec![1, amount],
        OldAuxEffect::FillLeft(len) => vec![2, len],
        OldAuxEffect::FillCenter(len) => vec![3, len],
        OldAuxEffect::FillRight(len) => vec![4, len],
        OldAuxEffect::FillEdges(len) => vec![5, len],
        OldAuxEffect::FillDouble(len) => vec![6, len],
    });
    println!("Setting effect {:#?}, color: {:?}", effect, color);
    try!(port.write_all(&bytes));
    try!(port.flush());
    Ok(())
}
