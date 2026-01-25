pub mod latency;
pub mod limiter;
use limiter::inflight;



pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread, time::Duration};

    use crate::limiter::inflight::Inflight;

    use super::*;
    
    #[test]
    fn it_works() {
       let  inflight = Arc::new(Inflight::new());

        let handles = (0..10).map(|_|{
            
            let inflight = Arc::clone(&inflight);
            thread::spawn(move||{
                let guard = inflight.acquire();

                println!("Processing count:{}" , inflight.load());

                thread::sleep(Duration::from_millis(100));
            })
        }).collect::<Vec<_>>();

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
