macro_rules! await_nb {
    ($fd:expr, $e:expr, Read) => (
        await_nb!($e, {
            ::mirage_core::context().register_read($fd);
        })
    );

    ($fd:expr, $e:expr, Write) => (
        await_nb!($e, {
            ::mirage_core::context().register_write($fd);
        })
    );

    ($fd:expr, $e:expr, All) => (
        await_nb!($e, {
            ::mirage_core::context().register_all($fd);
        })
    );

    ($e: expr, $p: expr) => {
        loop {
            {
                let nb = $e;
                match nb {
                    Ok(v) => { break Ok(v) },
                    Err(err) => {
                        if err.kind() != ::std::io::ErrorKind::WouldBlock {
                            break Err(err)
                        }
                    },
                }
            }
            $p
            yield
        }
    };
}
