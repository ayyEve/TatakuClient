use crate::prelude::*;

mod fps_display;
mod volume_control;
mod replay_helpers;
mod benchmark_helper;

pub mod io;
pub mod math;
pub mod curve;
pub mod crypto;
pub mod instant;
pub mod key_counter;
pub mod score_helper;
pub mod event_handler;
mod score_submit_helper;
pub mod centered_text_helper;

pub use instant::*;
pub use fps_display::*;
pub use score_helper::*;
pub use event_handler::*;
pub use volume_control::*;
pub use replay_helpers::*;
pub use benchmark_helper::*;
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



pub fn visibility_bg(pos:Vector2, size:Vector2, depth: f64) -> Box<Rectangle> {
    let mut color = Color::WHITE;
    color.a = 0.6;
    Box::new(Rectangle::new(
        color,
        depth,
        pos,
        size,
        None
    ))
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


pub trait DurationAndReset {
    fn duration_and_reset(&mut self) -> f32;
}
impl DurationAndReset for Instant {
    fn duration_and_reset(&mut self) -> f32 {
        let now = Instant::now();
        let dur = now.duration_since(*self).as_secs_f32() * 1000.0;
        *self = now;
        dur
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

pub fn color_from_byte(r:u8, g:u8, b:u8) -> Color {
    Color::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        1.0
    )
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