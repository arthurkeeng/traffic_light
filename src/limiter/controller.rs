use std::{sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}}, thread, time::Duration};

use crate::{latency::ewma::LatencyTracker, limiter::inflight::{Inflight, InflightGuard}};



pub struct Controller {
    // controlled variable 
    limit : AtomicUsize,

    // safety bounds
    min_limit : usize , 
    max_limit : usize,

    // control parameters 

    healthy_threshold : f64 ,
    overload_threshold : f64 ,
    increase_step : usize ,
    decrease_factor : f64,

    // inputs 
    inflight : Inflight, 
    latency : LatencyTracker,

    // timing 
    control_interval : Duration, 
    running : AtomicBool,
    // last_update : AtomicUsize,
}


impl Controller{
    pub fn new(
        initial_limit : usize , 
        min_limit : usize,
        max_limit : usize,
        increase_step : usize , 
        decrease_factor : f64 , 
        healthy_threshold : f64 , 
        overload_threshold : f64 , 
        latency : LatencyTracker, 
        inflight : Inflight, 
        control_interval : Duration,
    ) -> Self {
        Self { limit: AtomicUsize::new(initial_limit), min_limit, max_limit, healthy_threshold, overload_threshold, increase_step, decrease_factor, inflight, latency ,
        control_interval, 
        running : AtomicBool::new(false),
        }
    }
    #[inline]

    pub fn limit (&self) -> usize{
        self.limit.load(Ordering::Relaxed)
    }

    pub fn update(&self) {
        let (current , baseline ) = self.latency.snapshot();

        if baseline <= 0.0 {
            return ; 
        }

        let ratio = current / baseline;

        let current_limit = self.limit();

        let mut new_limit = current_limit;

        if ratio <= self.healthy_threshold{
            new_limit = current_limit.saturating_add(self.increase_step);
        }
        else if ratio >= self.overload_threshold{
            let decreased = ((current_limit as f64) * self.decrease_factor) as usize;

            new_limit = decreased;
        }
        new_limit = new_limit.max(self.min_limit).min(self.max_limit);

        self.limit.store(new_limit, Ordering::Relaxed);
    }

    pub fn try_admit(&self) -> Option<InflightGuard<'_>>{

        let inflight = self.inflight.load();
        let limit = self.limit();

        if inflight >= limit {
            return None
        }
        Some(self.inflight.acquire())

        
    }

    pub fn start(self : Arc<Self>){
        self.running.store(true, Ordering::Relaxed);

        thread::spawn(move||{
            while self.running.load(Ordering::Relaxed) {
                self.update();
                thread::sleep(self.control_interval);
            }
        });
    }
    pub fn stop(&self){
        self.running.store(false, Ordering::Relaxed);
    }
}