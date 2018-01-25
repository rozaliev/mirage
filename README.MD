# Mirage

Mirage is a set of libraries for building async applications. It provides
async/await syntax and a networking library similar to std::net.

It's a PoC that might grow into something bigger one day.

## Goals

The main goal is to provide a fast and ergonomic async API that looks almost
like a blocking one. It should be easy to write both low level protocols implementations
and high level code.

## API

```
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
```

This example is very similar to a blocking code, the differences are:
* uses `[async]` to mark async functions
* returns `-> impl Async<ReturnType>` instead of `-> Return`
* uses `await!(async_func())` to get result of a async function

This code is explicit about "resume" points, but in everything else it looks
like `std::net`. You can use stack-allocated buffers, you don't need to `move` anything
inside `.read` method, just pass a reference and you're good. Error handling with `?` work
out of the box, though this example does a poor job presenting it.

Take a look at a working example in `src/bin/simple.rs`

## How does it work

Mirage is built on top of generators (stackless coroutines). Generators allow
us to write both ergonomic and performant code. Mirage uses shared nothing approach,
meaning that there is no syncronization inside the library. For communication threads
should use various kinds of queues. Mirage's `Core` is a concurrency primitive, and
should not be used to archive parallelism.


## How is it different from other libraries

Unlike stackful coroutines it does not require a stack per coroutine and does not
use any syncronization.

Unlike futures it does not cause `then(|_| ).then(|_| )` spaghetti, with Mirage
you'll end up with much more readable and understandable code.

Unlike futures-rs/tokio-rs ecosystem Mirage is more ergonomic and does not use
any syncronization in scheduler.


## How to use

Mirage can be only built on nightly >= `rustc 1.25.0-nightly (a0dcecff9 2018-01-24)`


## What's next

* `select! {}`
* timers
* simple http server
* benchmarks
* more examples