use std::{
    time::Duration,
    sync::atomic::{ AtomicU64, Ordering },
};

static TIME: AtomicU64 = AtomicU64::new(0);
fn get_time() -> u64 {
    TIME.load(Ordering::Acquire)
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Instant(u64);
impl Instant {
    pub fn now() -> Self {
        Self(get_time())
    }

    fn elapsed(&self) -> Duration {
        Duration::from_nanos(get_time() - self.0)
    }
    
    pub fn duration_since(&self, other: Self) -> Duration {
        Duration::from_nanos(self.0 - other.0)
    }

    /// time elapsed in milliseconds
    pub fn as_millis(&self) -> f32 {
        self.elapsed().as_secs_f32() * 1000.0
    }

    pub fn reset(&mut self) {
        *self = Self::now();
    }
    
    /// time elapsed in milliseconds
    pub fn elapsed_and_reset(&mut self) -> f32 {
        let now = Self::now();
        let dur = now.duration_since(*self).as_secs_f32() * 1000.0;
        *self = now;
        dur
    }

    pub fn set_time(t: Duration) {
        TIME.store(t.as_nanos() as u64, Ordering::Release);
    }
}
impl Default for Instant {
    fn default() -> Self {
        Self::now()
    }
}
