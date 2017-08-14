use std::str;
use std::collections::{HashMap, BTreeMap};
use std::convert::TryInto;

use lru_cache::LruCache;

use std::io;
use std::io::{Write, BufRead};
use std::net::SocketAddr;

use rand::{Rng, thread_rng};

use futures::{Stream, Sink};
use tokio_core::reactor::Handle;
use tokio_core::net::{UdpSocket, UdpCodec};
use serde::Serialize;
use serde::de::DeserializeOwned;
use bincode::{Bounded, serialize_into};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};

pub struct Packet {
    kind: u16,
    data: Vec<u8>,
}

impl<T: DeserializeOwned> TryInto<T> for Packet {
    type Error = ();
    fn try_into(self) -> Result<T, Self::Error> {
        unimplemented!()
    }
}

pub trait IntoPacket {
    fn into_packet(self, out: &mut Vec<u8>);
}

pub trait FromPacket: Sized {
    fn from_packet(packet: Vec<u8>) -> io::Result<Self>;
}

//impl<T: Serialize> IntoPacket for (u16, T) {
//    fn into_packet(self) -> Vec<u8> {
//        let (kind, p) = self;
//        let out = Vec::new();
//        out.write_u16(kind);
//        serialize_into(&mut out, &p, Bounded(65536)).unwrap();
//        out
//    }
//}

//impl FromPacket for (u16, Vec<u8>) {
//    fn from_packet(mut v: Vec<u8>) -> io::Result<Self> {
//        let r = io::Cursor::new(v);
//        let kind = r.read_u16()?;
//        Ok((kind, r.into_inner().split_off(r.position() as usize)))
//    }
//}

/* impl Split for Packet {
    type OutIter = Vec<Vec<u8>>;
    fn split(self, mtu: usize, limit: usize) -> Result<(Self::OutIter, usize), Self> {
        //let len = self.key.len() + self.payload.len();
        //if len > limit {
        //    return Err(self);
        //}
        //let mut key = self.key.into_bytes();
        //let mut payload = self.payload;
        //Ok((
        //    (0..(len / mtu))
        //        .map(move |i| (i * mtu, i + 1 * mtu))
        //        .map(move |(i, j)| {
        //            let mut out: Vec<u8> = Vec::with_capacity(mtu);
        //            if i < key.len() {
        //                let key_end = if j <= key.len() { j } else { key.len() };
        //                out.extend_from_slice(&mut key[i..key_end]);
        //            };
        //            if j > key.len() {
        //                let pay_start = i + key.len();
        //                let pay_end = if j + key.len() < payload.len() {
        //                    j + key.len()
        //                } else {
        //                    payload.len()
        //                };
        //                out.extend_from_slice(&mut payload[pay_start..pay_end]);
        //            }
        //            out
        //        })
        //        .collect(),
        //    len,
        //))
    }
}

impl FromSplit for Packet {} */

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
struct FrameHeader {
    pub id: u64,
    pub len: u16,
    pub size: u16,
}

struct Frame {
    pub header: FrameHeader,
    pub num: u8,
    pub payload: Vec<u8>,
}

//pub fn new(s: UdpSocket) -> impl Stream + Sink {
//    s.framed(ServerCodec).with_flat_map(|x| x)
//}

pub struct ServerCodec;
//pub struct ServerCodec {
//    partial_packets: LruCache<>
//}

pub fn new<Out: FromPacket, In: IntoPacket>(
    h: &Handle,
) -> io::Result<
    impl Stream<Item = Out, Error = io::Error> + Sink<SinkItem = In, SinkError = io::Error>,
> {
    let sock = UdpSocket::bind(&([239, 53, 38, 42u8], 5338u16).into(), &h)?;

    sock.set_multicast_loop_v4(true)?;
    sock.set_multicast_ttl_v4(1)?;
    sock.join_multicast_v4(
        &[239, 53, 38, 42u8].into(),
        &[0, 0, 0, 0].into(),
    )?;
    let join = PacketJoiner::new(50);
    let split = PacketSplitter::new(300);
    Ok(
        sock.framed(ServerCodec)
            .with_flat_map(move |p: In| split.process(p.into_packet()).unwrap())
            .map(move |(_a, p)| {
                Out::from_packet(join.process(p).unwrap()).unwrap()
            }),
    )
}

struct PacketJoiner {
    cache: LruCache<FrameHeader, HashMap<u8, Frame>>,
}
impl PacketJoiner {
    fn new(size: usize) -> Self {
        PacketJoiner { cache: LruCache::new(size) }
    }
    fn process(&mut self, f: Frame) -> Option<Vec<u8>> {
        let header = f.header;
        let m = if self.cache.contains_key(&header) {
            self.cache.insert(f.header, HashMap::new());
            self.cache.get_mut(&header).unwrap()
        } else {
            self.cache.get_mut(&header).unwrap()
        };
        m.insert(f.num, f);
        let count = ((header.len as u64 + header.size as u64 - 1) / header.size as u64) as u16;

        if m.len() as u16 >= header.len / header.size {
            let mut v = Vec::with_capacity(header.len as usize);
            for i in 0..count as u8 {
                match m.remove(&i) {
                    Some(f) => v.extend(f.payload),
                    None => return None,
                }
            }
            Some(v)
        } else {
            None
        }
    }
}
struct PacketSplitter {
    mtu: usize,
}
impl PacketSplitter {
    fn new(mtu: usize) -> Self {
        PacketSplitter { mtu: mtu - 16 }
    }
    fn process(&mut self, v: Vec<u8>) -> Result<Vec<Frame>, ()> {
        if v.len() > 65535 {
            return Err(());
        }
        let head = FrameHeader {
            len: v.len() as u16,
            size: self.mtu as u16,
            id: thread_rng().next_u64(),
        };
        Ok({
            v.chunks(self.mtu)
                .enumerate()
                .map(move |(i, x)| {
                    Frame {
                        header: head,
                        num: i as u8,
                        payload: x.to_vec(),
                    }
                })
                .collect()
        })
    }
}

impl UdpCodec for ServerCodec {
    type In = (SocketAddr, Frame);
    type Out = Frame;
    fn decode(&mut self, src: &SocketAddr, buf: &[u8]) -> io::Result<Self::In> {
        let mut c = io::Cursor::new(buf);
        let id = c.read_u64::<LE>()?;
        let len = c.read_u16::<LE>()?;
        let size = c.read_u16::<LE>()?;
        let num = c.read_u8()?;
        let payload = c.fill_buf()?;
        Ok((
            src.clone(),
            Frame {
                header: FrameHeader {
                    id: id,
                    len: len,
                    size: size,
                },
                num: num,
                payload: payload.to_vec(),
            },
        ))
    }
    fn encode(&mut self, mut msg: Self::Out, buf: &mut Vec<u8>) -> SocketAddr {
        buf.write_u64::<LE>(msg.header.id).unwrap();
        buf.write_u16::<LE>(msg.header.len).unwrap();
        buf.write_u16::<LE>(msg.header.size).unwrap();
        buf.write_u8(msg.num).unwrap();
        buf.extend(msg.payload);
        ([239, 53, 38, 42u8], 5338u16).into()
    }
}
