#![feature(conservative_impl_trait, try_from)]
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
