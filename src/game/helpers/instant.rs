use crate::prelude::*;

static mut TIME:u64 = 0;
fn get_time() -> u64 {
    unsafe {
        TIME
    }
}
pub fn set_time(t: Duration) {
    unsafe {
        TIME = t.as_nanos() as u64
    }
}


#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Instant(u64);
impl Instant {
    pub fn now() -> Self {
        Self (get_time())
    }

    pub fn elapsed(&self) -> Duration {
        Duration::from_nanos(get_time() - self.0)
    }

    pub fn duration_since(&self, other: Self) -> Duration {
        Duration::from_nanos(self.0 - other.0)
    }
}
