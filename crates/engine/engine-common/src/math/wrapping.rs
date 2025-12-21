use core::ops::{ Add, Sub };
use core::cmp::Ord;

pub trait WrappingClamp {
    fn wrapping_clamp(self, min: Self, max: Self) -> Self;
}

impl<T: Add<Output=T> + Sub<Output=T> + Ord + Eq + Copy> WrappingClamp for T {
    fn wrapping_clamp(self, min: Self, max: Self) -> Self {
        if self < min {
            max
        } else if self >= max {
            min
        } else {
            self
        }
    }
}
