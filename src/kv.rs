use std::sync::Arc;
use std::rc::Rc;
use std::collections::HashMap;
use std::any::Any;
use std::io::Result as IoResult;

use futures::{Stream, Future, Sink};
use futures::sync::mpsc;
use tokio_core::reactor::Handle;

use protocol::{self, IntoPacket, FromPacket, Packet};

use atomic_box::AtomicBox;

use serde::Serialize;
use serde::de::DeserializeOwned;
use bincode::{deserialize, serialize, Error as BincodeError, Infinite};
use untrusted::{Input, Reader};

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

pub struct KVPacket {
    pub key: String,
    value: Vec<u8>,
}

impl FromPacket for KVPacket {
    fn from_packet(packet: Vec<u8>) -> IoResult<Self> {
        let mut reader = Reader::new(Input::from(&packet));
        let str_len = reader.read_byte()?;
        let key = reader.read_utf8(str_len)?.to_owned();
        //let data = reader.read_bytes(
    }
}

impl IntoPacket for KVPacket {
    fn into_packet(self, out: &mut Vec<u8>) {
        unimplemented!()
    }
}

pub struct Server {
    registrations: HashMap<String, Box<Recieve + Send>>,
}

#[derive(Clone)]
pub struct SendHandle<In: IntoPacket> {
    channel: mpsc::Sender<In>,
}

impl<In: IntoPacket> SendHandle<In> {
    pub fn send<S: Into<String>, T: Serialize>(
        &self,
        key: S,
        msg: T,
    ) -> Result<impl Future<Item = (), Error = ()>, BincodeError> {
        match serialize(&msg, Infinite) {
            Err(e) => Err(e),
            Ok(v) => Ok(self.channel.clone().send().map_err(|_| ()).map(|_| ())),
        }
    }
    pub fn send_sync<S: Into<String>, T: Serialize>(
        &self,
        key: S,
        msg: T,
    ) -> Result<Any, Result<(), BincodeError>> {
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
    reg_recv: mpsc::Receiver<(String, Box<Recieve + Send + 'static>)>,
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

        //let (framed_send, framed_recv) = sock.framed(ServerCodec).split();
        let (framed_send, framed_recv) = protocol::new::<KVPacket, KVPacket>(&h)?.split();

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
                        match decode(p) {
                            PacketKind::KV(p) => {
                                println!("Processing Packet");
                                match server.recieve(p) {
                                    Err(e) => println!("Error processing packet: {:?}", e),
                                    _ => {}
                                };
                            }
                        }
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
        std::thread::sleep(std::time::Duration::new(0, 1000));
        println!("{:?}", b);
    }
}
