use core::ops::{ Add, Sub };
use core::cmp::Ord;

pub trait WrappingLimit {
    fn wrapping_add_1(self, limit: Self) -> Self;
    fn wrapping_sub_1(self, limit: Self) -> Self;
}

impl<T: Add<Output=T> + Sub<Output=T> + GetOne + Ord + Eq + Copy> WrappingLimit for T {
    fn wrapping_add_1(self, limit: Self) -> Self {
        let n = self + Self::one();
        if n >= limit {
            Self::zero()
        } else {
            n
        }
    }
    fn wrapping_sub_1(self, limit: Self) -> Self {
        if self == Self::zero() {
            limit - Self::one()
        } else {
            self - Self::one()
        }
    }
}

pub trait GetOne: Sized {
    fn zero() -> Self;
    fn one() -> Self;
}

macro_rules! impl_get_one {
    ($($t: ty),*) => {$(
        impl GetOne for $t {
            fn zero() -> Self { 0 as $t }
            fn one() -> Self { 1 as $t }
        }
    )*}
}

impl_get_one!(u8, u16, u32, u64, u128, usize);
impl_get_one!(i8, i16, i32, i64, i128, isize);