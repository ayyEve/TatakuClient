use crate::prelude::*;

pub struct FlashlightDrawable {
    pub pos: Vector2,
    pub radius: f32,
    pub fade_radius: f32,
    pub bounds: Bounds,

    pub color: Color,
}
impl FlashlightDrawable {
    pub fn new(
        pos: Vector2, 
        radius: f32, 
        fade_radius: f32, 
        bounds: Bounds, 
        color: Color
    ) -> Self {
        Self {
            pos, 
            radius, 
            fade_radius, 
            bounds,
            color,
        }
    }
}

impl TatakuRenderable for FlashlightDrawable {
    fn get_name(&self) -> String { "Flashlight".to_owned() }

    fn get_blend_mode(&self) -> Pipeline { Pipeline::Flashlight }
    fn set_blend_mode(&mut self, _blend_mode: Pipeline) { }


    #[cfg(feature="graphics")]
    fn draw(
        &self, 
        _options: &DrawOptions,
        transform: Matrix, 
        g: &mut dyn GraphicsEngine,
    ) {
        g.draw_flashlight(
            self.bounds.into_quad(), 
            transform, 
            FlashlightData {
                center: self.pos,
                flashlight_radius: self.radius,
                fade_radius: self.fade_radius,
                color: self.color,
            }
        );
    }
} 
