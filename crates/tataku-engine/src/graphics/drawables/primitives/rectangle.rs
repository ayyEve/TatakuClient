use crate::prelude::*;

#[derive(ChainableInitializer)]
#[derive(Copy, Clone)]
pub struct Rectangle {
    inner: Bounds,
    
    pub color: Color,
    pub rotation: f32,

    pub origin: Vector2,
    pub scale: Vector2,
    blend_mode: Pipeline,

    #[chain] pub shape: Shape,
    pub border: Option<Border>,
}
impl Rectangle {
    pub fn new(
        pos: Vector2, 
        size: Vector2, 
        color: Color, 
    ) -> Self {
        Self::new_bounds(Bounds::new(pos, size), color)
    }

    pub fn new_bounds(
        bounds: Bounds, 
        color: Color,
    ) -> Self {
        Self {
            inner: bounds,
            scale: Vector2::ONE,

            color,
            rotation: 0.0,
            shape: Shape::Square,
            blend_mode: Pipeline::AlphaBlending,

            border: None,
            origin: bounds.size / 2.0,
        }
    }

    pub fn border(mut self, border: Border) -> Self {
        self.border = Some(border);
        self
    }
    pub fn border_maybe(mut self, border: Option<Border>) -> Self {
        self.border = border;
        self
    }
}

impl TatakuRenderable for Rectangle {
    fn get_name(&self) -> String { "Rectangle".to_owned() }
    fn get_bounds(&self) -> Bounds { self.inner }

    fn get_blend_mode(&self) -> Pipeline { self.blend_mode }
    fn set_blend_mode(&mut self, blend_mode: Pipeline) { self.blend_mode = blend_mode }

    fn draw(
        &self, 
        options: &DrawOptions, 
        transform: Matrix, 
        g: &mut dyn GraphicsEngine
    ) {
        let color = options.color_with_alpha(self.color);
        
        let border = self.border.map(|mut b| { b.color = options.border_color_with_alpha(b.color); b });
        
        let transform = transform * Matrix::identity()
            .trans(-self.origin) // apply origin
            .rot(self.rotation) // rotate to rotate
            .trans(self.origin) // undo origin
            .scale(self.scale) // scale to size
            .trans(self.inner.pos) // move to pos
        ;

        g.draw_rect(
            [0.0, 0.0, self.inner.size.x, self.inner.size.y], 
            border, 
            self.shape, 
            color, 
            transform, 
            self.blend_mode
        );
    }
}

impl From<Bounds> for Rectangle {
    fn from(other: Bounds) -> Self {
        Self {
            inner: other,
            color: Color::BLACK,
            rotation: 0.0,
            origin: other.size / 2.0,
            scale: Vector2::ONE,
            blend_mode: Pipeline::default(),
            shape: Shape::Square,
            border: None
        }
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
#[derive(Copy, Clone, Debug, PartialEq)]
#[derive(serde::Deserialize)]
pub enum Shape {
    /// Square corners
    Square,

    /// Round corners
    Round(f32),

    /// Round corners with separate vals
    /// tl,tr, bl,br
    RoundSep([f32; 4]),
}
impl From<[f32;4]> for Shape {
    fn from(value: [f32;4]) -> Self {
        Self::RoundSep(value)
    }
}
