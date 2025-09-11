use crate::*;
use common::reflect::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum CssValue<T> {
    #[default]
    Unset,
    Inherit,
    Value(T),
    Variable(ArcStr),
}
impl<T> CssValue<T> {
    pub fn parse<E>(
        s: &str, 
        default: Self,
        value_parser: impl Fn(&str) -> core::result::Result<T, E>
    ) -> Self {
        match s {
            "unset" => Self::Unset,
            "inherit" => Self::Inherit,
            
            other if other.starts_with("var(") => {
                let var = other
                    .trim_start_matches("var(")
                    .trim_end_matches(|c| [' ', ',', ';', ')'].contains(&c));
                Self::Variable(var.to_owned().into())
            }

            other => value_parser(other)
                .map(Self::Value)
                .unwrap_or(default),
        }
    }

    pub fn check_inherit(self, parent: Self) -> Self {
        match (self, parent) { 
            (Self::Inherit, value) => value,
            (other, _) => other
        }
    }
    pub fn check_unset(self, parent: Self) -> Self {
        match (self, parent) { 
            (Self::Inherit | Self::Unset, value) => value,
            (other, _) => other
        }
    }
    
    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Value(v) => Some(v),
            _ => None
        }
    }

    /// returns `other` if self is unset, otherwise returns self
    pub fn merge<'a, 'b: 'a>(&'a self, other: &'b Self) -> &'a Self {
        if let Self::Unset = self {
            other
        } else {
            self
        }
    }
}
impl<T: std::str::FromStr> CssValue<T> {
    pub fn parse_or_unset(s: &str) -> Self {
        Self::parse(
            s,
            Self::Unset,
            T::from_str
        )
    }
}

impl<T:Reflect + std::fmt::Debug> CssValue<T> {
    pub fn resolve<'a: 'b, 'b>(
        &'b self, 
        values: &'a dyn Reflect
    ) -> Option<MaybeOwned<'b, T>> {
        match self {
            Self::Value(v) => Some(MaybeOwned::Borrowed(v)),
            Self::Variable(path) => values.reflect_get(path).ok(),
            _ => None
        }
    }
}
impl<T:Reflect + std::fmt::Debug + Clone> CssValue<T> {
    pub fn resolve_cloned(&self, values: &dyn Reflect) -> Option<T> {
        match self {
            Self::Value(v) => Some(v.clone()),
            Self::Variable(path) => {
                values
                    .reflect_get::<T>(&**path)
                    .ok()
                    .map(|i| i.cloned())
            }
            _ => None
        }
    }
}
impl<T:Reflect + std::fmt::Debug + Copy> CssValue<T> {
    pub fn resolve_copied(&self, values: &dyn Reflect) -> Option<T> {
        match self {
            Self::Value(v) => Some(*v),
            Self::Variable(path) => {
                values
                    .reflect_get::<T>(&**path)
                    .map(|i| i.copied())
                    .ok()
            }
            _ => None
        }
    }
}

impl<T> From<T> for CssValue<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}
