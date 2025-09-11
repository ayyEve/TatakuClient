use crate::*;
use style::*;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct QuadShorthand<T> {
    pub top: CssValue<T>,
    pub left: CssValue<T>,
    pub bottom: CssValue<T>,
    pub right: CssValue<T>,
}
impl<T> QuadShorthand<T> {
    pub fn merge<'a, 'b: 'a>(
        &'a self, 
        top: &'b CssValue<T>,
        left: &'b CssValue<T>,
        bottom: &'b CssValue<T>,
        right: &'b CssValue<T>,
    ) -> (&'a CssValue<T>, &'a CssValue<T>, &'a CssValue<T>, &'a CssValue<T>) {
        (
            top.merge(&self.top),
            left.merge(&self.left),
            bottom.merge(&self.bottom),
            right.merge(&self.right),
        )
    }

    pub fn check_inherit(self, parent: Self) -> Self {
        Self {
            top: self.top.check_inherit(parent.top),
            left: self.left.check_inherit(parent.left),
            bottom: self.bottom.check_inherit(parent.bottom),
            right: self.right.check_inherit(parent.right),
        }
    }
    pub fn check_unset(self, parent: Self) -> Self {
        Self {
            top: self.top.check_unset(parent.top),
            left: self.left.check_unset(parent.left),
            bottom: self.bottom.check_unset(parent.bottom),
            right: self.right.check_unset(parent.right),
        }
    }
    
}

impl<T> Default for QuadShorthand<T> {
    fn default() -> Self {
        Self {
            top: CssValue::Unset,
            left: CssValue::Unset,
            bottom: CssValue::Unset,
            right: CssValue::Unset,
        }
    }
}

impl<T: FromStr + Clone> FromStr for QuadShorthand<T> {
    type Err = <T as FromStr>::Err;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let s = s.trim();
        let split = s.split(" ").collect::<Vec<_>>();

        match split.len() {
            0 => Ok(Self::default()),
            1 => {
                let val = CssValue::parse_or_unset(s);
                Ok(Self {
                    top: val.clone(),
                    left: val.clone(),
                    bottom: val.clone(),
                    right: val,
                })
            }
            2 => {
                let first = CssValue::parse_or_unset(split[0]);
                let second = CssValue::parse_or_unset(split[1]);

                Ok(Self {
                    top: first.clone(),
                    bottom: first,
                    left: second.clone(),
                    right: second,
                })
            }
            3 => {
                let first = CssValue::parse_or_unset(split[0]);
                let second = CssValue::parse_or_unset(split[1]);
                let third = CssValue::parse_or_unset(split[2]);

                Ok(Self {
                    top: first,
                    left: second.clone(),
                    right: second,
                    bottom: third,
                })
            }
            4.. => {
                Ok(Self {
                    top: CssValue::parse_or_unset(split[0]),
                    right: CssValue::parse_or_unset(split[1]),
                    bottom: CssValue::parse_or_unset(split[2]),
                    left: CssValue::parse_or_unset(split[3]),
                })
            }
        }
    }
}
