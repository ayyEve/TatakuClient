use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct Rectangle {
    pub color: Color,
    pub pos: Vector2,
    pub rotation: f64,
    pub scale: Vector2,

    pub origin: Vector2,
    draw_state: Option<DrawState>,

    pub shape: Shape,


    pub depth: f64,
    pub size: Vector2,
    pub border: Option<Border>,
}
impl Rectangle {
    pub fn new(color: Color, depth: f64, pos: Vector2, size: Vector2, border: Option<Border>) -> Rectangle {
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
            draw_state: None,
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

impl Renderable for Rectangle {
    fn get_name(&self) -> String { "Rectangle".to_owned() }
    fn get_depth(&self) -> f64 {self.depth}
    fn get_draw_state(&self) -> Option<DrawState> { self.draw_state }
    fn set_draw_state(&mut self, c:Option<DrawState>) { self.draw_state = c }

    fn draw(&self, g: &mut GlGraphics, c: Context) {
        let border_alpha = self.border.map(|b|b.color.a).unwrap_or_default();
        self.draw_with_transparency(c, self.color.a, border_alpha, g)
    }
}

impl TatakuRenderable for Rectangle {
    fn draw_with_transparency(&self, c: Context, alpha: f32, border_alpha: f32, g: &mut GlGraphics) {
        let mut r = graphics::Rectangle::new(self.color.alpha(alpha).into());
        r.shape = self.shape;

        if let Some(mut b) = self.border { 
            b.color.a = border_alpha;
            r.border = Some(b.into())
        }
        
        let transform = c.transform
            // move to pos
            .trans_pos(self.pos)

            // scale to size
            .scale_pos(self.scale)

            // undo origin
            .trans_pos(self.origin)

            // rotate to rotate
            .rot_rad(self.rotation)

            // apply origin
            .trans_pos(-self.origin)
        ;

        r.draw(
            [0.0, 0.0, self.size.x, self.size.y], 
            &self.draw_state.unwrap_or(c.draw_state), 
            transform, 
            g
        );
    }
}


impl From<ayyeve_piston_ui::prelude::Rectangle> for Rectangle {
    fn from(r: ayyeve_piston_ui::prelude::Rectangle) -> Self {
        Self::new(r.color, r.depth, r.pos, r.size, r.border)
    }
}
