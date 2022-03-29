use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct Rectangle {

    // initial
    pub initial_color: Color,
    pub initial_pos: Vector2,
    pub initial_rotation: f64,
    pub initial_scale: Vector2,

    // current
    pub current_color: Color,
    pub current_pos: Vector2,
    pub current_rotation: f64,
    pub current_scale: Vector2,

    pub origin: Vector2,
    context: Option<Context>,

    pub shape: Shape,


    pub depth: f64,
    pub size: Vector2,
    pub border: Option<Border>,
}
impl Rectangle {
    pub fn new(color: Color, depth: f64, pos: Vector2, size: Vector2, border: Option<Border>) -> Rectangle {
        let initial_pos = pos;
        let current_pos = pos;
        let initial_rotation = 0.0;
        let current_rotation = 0.0;
        let initial_color = color;
        let current_color = color;
        let initial_scale = Vector2::one();
        let current_scale = Vector2::one();
        
        Rectangle {
            initial_color,
            current_color,
            initial_pos,
            current_pos,
            initial_scale,
            current_scale,
            initial_rotation,
            current_rotation,
            shape: Shape::Square,

            depth,
            size,
            border,
            origin: size / 2.0,
            context: None,
        }
    }
    
    /// helpful shortcut when you only want to measure text
    pub fn bounds_only(pos: Vector2, size: Vector2) -> Rectangle {
        Rectangle::new(Color::BLACK, 0.0, pos, size, None)
    }

    /// check if this rectangle contains a point
    pub fn contains(&self, p:Vector2) -> bool {
        p.x > self.current_pos.x && p.x < self.current_pos.x + self.size.x && p.y > self.current_pos.y && p.y < self.current_pos.y + self.size.y
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
    fn get_depth(&self) -> f64 {self.depth}
    fn get_context(&self) -> Option<Context> {self.context}
    fn set_context(&mut self, c:Option<Context>) {self.context = c}

    fn draw(&mut self, g: &mut GlGraphics, c: Context) {
        let mut r = graphics::Rectangle::new(self.current_color.into());
        r.shape = self.shape;

        if let Some(b) = self.border {r.border = Some(b.into())}

        let pre_rotation = self.current_pos / self.current_scale + self.origin;
        
        let transform = c
            .transform
            // scale to size
            .scale(self.current_scale.x, self.current_scale.y)

            // move to pos
            .trans(pre_rotation.x, pre_rotation.y)

            // rotate to rotate
            .rot_rad(self.current_rotation)

            // apply origin
            .trans(-self.origin.x, -self.origin.y)
        ;


        r.draw([0.0, 0.0, self.size.x, self.size.y], &DrawState::default(), transform, g);
    }
}

impl Transformable for Rectangle {
    fn apply_transform(&mut self, transform: &Transformation, val: TransformValueResult) {
        match transform.trans_type {
            TransformType::Position { .. } => {
                let val:Vector2 = val.into();
                self.current_pos = self.initial_pos + val;
            },
            TransformType::Scale { .. } => {
                let val:f64 = val.into();
                self.current_scale = self.initial_scale + val;
            },
            TransformType::Rotation { .. } => {
                let val:f64 = val.into();
                self.current_rotation = self.initial_rotation + val;
            }
            
            // self color
            TransformType::Transparency { .. } => {
                let val:f64 = val.into();
                self.current_color = self.current_color.alpha(val.clamp(0.0, 1.0) as f32);
            },
            TransformType::Color { .. } => {
                let col = val.into();
                self.current_color = col;
            },

            // border
            TransformType::BorderTransparency { .. } => if let Some(border) = self.border.as_mut() {
                // this is a circle, it doesnt rotate
                let val:f64 = val.into();
                border.color = border.color.alpha(val.clamp(0.0, 1.0) as f32);
            },
            TransformType::BorderSize { .. } => if let Some(border) = self.border.as_mut() {
                // this is a circle, it doesnt rotate
                border.radius = val.into();
            },
            TransformType::BorderColor { .. } => if let Some(border) = self.border.as_mut() {
                let val:Color = val.into();
                border.color = val
            },

            TransformType::None => {},
        }
    }
    
    fn visible(&self) -> bool {
        self.current_scale.x != 0.0 && self.current_scale.y != 0.0
    }
}