mod pie;
mod bar;
mod scatter;

pub use pie::*;
pub use bar::*;
pub use scatter::*;


use crate::prelude::*;
pub trait StatsGraph: Send + Sync {
    fn draw(&self, bounds: &Rectangle, depth: f32, list: &mut RenderableCollection);
}