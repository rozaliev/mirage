

macro_rules! await_nb {
    ($e: expr) => {
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
            yield
        }
    };
}
