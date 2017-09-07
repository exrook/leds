#[macro_use]
extern crate futures;
extern crate tokio_core;
extern crate multinet;

use futures::{Future, Stream, Sink, Async, future};
use tokio_core::reactor::Core;
use multinet::{Server, ControlPacket, AssembledDataPacket, ChannelID};

fn main() {
    let mut reactor = Core::new().unwrap();

    let (s, handle) = Server::new(reactor.handle(), &[0, 0, 0, 0u8].into()).unwrap();
    reactor.handle().spawn(s.map_err(|e| {
        println!("Error polling server: {:?}", e)
    }));

    let handle = handle
        .send(ControlPacket::SendData(AssembledDataPacket::new(
            "Hello World".to_owned().into_bytes(),
            ChannelID::new(5),
            5,
        )))
        .wait()
        .unwrap();
    println!("LUL");
    reactor.run(handle.for_each(|p| {
        println!("!!!Recieved: {:?}", p);
        Ok(())
    }));
}
