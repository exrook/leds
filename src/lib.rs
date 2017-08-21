#![feature(conservative_impl_trait, try_from)]
#[macro_use]
extern crate futures;
extern crate tokio_core;
extern crate atomic_box;
extern crate serde;
extern crate bincode;
extern crate lru_cache;
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate byteorder;
extern crate untrusted;
extern crate untrustended;
#[macro_use]
extern crate error_chain;
extern crate set_neopixels;

use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::net::SocketAddr;
use std::io::Result as IoResult;
use std::collections::HashMap;

use futures::sync::mpsc::*;
use futures::{Future, Stream, Sink, Poll, Async};

use tokio_core::reactor::{Handle, Core};
use tokio_core::net::{UdpSocket, UdpFramed, UdpCodec};

use untrusted::{Reader, Input};
use untrustended::{ReaderExt, Error as UntrustendedError};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use atomic_box::AtomicBox;

mod errors {
    error_chain!{}
}

use errors::*;

// All values are u64 until wrapping can be figured out

struct Packet {
    epoch: u64,
    seq: u64,
    kind: PacketKind,
}

enum PacketKind {
    Ack(AckPacket),
    Pixel(PixelPacket),
}

#[derive(Serialize)]
struct AckPacket {
    seq: u64,
    epoch: u64,
    set: u64,
}

#[derive(Serialize)]
struct PixelPacket {
    screen_id: u8,
    start: u64,
    buf: Box<u8>,
}

struct RawPacket {
    epoch: u64,
    seq: u64,
    kind: u64,
    data: Box<u8>,
}

struct Codec {}

impl UdpCodec for Codec {
    type In = (SocketAddr, Packet);
    type Out = (SocketAddr, Packet);
    fn decode(&mut self, src: &SocketAddr, buf: &[u8]) -> IoResult<Self::In> {
        let mut r = Reader::new(Input::from(buf));
        r.read_u8();
        unimplemented!()
    }
    fn encode(&mut self, (addr, pack): Self::Out, buf: &mut Vec<u8>) -> SocketAddr {
        unimplemented!()
    }
}

struct ServerBuilder {}

impl ServerBuilder {
    fn new(h: Handle) -> Self {
        Self {}
    }
    fn run(self) -> Result<ServerHandle> {
        let t = thread::spawn(move || {
            Server::new()?;
            Ok(())
        });
        Ok(ServerHandle { thread: t })
    }
}

trait Watchable {}
impl<T> Watchable for T
where
    T: DeserializeOwned + Serialize,
{
}

struct Watched {
    item: AtomicBox<Watchable>,
}

struct ServerHandle {
    thread: JoinHandle<Result<()>>,
}

struct Server {
    framed: UdpFramed<Codec>,
    watched_vals: HashMap<String, Watched>,
}

impl Server {
    fn new() -> Result<()> {
        let mut core = Core::new().chain_err(|| "Error starting core")?;
        let h = core.handle();
        let sock = UdpSocket::bind(&([239, 53, 38, 0], 5338).into(), &h)
            .chain_err(|| "Error binding socket")?;

        sock.set_multicast_loop_v4(true).chain_err(
            || "Unable to set multicast loop setting",
        )?;
        sock.set_multicast_ttl_v4(1).chain_err(
            || "Unable to set multicast ttl",
        )?;
        sock.join_multicast_v4(&[239, 53, 38, 0].into(), &[0, 0, 0, 0].into())
            .chain_err(|| "Unable to join multicast socket")?;
        let s = Server {
            framed: sock.framed(Codec {}),
            watched_vals: HashMap::new(),
        };
        core.run(s)
    }
}

impl Future for Server {
    type Item = ();
    type Error = Error;
    fn poll(&mut self) -> Poll<(), Error> {
        match self.framed.poll().chain_err(|| "Error recieving packet")? {
            Async::Ready(p) => {}
            Async::NotReady => {}
        }
        self.framed.poll_complete().chain_err(
            || "Error sending packets",
        )?;
        Ok(Async::NotReady)
    }
}
