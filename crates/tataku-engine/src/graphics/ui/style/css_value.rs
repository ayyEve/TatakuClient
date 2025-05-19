use crate::prelude::*;

#[derive(Clone, Debug, Default)]
pub enum CssValue<T> {
    #[default]
    Unset,
    Inherit,
    Variable(String),
    Value(T),
}
impl<T> CssValue<T> {
    pub fn parse<E>(
        s: &str, 
        default: Self,
        value_parser: impl Fn(&str) -> Result<T, E>
    ) -> Self {
        match s {
            "unset" => Self::Unset,
            "inherit" => Self::Inherit,
            
            other if other.starts_with("var(") => {
                let var = other
                    .trim_start_matches("var(")
                    .trim_end_matches(|c| [' ', ',', ';', ')'].contains(&c));
                Self::Variable(var.to_owned())
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
}
impl<T:Reflect + std::fmt::Debug> CssValue<T> {
    pub fn value_var<'a: 'b, 'b>(
        &'b self, 
        values: &'a dyn Reflect
    ) -> Option<MaybeOwned<'b, T>> {
        match self {
            Self::Value(v) => Some(MaybeOwned::Borrowed(v)),
            Self::Variable(path) => {
                let val = values.reflect_get(path).ok();
                // println!("================================");
                // println!("path: '{path}' = {val:?}");
                // println!("================================");
                val
            },
            _ => None
        }
    }
}
impl<T:Reflect + std::fmt::Debug + Clone> CssValue<T> {
    pub fn value_var_cloned(&self, values: &dyn Reflect) -> Option<T> {
        match self {
            Self::Value(v) => Some(v.clone()),
            Self::Variable(path) => {
                let val = values
                    .reflect_get::<T>(path)
                    .ok()
                    .map(|i| i.cloned());
                // println!("================================");
                // println!("path: '{path}' = {val:?}");
                // println!("================================");
                val
            }
            _ => None
        }
    }
}
impl<T:Reflect + std::fmt::Debug + Copy> CssValue<T> {
    pub fn value_var_copied(&self, values: &dyn Reflect) -> Option<T> {
        match self {
            Self::Value(v) => Some(*v),
            Self::Variable(path) => {
                let val = values
                    .reflect_get::<T>(path)
                    .map(|i| i.copied())
                    .ok();
                // println!("================================");
                // println!("path: '{path}' = {val:?}");
                // println!("================================");
                val
            }
            _ => None
        }
    }
}
