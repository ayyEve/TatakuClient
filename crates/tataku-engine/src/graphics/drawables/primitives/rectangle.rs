use crate::prelude::*;

// TODO: make border not required in new
#[derive(ChainableInitializer)]
#[derive(Copy, Clone)]
pub struct Rectangle {
    inner: Bounds,
    
    pub color: Color,
    pub rotation: f32,

    pub origin: Vector2,
    pub scale: Vector2,
    scissor: Scissor,
    blend_mode: BlendMode,

    #[chain] pub shape: Shape,
    #[chain] pub border: Option<Border>,
}
impl Rectangle {
    pub fn new(
        pos: Vector2, 
        size: Vector2, 
        color: Color, 
        border: Option<Border>
    ) -> Self {
        Self::new_bounds(Bounds::new(pos, size), color, border)
    }

    pub fn new_bounds(
        bounds: Bounds, 
        color: Color, 
        border: Option<Border>
    ) -> Self {
        Self {
            inner: bounds,
            scale: Vector2::ONE,

            color,
            rotation: 0.0,
            shape: Shape::Square,
            scissor: None,
            blend_mode: BlendMode::AlphaBlending,

            border,
            origin: bounds.size / 2.0,
        }
    }

    /// used when a rect is only used for style info
    pub fn style_only(
        color: Color, 
        border: Option<Border>, 
        shape: Shape
    ) -> Self {
        Self::new(
            Vector2::ZERO, 
            Vector2::ZERO, 
            color, 
            border
        ).shape(shape)
    }
}

impl TatakuRenderable for Rectangle {
    fn get_name(&self) -> String { "Rectangle".to_owned() }
    fn get_bounds(&self) -> Bounds { self.inner }

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
            scissor: None,
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

mod lua {
    use crate::prelude::*;
    use lua::*;
    impl FromLua for Shape {
        fn from_lua(lua_value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
            #[cfg(feature="debug_custom_menus")] info!("Reading Shape");
            match lua_value {
                LuaValue::Integer(i) => Ok(Self::Round(i as f32)),
                LuaValue::Number(n) => Ok(Self::Round(n as f32)),
                LuaValue::Table(table) => {
                    if let Some(round) = table.get("round")? {
                        Ok(Self::Round(round))
                    } else {
                        todo!("i got lazy")
                    }

                }

                other => Err(FromLuaConversionError { 
                    from: other.type_name(), 
                    to: "Shape".to_owned(), 
                    message: Some("Invalid type".to_owned()) 
                })
            }
        }
    }
}
