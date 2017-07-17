#![feature(conservative_impl_trait)]
extern crate futures;
extern crate tokio_core;
extern crate atomic_box;
extern crate serde;
extern crate bincode;

use std::rc::Rc;
use std::sync::Arc;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::io::Result as IoResult;
use std::io::{Error, ErrorKind};

use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use bincode::{deserialize, serialize, Infinite};
use bincode::internal::Error as BincodeError;

pub use atomic_box::AtomicBox;

use futures::{Stream, Sink, Future};
use futures::sync::mpsc;
use futures::stream::MergedItem;
use tokio_core::net::{UdpCodec, UdpSocket};
use tokio_core::reactor::Handle;

pub trait Recieve {
    fn recieve(&mut self, buf: &[u8]) -> Result<(), String>;
}

impl<T> Recieve for Box<T>
where
    T: Recieve,
{
    fn recieve(&mut self, buf: &[u8]) -> Result<(), String> {
        (**self).recieve(buf)
    }
}
impl<T> Recieve for Arc<T>
where
    for<'a> &'a T: Recieve,
{
    fn recieve(&mut self, buf: &[u8]) -> Result<(), String> {
        (&**self).recieve(buf)
    }
}
impl<T> Recieve for Rc<T>
where
    for<'a> &'a T: Recieve,
{
    fn recieve(&mut self, buf: &[u8]) -> Result<(), String> {
        (&**self).recieve(buf)
    }
}

impl<'a, T> Recieve for &'a AtomicBox<T>
where
    T: DeserializeOwned,
{
    fn recieve(&mut self, buf: &[u8]) -> Result<(), String> {
        println!("WHAT'S GOING ON ");
        println!("LEN: {:?}", buf.len());
        match deserialize(buf) {
            Ok(v) => {
                println!("GOTTEM");
                self.store(v);
                Ok(())
            }
            Err(e) => {
                println!("NOGOTTEM: {:?}", e);
                Err(format!("NOGOTTEM: {:?}", e))
            }
        }
    }
}

pub struct Server {
    registrations: HashMap<String, Box<Recieve + Send>>,
}

pub struct Packet {
    key: String,
    payload: Vec<u8>,
}


#[derive(Clone)]
pub struct SendHandle {
    channel: mpsc::Sender<Packet>,
}

impl SendHandle {
    pub fn send<S: Into<String>, T: Serialize>(
        &self,
        key: S,
        msg: T,
    ) -> Result<impl Future<Item = (), Error = ()>, BincodeError> {
        match serialize(&msg, Infinite) {
            Err(e) => Err(e),
            Ok(v) => {
                Ok(
                    self.channel
                        .clone()
                        .send(Packet {
                            key: key.into(),
                            payload: v,
                        })
                        .map_err(|_| ())
                        .map(|_| ()),
                )
            }
        }
    }
    pub fn send_sync<S: Into<String>, T: Serialize>(
        &self,
        key: S,
        msg: T,
    ) -> Result<impl std::any::Any, Result<(), BincodeError>> {
        self.send(key, msg).map_err(|e| Err(e))?.wait().map_err(
            |e| Ok(e),
        )
    }
}

#[derive(Clone)]
pub struct ServerHandle {
    channel: mpsc::Sender<(String, Box<Recieve + Send>)>,
}
impl ServerHandle {
    pub fn register<T: Recieve + Send + 'static, S: Into<String>>(
        &self,
        key: S,
        r: T,
    ) -> impl Future<Item = (), Error = Box<Recieve + Send>> {
        self.channel
            .clone()
            .send((key.into(), Box::new(r) as Box<Recieve + Send + 'static>))
            .map(|_| ())
            .map_err(|e| e.into_inner().1)
    }
    pub fn register_sync<T: Recieve + Send + 'static, S: Into<String>>(
        &self,
        key: S,
        r: T,
    ) -> Result<(), Box<Recieve + Send + 'static>> {
        self.register(key, r).wait()
    }
}

pub struct ServerBuilder {
    reg_recv: mpsc::Receiver<
        (std::string::String,
         std::boxed::Box<Recieve + std::marker::Send + 'static>),
    >,
    msg_recv: mpsc::Receiver<Packet>,
}

impl ServerBuilder {
    pub fn new() -> (ServerBuilder, ServerHandle, SendHandle) {
        let (reg_send, reg_recv) = mpsc::channel(10);
        let (msg_send, msg_recv) = mpsc::channel(10);
        (
            Self {
                reg_recv: reg_recv,
                msg_recv: msg_recv,
            },
            ServerHandle { channel: reg_send },
            SendHandle { channel: msg_send },
        )
    }
    pub fn spawn(self, h: Handle) -> IoResult<()> {
        let mut server = Server { registrations: HashMap::new() };
        let sock = UdpSocket::bind(&([239, 53, 38, 42u8], 5338u16).into(), &h)?;

        sock.set_multicast_loop_v4(true)?;
        sock.set_multicast_ttl_v4(1)?;
        sock.join_multicast_v4(
            &[239, 53, 38, 42u8].into(),
            &[0, 0, 0, 0].into(),
        )?;

        let (framed_send, framed_recv) = sock.framed(ServerCodec).split();

        h.spawn(
            framed_recv
                .merge(self.reg_recv.map_err(|_| unreachable!()))
                .for_each(move |e| {
                    use MergedItem::*;
                    let (a, b) = match e {
                        First(a) => (Some(a), None),
                        Second(b) => (None, Some(b)),
                        Both(a, b) => (Some(a), Some(b)),
                    };
                    if let Some((key, r)) = b {
                        println!("Processing Registration");
                        server.registrations.insert(key, r);
                    }
                    if let Some(p) = a {
                        println!("Processing Packet");
                        match server.recieve(p) {
                            Err(e) => println!("Error processing packet: {:?}", e),
                            _ => {}
                        };
                    }
                    Ok(())
                })
                .map_err(|e| println!("Err: {:?}", e)),
        );

        h.spawn(
            self.msg_recv
                .forward(framed_send.sink_map_err(|e| println!("Error: {:?}", e)))
                .map(|_| ()),
        );

        Ok(())
    }
}
impl Server {
    fn recieve(&mut self, p: Packet) -> Result<(), String> {
        println!("Looking for \"{:?}\"", p.key);
        match self.registrations.get_mut(&p.key) {
            Some(ref mut r) => {
                println!("Found a match, dispatching");
                r.recieve(&p.payload)?;
            }
            None => {
                println!("No match found");
            }
        }
                println!("{:?}", self.registrations.keys());
        Ok(())
    }
}

struct ServerCodec;

impl UdpCodec for ServerCodec {
    type In = Packet;
    type Out = Packet;
    fn decode(&mut self, _src: &SocketAddr, buf: &[u8]) -> IoResult<Self::In> {
        if buf.len() < 1 {
            return Err(Error::from(ErrorKind::InvalidData));
        }
        let key_len = buf[0] as usize;
        let key = std::str::from_utf8(&buf[1..1+key_len])
            .map_err(|_| Error::from(ErrorKind::InvalidData))?
            .to_owned();
        let p = Packet {
            key: key,
            payload: buf[1+key_len..].to_owned(),
        };
        Ok(p)
    }
    fn encode(&mut self, mut msg: Self::Out, buf: &mut Vec<u8>) -> SocketAddr {
        assert!(msg.key.len() < 255); // TODO: figure out how to signal an error instead of panicing
        buf.push(msg.key.len() as u8);
        buf.append(&mut msg.key.into_bytes());
        buf.append(&mut msg.payload);
        ([239, 53, 38, 42u8], 5338u16).into()
    }
}

#[cfg(test)]
#[test]
fn test_server() {
    let (mut s, reg, send) = ServerBuilder::new();
    let b = Arc::new(AtomicBox::new(vec![0u8; 10]));
    reg.register_sync("mybox", b.clone());
    let l = std::thread::spawn(move || {
        let mut c = tokio_core::reactor::Core::new().unwrap();
        println!("Spawning");
        s.spawn(c.handle()).unwrap();
        loop {
            c.turn(None)
        }
    });
    println!("{:?}", b);
    send.send_sync("mybox", vec![1u8; 10]).unwrap();
    send.send_sync("mybox", vec![2u8; 10]).unwrap();
    for _ in 0..100 {
        std::thread::sleep(std::time::Duration::new(0,1000));
        println!("{:?}", b);
    }
}
