use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct Rectangle {
    pub color: Color,
    pub pos: Vector2,
    pub rotation: f32,
    pub scale: Vector2,

    pub origin: Vector2,
    // draw_state: Option<DrawState>,

    pub shape: Shape,


    pub depth: f32,
    pub size: Vector2,
    pub border: Option<Border>,
}
impl Rectangle {
    pub fn new(color: Color, depth: f32, pos: Vector2, size: Vector2, border: Option<Border>) -> Rectangle {
        let rotation = 0.0;
        let scale = Vector2::ONE;
        
        Rectangle {
            color,
            pos,
            scale,
            rotation,
            shape: Shape::Square,

            depth,
            size,
            border,
            origin: size / 2.0,
            // draw_state: None,
        }
    }
    
    /// helpful shortcut when you only want to measure text
    pub fn bounds_only(pos: Vector2, size: Vector2) -> Rectangle {
        Rectangle::new(Color::BLACK, 0.0, pos, size, None)
    }

    /// check if this rectangle contains a point
    pub fn contains(&self, p:Vector2) -> bool {
        p.x > self.pos.x && p.x < self.pos.x + self.size.x && p.y > self.pos.y && p.y < self.pos.y + self.size.y
    }
}

// chaining properties
impl Rectangle {
    pub fn shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        self
    }
}

impl TatakuRenderable for Rectangle {
    fn get_name(&self) -> String { "Rectangle".to_owned() }
    fn get_depth(&self) -> f32 {self.depth}
    // fn get_draw_state(&self) -> Option<DrawState> { self.draw_state }
    // fn set_draw_state(&mut self, c:Option<DrawState>) { self.draw_state = c }

    fn draw(&self, transform: Matrix, g: &mut GraphicsState) {
        let border_alpha = self.border.map(|b|b.color.a).unwrap_or_default();
        self.draw_with_transparency(self.color.a, border_alpha, transform, g)
    }

    fn draw_with_transparency(&self, alpha: f32, border_alpha: f32, transform: Matrix, g: &mut GraphicsState) {
        // TODO: shapes
        // r.shape = self.shape;
        let border = self.border.map(|mut b|{b.color.a = border_alpha; b});
        
        let transform = transform
            // apply origin
            .trans(-self.origin)

            // rotate to rotate
            .rot(self.rotation)

            // undo origin
            .trans(self.origin)

            // scale to size
            .scale(self.scale)

            // move to pos
            .trans(self.pos)
        ;

        g.draw_rect([0.0, 0.0, self.size.x, self.size.y], self.depth, border, self.color.alpha(alpha), transform)
    }
}

