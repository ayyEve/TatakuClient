use crate::prelude::*;
// this is bad, i dont care

#[derive(Clone, Copy)]
pub struct Sector {
    pub depth: f64,
    pub radius: f64,
    pub start: f64,
    pub end: f64,

    // initial
    pub initial_pos: Vector2,
    pub initial_scale: Vector2,
    pub initial_color: Color,

    // current
    pub current_pos: Vector2,
    pub current_scale: Vector2,
    pub current_color: Color,

    context: Option<Context>,

    pub border: Option<Border>
}
impl Sector {
    pub fn new(pos:Vector2, radius: f64, start:f64, end:f64, color:Color, depth:f64, border: Option<Border>) -> Self {
        let initial_color = color;
        let current_color = color;

        let initial_pos = pos;
        let current_pos = pos;

        let initial_size = Vector2::one();
        let current_size = Vector2::one();

        Self {
            depth,
            radius,
            start,
            end,

            initial_color,
            current_color,
            initial_pos,
            current_pos,

            initial_scale: initial_size,
            current_scale: current_size,

            border,
            context: None,
        }
    }
}
impl Renderable for Sector {
    fn get_name(&self) -> String { "Sector".to_owned() }
    fn get_depth(&self) -> f64 {self.depth}
    fn get_context(&self) -> Option<Context> {self.context}
    fn set_context(&mut self, c:Option<Context>) {self.context = c}

    fn draw(&self, g: &mut GlGraphics, c: Context) {
        let r = self.current_scale * self.radius;
        
        graphics::circle_arc(
            self.current_color.into(), 
            self.radius / 4.0,
            self.start, 
            self.end, 
            [self.current_pos.x, self.current_pos.y, r.x, r.y],
            c.transform.trans(-self.radius/2.0, -self.radius/2.0), 
            g
        );
    }
}

impl Transformable for Sector {
    fn apply_transform(&mut self, transform: &Transformation, val:TransformValueResult) {

        match transform.trans_type {
            TransformType::Position { .. } => {
                let val:Vector2 = val.into();
                // trace!("val: {:?}", val);
                self.current_pos = self.initial_pos + val;
            },
            TransformType::Scale { .. } => {
                let val:f64 = val.into();
                self.current_scale = Vector2::one() * val;
                // self.current_radius = self.initial_radius * val;
            },
            TransformType::Transparency { .. } => {
                // this is a circle, it doesnt rotate
                let val:f64 = val.into();
                self.current_color = self.current_color.alpha(val.clamp(0.0, 1.0) as f32);
            },
            TransformType::Color { .. } => {
                let val:Color = val.into();
                self.current_color = val
            },
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
            // this is a circle, it doesnt rotate
            TransformType::Rotation { .. } => {}
        }
    }

    fn visible(&self) -> bool {
        (self.current_color.a > 0.0 && self.current_scale.length_squared() > 0.0) 
        || if let Some(b) = &self.border {b.color.a > 0.0 && b.radius > 0.0} else {false}
    }
}

