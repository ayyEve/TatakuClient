use crate::prelude::*;

pub trait TatakuRenderable: Sync + Send {
    fn get_name(&self) -> String { "Unnamed".to_owned() }
    
    fn get_scissor(&self) -> Scissor { None }
    fn set_scissor(&mut self, _c: Scissor) {}

    fn draw(&self, transform: Matrix, g: &mut GraphicsState);
    fn draw_with_transparency(&self, _alpha: f32, _border_alpha: f32, transform: Matrix, g: &mut GraphicsState) {
        self.draw(transform, g)
    }
}
