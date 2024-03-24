use crate::prelude::*;

mod crypto;
mod score_submit_helper;

pub use crypto::*;
pub use score_submit_helper::*;



/// format a number into a locale string ie 1000000 -> 1,000,000
pub fn format_number(num: impl num_format::ToFormattedStr) -> String {
    use num_format::{Buffer, Locale};
    let mut buf = Buffer::default();
    buf.write_formatted(&num, &Locale::en);

    buf.as_str().to_owned()
}

/// format a float into a locale string ie 1000.1 -> 1,000.100
pub fn format_float(num: impl ToString, precis: usize) -> String {
    let num = num.to_string();
    let mut split = num.split(".");
    let Some(num) = split.next().and_then(|a|a.parse::<i64>().ok()).map(format_number) else { return String::new() };

    let Some(dec) = split.next() else {
        return format!("{num}.{}", "0".repeat(precis));
    };

    let dec = if dec.len() > precis {
        dec.split_at(precis).0.to_owned()
    } else {
        format!("{dec:0precis$}")
    };

    format!("{num}.{dec}")
}

pub fn visibility_bg(pos:Vector2, size:Vector2) -> impl TatakuRenderable {
    Rectangle::new(
        pos,
        size,
        Color::WHITE.alpha(0.6),
        None
    )
}



#[macro_export]
macro_rules! create_value_helper {
    ($struct: ident, $type: ty, $helper_name: ident) => {
        #[derive(Default)]
        pub struct $struct(pub $type);
        impl Deref for $struct {
            type Target = $type;
            fn deref(&self) -> &Self::Target { &self.0 }
        }
        
        pub type $helper_name = GlobalValue<$struct>;
    }
}

pub trait Remove<T> {
    fn remove_item(&mut self, item:T);
}
impl<T> Remove<T> for Vec<T> where T:Eq {
    fn remove_item(&mut self, remove_item:T) {
        for (index, item) in self.iter().enumerate() {
            if *item == remove_item {
                self.remove(index);
                return;
            }
        }
    }
}


pub trait UnwrapNormal {
    fn normal_or(self, other:Self) -> Self;
}
impl UnwrapNormal for f32 {
    fn normal_or(self, other:Self) -> Self {
        if self.is_normal() {self} else {other}
    }
}
impl UnwrapNormal for f64 {
    fn normal_or(self, other:Self) -> Self {
        if self.is_normal() {self} else {other}
    }
}


pub trait CopyDefault<T> {
    fn copy_or_default(&self) -> T;
}
impl<T:Copy+Default> CopyDefault<T> for Option<&T> {
    fn copy_or_default(&self) -> T {
        self.map(|n|*n).unwrap_or_default()
    }
}

/// helper trait so i dont need to if let Some(v) = v {v.thing()}
pub trait OkDo<T> {
    fn ok_do(&self, f: impl FnOnce(&T));
    fn ok_do_mut(&mut self, f: impl FnOnce(&mut T));
}
impl<T> OkDo<T> for Option<T> {
    fn ok_do(&self, f: impl FnOnce(&T)) {
        if let Some(s) = self { f(s) }
    }
    fn ok_do_mut(&mut self, f: impl FnOnce(&mut T)) {
        if let Some(s) = self { f(s) }
    }
}


#[macro_export]
macro_rules! async_retain {
    ($list:ident, $item:ident, $check_fn:expr) => {{

        let mut to_remove = Vec::new();
        for (n, $item) in $list.iter().enumerate() {
            if !$check_fn {
                to_remove.push(n)
            }
        }

        for i in to_remove.into_iter().rev() {
            $list.remove(i);
        }

    }}
}
