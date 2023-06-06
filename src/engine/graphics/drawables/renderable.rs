use crate::prelude::*;

pub trait TatakuRenderable: Sync + Send {
    fn get_depth(&self) -> f32;
    fn get_name(&self) -> String { "Unnamed".to_owned() }
    
    // fn get_draw_state(&self) -> Option<DrawState> { None }
    // fn set_draw_state(&mut self, _c:Option<DrawState>) {}

    fn draw(&self, transform: Matrix, g: &mut GraphicsState);
    fn draw_with_transparency(&self, alpha: f32, border_alpha: f32, transform: Matrix, g: &mut GraphicsState) {
        self.draw(transform, g)
    }
}


// pub struct Graphics {}