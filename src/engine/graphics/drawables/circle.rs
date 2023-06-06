use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct Circle {
    pub depth: f32,

    // current
    pub color: Color,
    pub pos: Vector2,
    pub radius: f32,

    // draw_state: Option<DrawState>,

    pub border: Option<Border>,
    pub resolution: u32,
}
impl Circle {
    pub fn new(color:Color, depth:f32, pos:Vector2, radius:f32, border: Option<Border>) -> Circle {

        Circle {
            depth,

            color,
            pos,
            radius,

            border,
            // draw_state: None,
            resolution: 128,
        }
    }
}

impl TatakuRenderable for Circle {
    fn get_name(&self) -> String { "Circle".to_owned() }
    fn get_depth(&self) -> f32 {self.depth}
    // fn get_draw_state(&self) -> Option<DrawState> {self.draw_state}
    // fn set_draw_state(&mut self, c:Option<DrawState>) {self.draw_state = c}

    fn draw(&self, transform: Matrix, g: &mut GraphicsState) {
        let border_alpha = self.border.map(|b|b.color.a).unwrap_or_default();
        self.draw_with_transparency(self.color.a, border_alpha, transform, g)
    }

    fn draw_with_transparency(&self, alpha: f32, border_alpha: f32, transform: Matrix, g: &mut GraphicsState) {
        let border = self.border.map(|mut b|{ b.color.a = border_alpha; b.into() });

        let transform = 
            // transform.scale(self)
            transform.trans(self.pos);


        g.draw_circle(self.depth as f32, self.radius, self.color.alpha(alpha), border, self.resolution, transform);

        // graphics::ellipse::Ellipse {
        //     color: self.color.alpha(alpha).into(),
        //     border,
        //     resolution: self.resolution
        // }.draw(
        //     graphics::ellipse::circle(self.pos.x, self.pos.y, self.radius),
        //     &self.draw_state.unwrap_or(c.draw_state),
        //     c.transform,
        //     g
        // );
    }
}

