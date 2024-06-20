use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct Rectangle {
    inner: Bounds,
    
    pub color: Color,
    pub rotation: f32,

    pub origin: Vector2,
    scissor: Scissor,
    blend_mode: BlendMode,

    pub shape: Shape,
    pub border: Option<Border>,
}
impl Rectangle {
    pub fn new(pos: Vector2, size: Vector2, color: Color, border: Option<Border>) -> Self {
        Rectangle {
            inner: Bounds {pos, size, scale: Vector2::ONE},

            color,
            rotation: 0.0,
            shape: Shape::Square,
            scissor: None,
            blend_mode: BlendMode::AlphaBlending,

            border,
            origin: size / 2.0,
        }
    }

    /// used when a rect is only used for style info
    pub fn style_only(color: Color, border: Option<Border>, shape: Shape) -> Self {
        Self::new(Vector2::ZERO, Vector2::ZERO, color, border).shape(shape)
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
    fn get_bounds(&self) -> Bounds { self.inner }

    fn get_scissor(&self) -> Scissor {self.scissor}
    fn set_scissor(&mut self, s:Scissor) {self.scissor = s}
    fn get_blend_mode(&self) -> BlendMode { self.blend_mode }
    fn set_blend_mode(&mut self, blend_mode: BlendMode) { self.blend_mode = blend_mode }

    fn draw(&self, transform: Matrix, g: &mut dyn GraphicsEngine) {
        let border_alpha = self.border.map(|b|b.color.a).unwrap_or_default();
        self.draw_with_transparency(self.color.a, border_alpha, transform, g)
    }

    fn draw_with_transparency(&self, alpha: f32, border_alpha: f32, transform: Matrix, g: &mut dyn GraphicsEngine) {
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

        g.draw_rect([0.0, 0.0, self.inner.size.x, self.inner.size.y], border, self.shape, self.color.alpha(alpha), transform, self.blend_mode)
    }
}

impl From<Bounds> for Rectangle {
    fn from(other: Bounds) -> Self {
        let mut r = Self::new(other.pos, other.size, Color::BLACK, None);
        r.inner.scale = other.scale;
        r
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


/// The shape of the rectangle corners
#[derive(Copy, Clone, Debug)]
pub enum Shape {
    /// Square corners
    Square,

    /// Round corners
    Round(f32),

    /// Round corners with separate vals
    /// tl,tr, bl,br
    RoundSep([f32;4]),
}

impl From<[f32;4]> for Shape {
    fn from(value: [f32;4]) -> Self {
        Self::RoundSep(value)
    }
}
