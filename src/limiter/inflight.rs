use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Relaxed};


pub struct Inflight{
    current : AtomicUsize
}

impl  Inflight{
    pub fn new () -> Self {
        Self{
            current : AtomicUsize::new(0)
        }
    }
    #[inline]
    pub fn load (&self) -> usize{
        self.current.load(Relaxed)
    }

    fn increment(&self){
        let prev = self.current.fetch_add(1, Relaxed);

        debug_assert!(
            prev < usize::MAX, 
            "Inflight counter overflowed"
        )
    }

    fn decrement(&self) {
        let prev = self.current.fetch_sub(1, Relaxed);

        debug_assert!(
            prev > 0, 
            "Inflight counter underflowed"
        )
    }
    pub fn acquire(&self) -> InflightGuard<'_>{
        self.increment();

        InflightGuard { inflight: &self }
    }
  
}



pub struct InflightGuard<'a>{
    inflight : &'a Inflight
}

impl<'a> Drop for InflightGuard<'a>{
    fn drop(&mut self) {
        self.inflight.decrement();
    }
}