use std::{sync::atomic::{AtomicU64 , Ordering}, time::Duration};

pub struct Ewma{
    value : AtomicU64 , 
    alpha : f64
}


impl Ewma {
    pub fn new(initial : f64 , alpha : f64) -> Self{
        Self{ value: AtomicU64::new(initial.to_bits()), alpha }

    }

    pub fn get(&self) -> f64 {
        f64::from_bits(self.value.load(Ordering::Relaxed))
    }

    pub fn observe (&self , sample : f64) {
        let mut prev = self.get();

        loop{
            let next = self.alpha * sample + (1.0 - self.alpha) * prev;

            match self.value.compare_exchange_weak(
                prev.to_bits(), next.to_bits(), Ordering::Relaxed, Ordering::Relaxed){
                    Ok(_) => break, 
                    Err(actual) => {
                        prev = f64::from_bits(actual);
                    }
                }
        }
    }
}


pub struct LatencyTracker{
    current : Ewma , 
    baseline : Ewma
}


impl LatencyTracker{
    pub fn new(
        initial_latency : Duration, 
        alpha_fast : f64 ,
        alpha_slow : f64 ,
    ) -> Self {
        let initial = initial_latency.as_secs_f64();

        Self { current: Ewma::new(initial, alpha_fast), baseline: Ewma::new(initial, alpha_slow) }
    }

    pub fn observe(&self , latency : Duration){

        let sample = latency.as_secs_f64();
        self.current.observe(sample);
        self.baseline.observe(sample);
    }

    pub fn snapshot(&self) -> (f64 , f64){
        (self.current.get() , self.baseline.get())
    }
}
// feedback loop + controller