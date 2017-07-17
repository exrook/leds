extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate rand;

use futures::{Future,Stream,Poll,Async,task};
use futures::task::Task;
use std::time::Duration;
use std::sync::{Arc, Mutex, TryLockError};
use std::sync::atomic::{AtomicUsize,AtomicBool,Ordering};
use std::thread;
use std::panic;
use std::mem;
use rand::{Rng,thread_rng};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

struct TestFuture {
    count: Arc<AtomicUsize>,
    thread: Option<thread::JoinHandle<()>>
}

impl TestFuture {
    fn new() -> Self {
        Self {
            count: Arc::new(AtomicUsize::new(42)),
            thread: None
        }
    }
}

impl Future for TestFuture {
    type Item = usize;
    type Error = ();
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        println!("POLLED");
        match self.thread {
            None => {
                let task = task::current();
                let count = self.count.clone();
                self.thread = Some(thread::spawn(move || {
                    loop {
                        thread::sleep(Duration::from_millis(300));
                        if count.fetch_add(1, Ordering::AcqRel) >= 42 {
                            break
                        }
                        if thread_rng().gen_weighted_bool(10) {
                            task.notify()
                        }
                    }
                    task.notify();
                }));
            }
            Some(_) => {}
        }
        if self.count.load(Ordering::Acquire) >= 42 {
            Ok(Async::Ready(42))
        } else {
            Ok(Async::NotReady)
        }
    }
}

#[cfg(test)]
#[test]
fn test_test_future() {
    let test = TestFuture::new();
    println!("{:?}", test.wait());
    println!("HELLO");
}

#[derive(Debug)]
struct TestStream {

}

impl TestStream {
    fn new() -> Self {
        Self {}
    }
}

impl Stream for TestStream {
    type Item = ();
    type Error = ();
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        println!("Stream polled");
        Ok(Async::NotReady)
    }
}

#[cfg(test)]
#[test]
fn test_test_stream() {
    let test = TestStream::new();
    let mut wait = test.wait();
    println!("{:?}", wait);
    println!("{:?}", wait.next());
    println!("HELLO");
}
