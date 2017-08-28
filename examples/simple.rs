extern crate futures;
extern crate tokio_core;
extern crate multi_net;

use futures::{Future, Stream, Sink};
use tokio_core::reactor::Core;
use multi_net::{Server, ControlPacket, AssembledDataPacket, ChannelID};

fn main() {
    let reactor = Core::new().unwrap();

    let (s, handle) = Server::new(reactor.handle(), &[0, 0, 0, 0u8].into()).unwrap();
    println!("{:#?}", s);
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
    println!("{:#?}", Stream::wait(handle).next());
}
