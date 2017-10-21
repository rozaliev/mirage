#[macro_export]
macro_rules! await {
    ($e:expr) => ({
        let mut g = $e;
        loop {
            match g.poll() {
                $crate::Await::NotReady => {},
                $crate::Await::Done(r) => break r,                    
           }
           yield;
        }
    })
}
