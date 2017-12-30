#![feature(proc_macro)]
#![feature(generators)]
#![feature(conservative_impl_trait)]

#[macro_use]
extern crate mirage_async;
extern crate mirage_async_codegen;
extern crate mirage_core;

extern crate mio;
extern crate net2;

use std::net::SocketAddr;
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write};

use mirage_async::Async;
use mirage_async_codegen::async;

#[macro_use]
mod nb_macro;

mod sys {
    pub(crate) use mio::tcp::{TcpListener, TcpStream};
}

#[derive(Debug)]
pub struct TcpStream(sys::TcpStream);
#[derive(Debug)]
pub struct TcpListener(sys::TcpListener);

impl TcpListener {
    pub fn bind(addr: &SocketAddr) -> IoResult<TcpListener> {
        sys::TcpListener::bind(addr).map(TcpListener)
    }

    #[async]
    pub fn accept<'a>(&'a self) -> impl Async<IoResult<(TcpStream, SocketAddr)>> + 'a {
        let (s, a) = await_nb!(&self.0, self.0.accept(), Read)?;
        Ok((TcpStream(s), a))
    }
}

impl TcpStream {
    #[async]
    pub fn connect<'a>(addr: &'a SocketAddr) -> impl Async<IoResult<TcpStream>> + 'a {
        let socket = sys::TcpStream::connect(addr).map(TcpStream)?;
        ::mirage_core::context().register_write(&socket.0);
        yield;
        return Ok(socket);
    }

    #[async]
    pub fn read<'a, 'b>(&'a mut self, buf: &'b mut [u8]) -> impl Async<IoResult<usize>> + 'a
    where
        'b: 'a,
    {
        await_nb!(&self.0, self.0.read(buf), Read)
    }

    #[async]
    pub fn write<'a, 'b>(&'a mut self, buf: &'b [u8]) -> impl Async<IoResult<usize>> + 'a
    where
        'b: 'a,
    {
        await_nb!(&self.0, self.0.write(buf), Write)
    }

    #[async]
    pub fn write_all<'a, 'b>(&'a mut self, mut buf: &'b [u8]) -> impl Async<IoResult<()>> + 'a
    where
        'b: 'a,
    {
        while !buf.is_empty() {
            let r = await!(self.write(buf));
            match r {
                Ok(0) => {
                    return Err(IoError::new(
                        IoErrorKind::WriteZero,
                        "failed to write whole buffer",
                    ))
                }
                Ok(n) => buf = &buf[n..],
                Err(e) => {
                    if e.kind() == IoErrorKind::Interrupted {
                        continue;
                    }
                    return Err(e);
                }
            }
        }
        Ok(())
    }
}
