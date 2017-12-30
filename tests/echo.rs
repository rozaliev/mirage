#![feature(conservative_impl_trait)]
#![feature(proc_macro)]
#![feature(generators)]
#![feature(immovable_types)]

#[macro_use]
extern crate mirage_async;
extern crate mirage_async_codegen;
extern crate mirage_core;
extern crate mirage_net;

use mirage_async::Async;
use mirage_async_codegen::async;
use mirage_net::{TcpListener, TcpStream};
use mirage_core::{context, Core};

use std::io::Result as IoResult;

#[test]
fn echo() {
    let mut core = Core::new();

    core.run(echo_main()).unwrap()
}

#[async]
fn echo_main() -> impl Async<IoResult<()>> {
    let addr = "127.0.0.1:8888".parse().unwrap();
    let listener = TcpListener::bind(&addr)?;
    println!("after bind");

    context().spawn(echo_client());
    println!("after spawn client");

    let (conn, _) = await!(listener.accept())?;
    println!("after client accept");
    await!(handle_conn(conn))
}

#[async]
fn handle_conn(mut conn: TcpStream) -> impl Async<IoResult<()>> {
    println!("handling conn: {:?}", conn);
    let mut buf = [0; 1024];
    loop {
        let n = await!(conn.read(&mut buf))?;
        if n == 0 {
            return Ok(());
        }
        await!(conn.write_all(&buf[0..n]))?;
    }
}

#[async]
fn echo_client() -> impl Async<()> {
    let addr = "127.0.0.1:8888".parse().unwrap();
    let mut conn = await!(TcpStream::connect(&addr)).unwrap();
    println!("connected");
    for i in 0..9 {
        let s = format!("hello_{}", i);
        println!("before write {:?}", conn);
        await!(conn.write_all(s.as_bytes())).unwrap();
        println!("written {}", i);

        let mut buf = [0; 7];
        let mut read = 0;
        loop {
            let n = await!(conn.read(&mut buf[read..])).unwrap();
            println!("{:?}", buf);
            if n == 0 {
                panic!("eof");
            }

            read += n;
            if read == 7 {
                assert_eq!(buf, s.as_bytes());
                break;
            }
        }
    }
}
