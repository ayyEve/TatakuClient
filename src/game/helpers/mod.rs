use crate::prelude::*;

mod fps_display;
mod volume_control;
mod benchmark_helper;

pub mod io;
pub mod math;
pub mod curve;
pub mod crypto;
pub mod key_counter;
pub mod score_helper;
pub mod event_handler;
pub mod centered_text_helper;

pub use fps_display::*;
pub use event_handler::*;
pub use volume_control::*;
pub use benchmark_helper::*;

pub use score_helper::*;

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

// i might move this to its own crate tbh
// i find myself needing something like this quite often

pub trait Find<T> {
    fn find(&self, predecate: fn(&T) -> bool) -> Option<&T>;
    fn find_mut(&mut self, predecate: fn(&T) -> bool) -> Option<&mut T>;

    fn find_all(&self, predecate: fn(&T) -> bool) -> Vec<&T>;
    fn find_all_mut(&mut self, predecate: fn(&T) -> bool) -> Vec<&mut T>;
}
impl<T> Find<T> for Vec<T> {
    fn find(&self, predecate: fn(&T) -> bool) -> Option<&T> {
        for i in self {
            if predecate(i) {
                return Some(i)
            }
        }
        None
    }
    fn find_mut(&mut self, predecate: fn(&T) -> bool) -> Option<&mut T> {
        for i in self {
            if predecate(i) {
                return Some(i)
            }
        }
        None
    }


    fn find_all(&self, predecate: fn(&T) -> bool) -> Vec<&T> {
        let mut list = Vec::new();
        for i in self {
            if predecate(i) {
                list.push(i)
            }
        }
        list
    }
    fn find_all_mut(&mut self, predecate: fn(&T) -> bool) -> Vec<&mut T> {
        let mut list = Vec::new();
        for i in self {
            if predecate(i) {
                list.push(i)
            }
        }
        list
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