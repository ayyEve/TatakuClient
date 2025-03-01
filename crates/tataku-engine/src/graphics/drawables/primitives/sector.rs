use crate::prelude::*;


// this is bad, i dont care
// TODO: care
#[derive(Copy, Clone)]
pub struct Sector {
    pub pos: Vector2,
    pub scale: Vector2,
    pub color: Color,

    pub radius: f32,
    pub start: f32,
    pub end: f32,

    scissor: Scissor,
    blend_mode: BlendMode,

    pub border: Option<Border>
}
impl Sector {
    pub fn new(
        pos: Vector2, 
        radius: f32, 
        start: f32, 
        end: f32, 
        color: Color, 
        border: Option<Border>
    ) -> Self {
        Self {
            radius,
            start,
            end,

            color,
            pos,
            scale: Vector2::ONE,

            border,
            scissor: None,
            blend_mode: BlendMode::AlphaBlending,
        }
    }
}

impl TatakuRenderable for Sector {
    fn get_name(&self) -> String { "Sector".to_owned() }
    fn get_bounds(&self) -> Bounds { 
        Bounds::new(self.pos, Vector2::ONE * self.radius) 
    }

    fn get_scissor(&self) -> Scissor { self.scissor }
    fn set_scissor(&mut self, s: Scissor) { self.scissor = s }
    fn get_blend_mode(&self) -> BlendMode { self.blend_mode }
    fn set_blend_mode(&mut self, blend_mode: BlendMode) { self.blend_mode = blend_mode }

    fn draw(
        &self,
        options: &DrawOptions, 
        transform: Matrix, 
        g: &mut dyn GraphicsEngine
    ) {
        g.draw_arc(
            self.start,
            self.end,
            self.radius,
            options.color_with_alpha(self.color),
            20,
            transform * Matrix::identity().scale(self.scale).trans(self.pos),
            self.blend_mode
        )
    }
}
