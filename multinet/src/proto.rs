use std::collections::{HashMap, BTreeMap};
use std::result::Result as StdResult;
use std::net::SocketAddr;
use std::mem;
use std::io::{Read, Write, Cursor, Result as IoResult, Error as IoError, ErrorKind};

use tokio_core::net::UdpCodec;
use byteorder::{BE, ReadBytesExt, WriteBytesExt};

use errors::*;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub struct ChannelID(pub u64);

impl ChannelID {
    pub fn new(id: u64) -> Self {
        ChannelID(id)
    }
}

pub type Epoch = u64;
pub type MessageId = u64;

// A struct that can be sent over the network
trait Wire: Sized {
    fn decode(buf: &[u8]) -> IoResult<Self>;
    fn encode(self, buf: &mut Vec<u8>);
}

#[derive(Debug)]
pub enum WirePacket {
    Data(DataPacket),
}

impl Wire for WirePacket {
    fn decode(buf: &[u8]) -> IoResult<Self> {
        let mut c = Cursor::new(buf);
        use self::WirePacket::*;
        match c.read_u8()? {
            0 => Ok(Data(DataPacket::decode(&buf[c.position() as usize..])?)),
            _ => Err(IoError::new(
                ErrorKind::InvalidData,
                "Unrecoginzed Packet type",
            )),
        }
    }
    fn encode(self, buf: &mut Vec<u8>) {
        use self::WirePacket::*;
        match self {
            Data(packet) => {
                buf.write_u8(0).unwrap();
                packet.encode(buf);
            }
        }
    }
}

#[derive(Debug)]
pub struct DataPacket {
    pub channel: ChannelID,
    pub epoch: Epoch,
    pub msg_id: MessageId, // These can be shrunk later
    pub msg_seq: u32,
    pub msg_len: u32,
    pub data: Box<[u8]>,
}

impl Wire for DataPacket {
    fn decode(buf: &[u8]) -> IoResult<Self> {
        let mut c = Cursor::new(buf);
        let channel = ChannelID(c.read_u64::<BE>()?);
        let epoch = c.read_u64::<BE>()?;
        let msg_id = c.read_u64::<BE>()?;
        let msg_seq = c.read_u32::<BE>()?;
        let msg_len = c.read_u32::<BE>()?;
        let mut data = vec![];
        c.read_to_end(&mut data)?;
        Ok(Self {
            channel,
            epoch,
            msg_id,
            msg_seq,
            msg_len,
            data: data.into(),
        })
    }
    fn encode(self, buf: &mut Vec<u8>) {
        let ChannelID(channel) = self.channel;
        move || -> StdResult<_, _> {
            buf.write_u64::<BE>(channel)?;
            buf.write_u64::<BE>(self.epoch)?;
            buf.write_u64::<BE>(self.msg_id)?;
            buf.write_u32::<BE>(self.msg_seq)?;
            buf.write_u32::<BE>(self.msg_len)?;
            buf.write_all(&self.data)
        }().unwrap();
    }
}

#[derive(Debug)]
pub struct AssembledDataPacket {
    pub channel: ChannelID,
    pub epoch: Epoch,
    pub msg_id: MessageId, // These can be shrunk later
    pub data: Box<[u8]>,
}

impl AssembledDataPacket {
    pub fn new<T: Into<Box<[u8]>>>(data: T, channel: ChannelID, epoch: Epoch) -> Self {
        AssembledDataPacket {
            channel,
            epoch,
            msg_id: 42,
            data: data.into(),
        }
    }
}

#[derive(Debug)]
pub enum AssembledPacket {
    Data(AssembledDataPacket),
}

impl AssembledPacket {}

#[derive(Debug)]
pub struct PacketAssembler {
    data_cache: HashMap<(ChannelID, Epoch), BTreeMap<MessageId, DataPacketAssembly>>,
}

impl PacketAssembler {
    pub fn assemble(&mut self, packet: WirePacket) -> Result<Option<AssembledPacket>> {
        Ok(match packet {
            WirePacket::Data(packet) => {
                let mut m = self.data_cache
                    .entry((packet.channel, packet.epoch))
                    .or_insert_with(BTreeMap::new);
                m.entry(packet.msg_id)
                    .or_insert(DataPacketAssembly::new(packet.msg_len))
                    .add(packet)
                    .map(AssembledPacket::Data)
            }
        })
    }
    pub fn new() -> Self {
        Self { data_cache: HashMap::new() }
    }
}

#[derive(Debug)]
struct DataPacketAssembly {
    pub msg_len: u32,
    pub count: u32,
    pieces: Vec<Option<DataPacket>>,
}

impl DataPacketAssembly {
    fn new(msg_len: u32) -> Self {
        DataPacketAssembly {
            msg_len,
            count: 0,
            pieces: (0..msg_len).map(|_| None).collect(),
        }
    }
    fn add(&mut self, packet: DataPacket) -> Option<AssembledDataPacket> {
        if self.msg_len != packet.msg_len {
            println!("Packet with invalid len recieved: {:?}", packet);
            return None;
        }
        let seq = packet.msg_seq;
        let channel = packet.channel;
        let epoch = packet.epoch;
        let msg_id = packet.msg_id;
        match mem::replace(&mut self.pieces[seq as usize], Some(packet)) {
            None => self.count += 1,
            Some(_) => println!("DUPLICATE PACKET RECIEVED"),
        };
        let c = self.pieces
            .iter()
            .map(|x| x.as_ref().map(|_| 1).unwrap_or(0))
            .sum();
        //assert!(c == self.count);
        if c != self.count {
            //println!("ERROR MISCOUNT {} != {}", c, self.count);
        }
        self.count = c;
        //println!("Count: {:?}", self.count);
        if self.count == self.msg_len {
            let mut out = vec![];
            for mut p in self.pieces.iter_mut() {
                let p = p.take().unwrap();
                out.append(&mut p.data.into())
            }
            Some(AssembledDataPacket {
                channel,
                epoch,
                msg_id,
                data: out.into(),
            })
        } else {
            assert!(self.count < self.msg_len);
            None
        }
    }
}

#[derive(Debug)]
pub struct PacketSpliter {
    data_packet_splitter: DataPacketSplitter,
}

impl PacketSpliter {
    pub fn new() -> Self {
        Self { data_packet_splitter: DataPacketSplitter::new() }
    }
    pub fn split(&mut self, packet: AssembledPacket) -> Vec<WirePacket> {
        match packet {
            AssembledPacket::Data(packet) => self.data_packet_splitter.split(packet),
        }
    }
}

#[derive(Debug)]
struct DataPacketSplitter {
    mtu: usize,
}

impl DataPacketSplitter {
    fn new() -> Self {
        Self { mtu: 200 }
    }
    fn split(&mut self, packet: AssembledDataPacket) -> Vec<WirePacket> {
        let len = packet.data.len();
        let mut out = vec![];
        let msg_len = len / self.mtu + (if len % self.mtu > 0 { 1 } else { 0 }); // round up when dividing
        let msg_len = msg_len as u32;
        for (i, c) in packet.data.chunks(self.mtu).enumerate() {
            let p = WirePacket::Data(DataPacket {
                channel: packet.channel,
                epoch: packet.epoch,
                msg_id: packet.msg_id,
                msg_len,
                msg_seq: i as u32,
                data: c.to_owned().into_boxed_slice(),
            });
            out.push(p);
        }
        out
    }
}

pub struct WireProto;

impl UdpCodec for WireProto {
    type In = (WirePacket, SocketAddr);
    type Out = (WirePacket, SocketAddr);
    fn decode(&mut self, addr: &SocketAddr, buf: &[u8]) -> IoResult<Self::In> {
        //println!("Recieved packet from {:?}", addr);
        Ok((WirePacket::decode(buf)?, addr.clone()))
    }
    fn encode(
        &mut self,
        (packet, addr): (WirePacket, SocketAddr),
        buf: &mut Vec<u8>,
    ) -> SocketAddr {
        //println!("sending: {:?} to {:?}", packet, addr);
        packet.encode(buf);
        addr
    }
}
