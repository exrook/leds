#![feature(conservative_impl_trait, catch_expr)]
#![deny(missing_debug_implementations)]
#[macro_use]
extern crate futures;
extern crate tokio_core;
extern crate net2;
extern crate lru_cache;
extern crate byteorder;
#[macro_use]
extern crate error_chain;

use std::result::Result as StdResult;
use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};
use std::collections::VecDeque;

use futures::{Future, Poll, Stream, Sink, StartSend, Async, AsyncSink};
use futures::sync::mpsc::{unbounded, UnboundedSender, UnboundedReceiver, SendError};

use tokio_core::reactor;
use tokio_core::net::{UdpFramed, UdpSocket};

use net2::UdpBuilder;

pub mod errors {
    error_chain!{}
}

mod proto;

use errors::*;
use proto::{WirePacket, WireProto, PacketAssembler, PacketSpliter};

pub use proto::{ChannelID, Epoch, AssembledPacket, AssembledDataPacket};

#[derive(Debug)]
pub enum ControlPacket {
    SendData(AssembledDataPacket),
}
#[derive(Debug)]
pub enum RecievedPacket {
    Data(AssembledDataPacket),
}

#[derive(Debug)]
pub struct ServerHandle {
    recv: UnboundedReceiver<RecievedPacket>,
    send: UnboundedSender<ControlPacket>,
}

impl Stream for ServerHandle {
    type Item = RecievedPacket;
    type Error = ();
    fn poll(&mut self) -> Poll<Option<RecievedPacket>, ()> {
        self.recv.poll()
    }
}

impl Sink for ServerHandle {
    type SinkItem = ControlPacket;
    type SinkError = SendError<ControlPacket>;

    fn start_send(
        &mut self,
        msg: ControlPacket,
    ) -> StartSend<ControlPacket, SendError<ControlPacket>> {
        self.send.start_send(msg)
    }
    fn poll_complete(&mut self) -> Poll<(), SendError<ControlPacket>> {
        self.send.poll_complete()
    }
    fn close(&mut self) -> Poll<(), SendError<ControlPacket>> {
        self.send.close()
    }
}

pub struct Server {
    addr: SocketAddr,
    group_addr: SocketAddrV4,
    recv: UnboundedReceiver<ControlPacket>,
    send: UnboundedSender<RecievedPacket>,
    handle: reactor::Handle,
    assembler: PacketAssembler,
    splitter: PacketSpliter,
    sock: UdpFramed<WireProto>,
    send_buf: VecDeque<AssembledPacket>,
    split_send_buf: VecDeque<WirePacket>,
    send_buf_buf: Option<WirePacket>,
}

impl std::fmt::Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> StdResult<(), std::fmt::Error> {
        f.debug_struct("Server")
            .field("addr", &self.addr)
            .field("recv", &self.recv)
            .field("send", &self.send)
            .field("handle", &self.handle)
            .field("assembler", &self.assembler)
            .field("splitter", &self.splitter)
            .field("sock", self.sock.get_ref())
            .field("send_buf", &self.send_buf)
            .field("split_send_buf", &self.split_send_buf)
            .field("send_buf_buf", &self.send_buf_buf)
            .finish()
    }
}

impl Server {
    pub fn new(handle: reactor::Handle, addr: &Ipv4Addr) -> Result<(Self, ServerHandle)> {
        let addr = addr.clone();
        let sock_addr = (addr, 5338u16).into();
        let group_addr: SocketAddrV4 = SocketAddrV4::new([239, 53, 38, 42u8].into(), 5338);
        //let group_addr: SocketAddrV4 = SocketAddrV4::new([80, 53, 38, 42u8].into(), 5338);
        let (client_send, server_recv) = unbounded();
        // TODO: Find out why the first packet sent is ignored
        UnboundedSender::send(
            &client_send,
            ControlPacket::SendData(AssembledDataPacket {
                channel: ChannelID::new(54238503),
                data: vec![0].into(),
                epoch: 0,
                msg_id: 0,
            }),
        ).chain_err(|| "Error injecting first packet")?;
        let (server_send, client_recv) = unbounded();

        let sock_builder = UdpBuilder::new_v4().chain_err(
            || "Unable to create UdpBuilder",
        )?;
        sock_builder.reuse_address(true).chain_err(
            || "Unable to set SO_REUSEADDR",
        )?;
        let sock = UdpSocket::from_socket(
            sock_builder.bind(&sock_addr).chain_err(|| {
                format!("Error binding socket to {}", addr)
            })?,
            &handle,
        ).chain_err(|| "Unable to create Tokio socket")?;
        sock.set_multicast_loop_v4(true).chain_err(
            || "Unable to set multicast_loop_v4",
        )?;
        sock.join_multicast_v4(group_addr.ip(), &addr).chain_err(
            || "Error joining multicast group",
        )?;
        Ok((
            Self {
                addr: sock_addr,
                group_addr,
                recv: server_recv,
                send: server_send,
                handle,
                assembler: PacketAssembler::new(),
                splitter: PacketSpliter::new(),
                sock: sock.framed(WireProto),
                send_buf: VecDeque::new(),
                split_send_buf: VecDeque::new(),
                send_buf_buf: None,
            },
            ServerHandle {
                recv: client_recv,
                send: client_send,
            },
        ))
    }
    fn process_incoming(&mut self) -> Poll<(), Error> {
        loop {
            let p = match try_ready!(self.sock.poll().chain_err(|| "Error polling Socket")) {
                None => return Ok(Async::NotReady),
                Some(p) => p,
            };
            match self.assembler.assemble(p.0)? {
                Some(AssembledPacket::Data(p)) => {
                    UnboundedSender::send(&self.send, RecievedPacket::Data(p))
                        .chain_err(|| "Error sending packet to channel")?;
                }
                None => {}
            }
        }
    }
    fn process_channels(&mut self) -> Poll<(), Error> {
        loop {
            let p = match try_ready!(self.recv.poll().map_err(|()| "Error polling channel")) {
                None => return Ok(Async::NotReady),
                Some(p) => p,
            };
            match p {
                ControlPacket::SendData(data) => {
                    self.send_buf.push_back(AssembledPacket::Data(data));
                }
            }
        }
    }
    fn process_outgoing(&mut self) -> Poll<(), Error> {
        self.send.poll_complete().chain_err(
            || "Error polling socket",
        )?;
        loop {
            while let Some(p) = self.split_send_buf.pop_front() {
                match self.sock
                    .start_send((p, self.group_addr.into()))
                    .chain_err(|| "Error sending packet to socket")? {
                    AsyncSink::NotReady(p) => {
                        self.split_send_buf.push_front(p.0);
                        return Ok(Async::NotReady);
                    }
                    AsyncSink::Ready => {}
                }
            }
            if let Some(p) = self.send_buf.pop_back() {
                self.split_send_buf.extend(self.splitter.split(p));
            } else {
                return Ok(Async::NotReady);
            }
        }
    }
}

impl Future for Server {
    type Error = Error;
    type Item = ();
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        //println!("Polled");
        self.process_incoming().unwrap();
        self.process_channels().unwrap();
        self.process_outgoing().unwrap();
        Ok(Async::NotReady)
    }
}
