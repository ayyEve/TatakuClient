use crate::prelude::*;
use super::prelude::*;

#[derive(Clone, Copy)]
pub struct Circle {
    pub depth: f64,

    // initial
    pub initial_color: Color,
    pub initial_pos: Vector2,
    pub initial_radius: f64,

    // current
    pub current_color: Color,
    pub current_pos: Vector2,
    pub current_radius: f64,

    pub border: Option<Border>,
    spawn_time: u64,
    lifetime: u64
}
impl Circle {
    pub fn new(color:Color, depth:f64, pos:Vector2, radius:f64) -> Circle {
        let initial_color = color;
        let current_color = color;

        let initial_pos = pos;
        let current_pos = pos;

        let initial_radius = radius;
        let current_radius = radius;
        Circle {
            depth,

            initial_color,
            current_color,
            initial_pos,
            current_pos,

            initial_radius,
            current_radius,

            border: None,
            spawn_time: 0,
            lifetime: 0
        }
    }
}
impl Renderable for Circle {
    fn get_depth(&self) -> f64 {self.depth}
    fn set_lifetime(&mut self, lifetime:u64) {self.lifetime = lifetime}
    fn get_lifetime(&self) -> u64 {self.lifetime}
    fn set_spawn_time(&mut self, time:u64) {self.spawn_time = time}
    fn get_spawn_time(&self) -> u64 {self.spawn_time}

    fn draw(&mut self, g: &mut GlGraphics, c: Context) {
        graphics::ellipse::Ellipse {
            color: self.current_color.into(),
            border: if self.border.is_some() {Some(self.border.unwrap().into())} else {None},
            resolution: 128
        }.draw(
            graphics::ellipse::circle(self.current_pos.x, self.current_pos.y, self.current_radius),
            &DrawState::default(),
            c.transform,
            g
        );
    }
}

impl Transformable for Circle {
    fn apply_transform(&mut self, transform: &Transformation, val:TransformValueResult) {

        match transform.trans_type {
            TransformType::Position { .. } => {
                let val:Vector2 = val.into();
                // println!("val: {:?}", val);
                self.current_pos = self.initial_pos + val;
            },
            TransformType::Scale { .. } => {
                let val:f64 = val.into();
                self.current_radius = self.initial_radius * val;
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
        (self.current_color.a > 0.0 && self.current_radius > 0.0) 
        || if let Some(b) = &self.border {b.color.a > 0.0 && b.radius > 0.0} else {false}
    }
}

