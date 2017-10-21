#![feature(conservative_impl_trait)]
#![feature(proc_macro)]
#![feature(generators)]

#[macro_use]
extern crate mirage_async;
extern crate mirage_async_codegen;
extern crate mirage_core;
extern crate mirage_net;

use mirage_async::Async;
use mirage_async_codegen::async;
use mirage_net::{TcpListener, TcpStream};
use mirage_core::Core;

fn main() {
    let mut core = Core::new();
    core.run(start());
}

#[async]
fn start() -> impl Async<()> {
    let addr = &"127.0.0.1:3333".parse().unwrap();
    let mut sock = await!(TcpStream::connect(addr)).unwrap();

    let mut buf = [0; 1024];

    loop {
        let n = await!(sock.read(&mut buf)).unwrap();
        println!("{}", ::std::str::from_utf8(&buf[..n]).unwrap());
    }
}
