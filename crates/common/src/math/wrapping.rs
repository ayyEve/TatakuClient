use core::ops::{ Add, Sub };
use core::cmp::Ord;

pub trait WrappingClamp {
    fn wrapping_clamp(self, min: Self, max: Self) -> Self;
    // fn wrapping_add_1(self, limit: Self) -> Self;
    // fn wrapping_sub_1(self, limit: Self) -> Self;
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
    

    // fn wrapping_add_1(self, limit: Self) -> Self {
    //     let n = self + Self::one();
    //     if n >= limit {
    //         Self::zero()
    //     } else {
    //         n
    //     }
    // }
    // fn wrapping_sub_1(self, limit: Self) -> Self {
    //     if self == Self::zero() {
    //         limit - Self::one()
    //     } else {
    //         self - Self::one()
    //     }
    // }
}

// pub trait GetOne: Sized {
//     fn zero() -> Self;
//     fn one() -> Self;
// }

// macro_rules! impl_get_one {
//     ($($t: ty),*) => {$(
//         impl GetOne for $t {
//             fn zero() -> Self { 0 as $t }
//             fn one() -> Self { 1 as $t }
//         }
//     )*}
// }

// impl_get_one!(u8, u16, u32, u64, u128, usize);
// impl_get_one!(i8, i16, i32, i64, i128, isize);