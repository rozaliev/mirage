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
use mirage_core::{context, Core};

fn main() {
    let mut core = Core::new();
    core.run(start());
}

#[async]
fn start() -> impl Async<()> {
    let addr = &"127.0.0.1:3333".parse().unwrap();
    let lst = TcpListener::bind(addr).unwrap();

    context().spawn(accept_conn(lst));

    await!(start_client())
}

#[async]
fn start_client() -> impl Async<()> {
    let addr = &"127.0.0.1:3333".parse().unwrap();
    let mut sock = await!(TcpStream::connect(addr)).unwrap();

    let mut buf = [0; 1024];

    loop {
        let n = await!(sock.read(&mut buf)).unwrap();
        if n == 0 {
            return;
        }
        println!("{}", ::std::str::from_utf8(&buf[..n]).unwrap());
    }
}

#[async]
fn accept_conn(lst: TcpListener) -> impl Async<()> {
    let (mut conn, _) = await!(lst.accept()).unwrap();
    for i in 0..50 {
        let s = format!("hello_{}", i);
        await!(conn.write_all(s.as_bytes())).unwrap();
    }
}
