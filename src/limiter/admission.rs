use std::sync::atomic::{AtomicUsize, Ordering};

use crate::limiter::inflight::{Inflight, InflightGuard};




pub enum Admission<'a>{
    Accepted(InflightGuard<'a>),
    Rejected,
}


pub fn try_admit<'a>(
    inflight : &'a Inflight, 
    limit : &AtomicUsize
)->Admission<'a>{
    let guard = inflight.acquire();

    let current = inflight.load();
    let max = limit.load(Ordering::Relaxed);

    if current > max {
        drop(guard);
        Admission::Rejected
    }
    else{
        Admission::Accepted(guard)
    }
}