use crate::prelude::*;

mod crypto;
mod score_helper;
mod score_submit_helper;

pub use crypto::*;
pub use score_helper::*;
pub use score_submit_helper::*;



/// format a number into a locale string ie 1000000 -> 1,000,000
pub fn format_number<T:Display>(num:T) -> String {
    let str = format!("{}", num);
    let mut split = str.split(".");
    let num = split.next().unwrap();
    let dec = split.next();

    // split num into 3s
    let mut new_str = String::new();
    let offset = num.len() % 3;

    num.char_indices().rev().for_each(|(pos, char)| {
        new_str.push(char);
        if pos % 3 == offset {
            new_str.push(',');
        }
    });

    let mut new_new = String::with_capacity(new_str.len());
    new_new.extend(new_str.chars().rev());
    if let Some(dec) = dec {
        new_new += &format!(".{}", dec);
    }
    new_new.trim_start_matches(",").to_owned()
}

/// format a number into a locale string ie 1000000 -> 1,000,000
pub fn format_float<T:Display>(num:T, precis: usize) -> String {
    let str = format!("{}", num);
    let mut split = str.split(".");
    let num = split.next().unwrap();
    let dec = split.next();

    // split num into 3s
    let mut new_str = String::new();
    let offset = num.len() % 3;

    num.char_indices().rev().for_each(|(pos, char)| {
        new_str.push(char);
        if pos % 3 == offset {
            new_str.push(',');
        }
    });

    let mut new_new = String::with_capacity(new_str.len());
    new_new.extend(new_str.chars().rev());
    if let Some(dec) = dec {
        let dec = if dec.len() < precis {
            format!("{}{}", dec, "0".repeat(precis - dec.len()))
        } else {
            dec.split_at(precis.min(dec.len())).0.to_owned()
        };
        new_new += &format!(".{}", dec);
    } else if precis > 0 {
        new_new += & format!(".{}", "0".repeat(precis))
    }
    new_new.trim_start_matches(",").to_owned()
}



pub fn visibility_bg(pos:Vector2, size:Vector2, depth: f32) -> impl TatakuRenderable {
    let mut color = Color::WHITE;
    color.a = 0.6;
    Rectangle::new(
        color,
        depth,
        pos,
        size,
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
