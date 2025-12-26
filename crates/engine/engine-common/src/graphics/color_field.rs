use crate::prelude::*;
use common::reflect::*;

use std::ops::{ 
    Add, AddAssign, 
    Sub, SubAssign,
    Mul, MulAssign, 
    Div, DivAssign, 
    Deref, DerefMut,
    Rem, RemAssign, 
    Neg, 
};
use std::cmp::{
    PartialEq,
    PartialOrd,
};


/// Wrapper type to implement floating-point operations on u8 colors
/// 
#[derive(Reflect)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(from = "u8", into = "u8")]
pub struct ColorField(pub u8);
impl ColorField {
    pub const MAX_VAL: u8 = 0xFF;
    pub const MAX: Self = Self::new_u8(Self::MAX_VAL);

    #[inline(always)]
    pub const fn f32_to_u8(n: f32) -> u8 {
        (n.clamp(0.0, 1.0) * Self::MAX_VAL as f32) as u8
    }
    
    #[inline(always)]
    pub const fn u8_to_f32(n: u8) -> f32 {
        (n as f32) / Self::MAX_VAL as f32
    }

    pub const fn to_f32(self) -> f32 {
        Self::u8_to_f32(self.0)
    }
    pub const fn to_u8(self) -> u8 {
        self.0
    }

    pub const fn new_f32(n: f32) -> Self {
        Self(Self::f32_to_u8(n))
    }
    pub const fn new_u8(n: u8) -> Self {
        Self(n)
    }
}

impl Default for ColorField {
    fn default() -> Self { Self::MAX }
}

impl AsRef<u8> for ColorField {
    fn as_ref(&self) -> &u8 {
        &self.0
    }
}
impl Deref for ColorField {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ColorField {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<u8> for ColorField {
    fn from(value: u8) -> Self { Self(value) }
}
impl From<f32> for ColorField {
    fn from(value: f32) -> Self { Self(Self::f32_to_u8(value)) }
}
impl From<ColorField> for u8 {
    fn from(value: ColorField) -> Self { value.0 }
}
impl From<ColorField> for f32 {
    fn from(value: ColorField) -> Self { ColorField::u8_to_f32(value.0) }
}

impl std::fmt::Display for ColorField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}


// cmp
impl PartialEq<u8> for ColorField {
    fn eq(&self, other: &u8) -> bool {
        &self.0 == other
    }
}
impl PartialOrd<u8> for ColorField {
    fn partial_cmp(&self, other: &u8) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}
impl PartialEq<f32> for ColorField {
    fn eq(&self, other: &f32) -> bool {
        let n = self.to_f32();
        &n == other
    }
}
impl PartialOrd<f32> for ColorField {
    fn partial_cmp(&self, other: &f32) -> Option<std::cmp::Ordering> {
        let n = self.to_f32();
        n.partial_cmp(other)
    }
}

// math

impl Neg for ColorField {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(255 - self.0)
    }
}

macro_rules! impl_math {
    ($($(+ $assign:ident)? $trait: ident $op_fn: ident),*$(,)?) => { $(
        impl_math!(@a $($assign)? $trait $op_fn);
    )*};

    (@a assign $trait: ident $op_fn: ident) => {
        impl $trait<Self> for ColorField {
            fn $op_fn(&mut self, rhs: Self) {
                self.0.$op_fn(rhs.0);
            }
        }
        impl $trait<u8> for ColorField {
            fn $op_fn(&mut self, rhs: u8) {
                self.0.$op_fn(rhs);
            }
        }
        impl $trait<f32> for ColorField {
            fn $op_fn(&mut self, rhs: f32) {
                let mut lhs = self.to_f32();
                lhs.$op_fn(rhs);
                *self = Self::new_f32(lhs);
            }
        }
    };

    (@a $trait: ident $op_fn: ident) => {
        impl $trait<Self> for ColorField {
            type Output = Self;
            
            fn $op_fn(self, rhs: Self) -> Self::Output {
                Self(self.0.$op_fn(rhs.0))
            }
        }
        impl $trait<u8> for ColorField {
            type Output = Self;
            
            fn $op_fn(self, rhs: u8) -> Self::Output {
                Self(self.0.$op_fn(rhs))
            }
        }
        impl $trait<f32> for ColorField {
            type Output = Self;
            
            fn $op_fn(self, rhs: f32) -> Self::Output {
                let lhs = self.to_f32();
                Self::new_f32(lhs.$op_fn(rhs))
            }
        }
    };
}
impl_math![
    Add add, +assign AddAssign add_assign,
    Sub sub, +assign SubAssign sub_assign,
    Mul mul, +assign MulAssign mul_assign,
    Div div, +assign DivAssign div_assign,
    Rem rem, +assign RemAssign rem_assign,
];
