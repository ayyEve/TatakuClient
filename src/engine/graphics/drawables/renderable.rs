use crate::prelude::*;

pub trait TatakuRenderable: Sync + Send {
    fn get_name(&self) -> String { "Unnamed".to_owned() }
    fn get_bounds(&self) -> Bounds;
    
    fn get_scissor(&self) -> Scissor { None }
    fn set_scissor(&mut self, _c: Scissor) {}

    fn get_blend_mode(&self) -> BlendMode;
    fn set_blend_mode(&mut self, blend_mode: BlendMode);
    fn with_blend_mode(mut self, blend_mode: BlendMode) -> Self where Self:Sized { self.set_blend_mode(blend_mode); self }

    fn draw(&self, transform: Matrix, g: &mut GraphicsState);
    fn draw_with_transparency(&self, _alpha: f32, _border_alpha: f32, transform: Matrix, g: &mut GraphicsState) {
        self.draw(transform, g)
    }
}
