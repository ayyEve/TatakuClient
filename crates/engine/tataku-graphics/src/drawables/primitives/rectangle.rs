use crate::*;

#[derive(Copy, Clone)]
#[derive(ChainableInitializer)]
pub struct Rectangle {
    inner: Bounds,
    
    pub color: Color,
    pub rotation: f32,

    pub origin: Vector2,
    pub scale: Vector2,
    blend_mode: BlendMode,

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
            blend_mode: BlendMode::AlphaBlending,

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

#[cfg(feature="graphics")]
impl TatakuRenderable for Rectangle {
    fn get_name(&self) -> String { "Rectangle".to_owned() }

    fn get_pipeline(&self) -> GraphicsPipeline { GraphicsPipeline::Standard(self.blend_mode) }
    fn set_pipeline(&mut self, pipeline: GraphicsPipeline) { 
        let GraphicsPipeline::Standard(blend_mode) = pipeline 
        else { return };

        self.blend_mode = blend_mode; 
    }

    fn draw(
        &self, 
        options: &DrawOptions, 
        transform: Matrix, 
        g: &mut dyn DrawEngine
    ) {
        let color = options.color_with_alpha(self.color);
        
        let border = self.border.map(|mut b| { 
            b.color = options.border_color_with_alpha(b.color); 
            b 
        });
        
        let transform = transform * Matrix::identity()
            .trans(-self.origin) // apply origin
            .rot(self.rotation) // rotate to rotate
            .trans(self.origin) // undo origin
            .scale(self.scale) // scale to size
            .trans(self.inner.pos) // move to pos
        ;

        g.draw_rect(
            [
                0.0, 0.0, 
                self.inner.size.x, self.inner.size.y
            ], 
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
            blend_mode: BlendMode::default(),
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
