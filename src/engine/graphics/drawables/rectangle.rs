use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct Rectangle {
    inner: Bounds,
    // pub pos: Vector2,
    // pub size: Vector2,
    // pub scale: Vector2,
    
    pub color: Color,
    pub rotation: f32,

    pub origin: Vector2,
    scissor: Scissor,

    pub shape: Shape,
    pub border: Option<Border>,
}
impl Rectangle {
    pub fn new(pos: Vector2, size: Vector2, color: Color, border: Option<Border>) -> Rectangle {
        Rectangle {
            inner: Bounds {pos, size, scale: Vector2::ONE},

            color,
            rotation: 0.0,
            shape: Shape::Square,
            scissor: None,

            border,
            origin: size / 2.0,
        }
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
    fn get_scissor(&self) -> Scissor {self.scissor}
    fn set_scissor(&mut self, s:Scissor) {self.scissor = s}

    fn draw(&self, transform: Matrix, g: &mut GraphicsState) {
        let border_alpha = self.border.map(|b|b.color.a).unwrap_or_default();
        self.draw_with_transparency(self.color.a, border_alpha, transform, g)
    }

    fn draw_with_transparency(&self, alpha: f32, border_alpha: f32, transform: Matrix, g: &mut GraphicsState) {
        // TODO: shapes
        // r.shape = self.shape;
        let border = self.border.map(|mut b|{b.color.a = border_alpha; b});
        
        let transform = transform * Matrix::identity()
            .trans(-self.origin) // apply origin
            .rot(self.rotation) // rotate to rotate
            .trans(self.origin) // undo origin
            .scale(self.inner.scale) // scale to size
            .trans(self.inner.pos) // move to pos
        ;

        g.draw_rect([0.0, 0.0, self.inner.size.x, self.inner.size.y], border, self.shape, self.color.alpha(alpha), transform, self.scissor)
    }
}

impl Deref for Rectangle {
    type Target = Bounds;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for Rectangle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Bounds {
    pub pos: Vector2,
    pub size: Vector2,
    pub scale: Vector2,
}
impl Bounds {
    pub fn new(pos: Vector2, size: Vector2) -> Self {
        Self { pos, size, scale: Vector2::ONE }
    }
    /// check if this rectangle contains a point
    pub fn contains(&self, p:Vector2) -> bool {
        p.x > self.pos.x && p.x < self.pos.x + self.size.x && p.y > self.pos.y && p.y < self.pos.y + self.size.y
    }
}
impl Into<Rectangle> for Bounds {
    fn into(self) -> Rectangle {
        let mut r = Rectangle::new(self.pos, self.size, Color::BLACK, None);
        r.inner.scale = self.scale;
        r
    }
}