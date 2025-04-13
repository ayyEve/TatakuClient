use crate::prelude::*;

pub struct Blur {
    bounds: Bounds,
    amount: f32,
}
impl Blur {
    pub fn new(bounds: Bounds, amount: f32) -> Self { 
        Self { 
            bounds, 
            amount 
        }
    }
}
impl TatakuRenderable for Blur {
    fn get_bounds(&self) -> Bounds { self.bounds }

    fn get_scissor(&self) -> Scissor { Some(self.bounds.into_scissor()) }

    fn get_blend_mode(&self) -> BlendMode { BlendMode::Blur }
    fn set_blend_mode(&mut self, _blend_mode: BlendMode) {}

    fn draw(
        &self, 
        _options: &DrawOptions,
        _transform: Matrix, 
        g: &mut dyn GraphicsEngine,
    ) {
        g.draw_blur(self.bounds, self.amount, 1);
    }
}
