#![feature(proc_macro)]
#![feature(generators)]
#![feature(conservative_impl_trait)]

extern crate mirage_async;
extern crate mirage_async_codegen;
extern crate net2;


use std::net::{SocketAddr, ToSocketAddrs};
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write};

use net2::{TcpBuilder, TcpStreamExt};

use mirage_async::Async;
use mirage_async_codegen::async;

#[macro_use]
mod nb_macro;

mod sys {
    pub(crate) use std::net::{TcpListener, TcpStream};
}


pub struct TcpStream(sys::TcpStream);
pub struct TcpListener(sys::TcpListener);


impl TcpListener {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> IoResult<TcpListener> {
        sys::TcpListener::bind(addr).map(|l| {
            l.set_nonblocking(true);
            TcpListener(l)
        })
    }

    #[async]
    pub fn accept<'a>(&'a self) -> impl Async<IoResult<(TcpStream, SocketAddr)>> + 'a {
        let (s, a) = await_nb!(self.0.accept())?;
        Ok((TcpStream(s), a))
    }
}

impl TcpStream {
    #[async]
    pub fn connect<'a>(addr: &'a SocketAddr) -> impl Async<IoResult<TcpStream>> + 'a {
        let sock = match *addr {
            SocketAddr::V4(..) => TcpBuilder::new_v4(),
            SocketAddr::V6(..) => TcpBuilder::new_v6(),
        }?.to_tcp_stream()?;

        // sock.set_nonblocking(true)?;
        await_nb!(sock.connect(addr))?;
        println!("yo");

        Ok(TcpStream(sock))
    }


    #[async]
    pub fn read<'a, 'b>(&'a mut self, buf: &'b mut [u8]) -> impl Async<IoResult<usize>> + 'a
    where
        'b: 'a,
    {
        await_nb!(self.0.read(buf))
    }

    #[async]
    pub fn write<'a, 'b>(&'a mut self, buf: &'b [u8]) -> impl Async<IoResult<usize>> + 'a
    where
        'b: 'a,
    {
        await_nb!(self.0.write(buf))
    }
}
