use crate::*;
use style::*;

#[derive(Clone, Debug)]
pub struct DualShorthand<T> {
    pub x: CssValue<T>,
    pub y: CssValue<T>,
}
impl<T: FromStr> DualShorthand<T> {
    pub fn merge<'a, 'b: 'a>(
        &'a self, 
        x: &'b CssValue<T>,
        y: &'b CssValue<T>,
    ) -> (&'a CssValue<T>, &'a CssValue<T>) {
        (
            x.merge(&self.x),
            y.merge(&self.y),
        )
    }

    pub fn check_inherit(self, parent: Self) -> Self {
        Self {
            x: self.x.check_inherit(parent.x),
            y: self.y.check_inherit(parent.y),
        }
    }
    pub fn check_unset(self, parent: Self) -> Self {
        Self {
            x: self.x.check_unset(parent.x),
            y: self.y.check_unset(parent.y),
        }
    }
    
}
impl<T> Default for DualShorthand<T> {
    fn default() -> Self {
        Self {
            x: CssValue::Unset,
            y: CssValue::Unset,
        }
    }
}
impl<T: FromStr + Clone> FromStr for DualShorthand<T> {
    type Err = <T as FromStr>::Err;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let s = s.trim();
        let mut split = s.split(" ");

        let Some(x) = split.next() 
        else { return Ok(Self::default()) };
        let x = CssValue::parse_or_unset(x);

        if let Some(y) = split.next() {
            let y = CssValue::parse_or_unset(y);
            Ok(Self { x, y })
        } else {
            Ok(Self { x: x.clone(), y: x })
        }
    }
}
