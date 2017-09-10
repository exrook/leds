extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tokio_core;
#[macro_use]
extern crate error_chain;

extern crate atomic_box;

extern crate bincode;

extern crate multinet;
extern crate led_control;
#[macro_use]
extern crate futures;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::net::Ipv4Addr;

use tokio_core::reactor::Handle;
use futures::{Stream, Sink, Future, future, Poll, Async, AsyncSink};
use futures::task::AtomicTask;

use atomic_box::AtomicBox;

use bincode::Infinite;

use led_control::Pixel;

use multinet::{Server, ServerHandle, ControlPacket, AssembledDataPacket, RecievedPacket, ChannelID};

pub mod errors {
    error_chain!{}
}
use errors::*;

mod message;
pub use message::Message;

pub struct LedServer {
    data: Arc<AtomicBox<Vec<Pixel>>>,
    task: Arc<AtomicTask>,
}

impl LedServer {
    pub fn new<T: AsRef<[Ipv4Addr]>>(handle: &Handle, addrs: T) -> Result<Self> {
        let (s, h) = Server::new(handle.clone(), addrs).chain_err(
            || "Error creating multinet server",
        )?;
        let data = Arc::new(AtomicBox::new(vec![]));
        let task = Arc::new(AtomicTask::new());
        let (mut sink, stream) = h.split();
        handle.spawn(s.map_err(|_| ()));
        handle.spawn(stream.map_err(|_| panic!()).for_each({
            let data_box = data.clone();
            move |p| {
                //println!("Recieved: {:?}", p);
                //println!("\n\n\n\n\n\n");
                match p {
                    RecievedPacket::Data(data) => {
                        match data.channel {
                            ChannelID(42) => {
                                match bincode::deserialize(&data.data) {
                                    Ok(msg) => {
                                        let msg: Message = msg;
                                        //println!("Message: {:?}", msg);
                                        data_box.store(msg.pixels)
                                    }
                                    Err(_) => {}
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Ok(())
            }
        }));
        handle.spawn({
            let task = task.clone();
            let data = data.clone();
            future::lazy(move || {
                task.register();
                future::poll_fn(move || {
                    try_ready!(sink.poll_complete().map_err(|_| ()));
                    let msg = Message { pixels: (*data.load()).to_owned() };
                    let msg_bytes = bincode::serialize(&msg, Infinite).unwrap();
                    match sink.start_send(ControlPacket::SendData(
                        AssembledDataPacket::new(msg_bytes, ChannelID::new(42), 34),
                    )).map_err(|_| ())? {
                        AsyncSink::Ready => {}
                        AsyncSink::NotReady(_) => return Ok(Async::NotReady),
                    };

                    Ok(Async::NotReady)
                })
            })
        });
        Ok(Self {
            data,
            task: task,
            //cond: AtomicBool::new(false),
        })
    }
    pub fn load(&self) -> Arc<Vec<Pixel>> {
        self.data.load()
    }
    pub fn store(&self, pixels: Vec<Pixel>) {
        self.data.store(pixels);
        self.task.notify();
        //self.cond.store(true, Ordering::Release);
    }
}
