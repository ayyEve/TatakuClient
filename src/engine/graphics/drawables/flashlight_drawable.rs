use crate::prelude::*;

pub struct FlashlightDrawable {
    pub pos: Vector2,
    pub radius: f32,
    pub fade_radius: f32,
    pub bounds: Bounds,

    pub color: Color,

    scissor: Scissor,
}
impl FlashlightDrawable {
    pub fn new(pos: Vector2, radius: f32, fade_radius: f32, bounds: Bounds, color: Color) -> Self {
        Self {
            pos, 
            radius, 
            fade_radius, 
            bounds,
            color,
            scissor: None
        }
    }
}

impl TatakuRenderable for FlashlightDrawable {
    fn get_name(&self) -> String { "Flashlight".to_owned() }
    fn get_bounds(&self) -> Bounds { Bounds::new(self.pos, Vector2::ONE * self.radius) }

    fn get_scissor(&self) -> Scissor {self.scissor}
    fn set_scissor(&mut self, s:Scissor) {self.scissor = s}
    fn get_blend_mode(&self) -> BlendMode { BlendMode::Flashlight }
    fn set_blend_mode(&mut self, _blend_mode: BlendMode) { }

    fn draw(&self, transform: Matrix, g: &mut dyn GraphicsEngine) {
        #[cfg(feature="graphics")]
        g.draw_flashlight(
            self.bounds.into_quad(), 
            transform, 
            FlashlightData {
                cursor_pos: self.pos.into(),
                flashlight_radius: self.radius,
                fade_radius: self.fade_radius,
                color: self.color.into(),
            }
        );
    }
} 