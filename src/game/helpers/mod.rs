use crate::prelude::*;

mod fps_display;
mod volume_control;
mod benchmark_helper;

pub mod io;
pub mod math;
pub mod curve;
pub mod crypto;
pub mod key_counter;
pub mod centered_text_helper;

pub use fps_display::*;
pub use volume_control::*;
pub use benchmark_helper::*;


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