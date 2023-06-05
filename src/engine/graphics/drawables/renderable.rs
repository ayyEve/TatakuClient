use crate::prelude::*;

pub trait TatakuRenderable:Renderable {
    // fn get_depth(&self) -> f64;
    // fn get_name(&self) -> String { "Unnamed".to_owned() }
    
    // fn get_draw_state(&self) -> Option<DrawState> { None }
    // fn set_draw_state(&mut self, _c:Option<DrawState>) {}

    // fn draw(&self, g: &mut GlGraphics, c:Context);
    fn draw_with_transparency(&self, c: Context, alpha: f32, border_alpha: f32, g: &mut GlGraphics);
}


// pub struct Graphics {}