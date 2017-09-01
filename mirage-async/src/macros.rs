#[macro_export]
macro_rules! await {
    ($e:expr) => ({
        let mut g = $e;
        loop {
            match g.poll() {
                $crate::base::Await::NotReady => {},
                $crate::base::Await::Done(r) => break r,                    
           }
           yield;
        }
    })
}
