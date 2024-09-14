#![allow(dead_code)]

use crate::prelude::*;
#[derive(Copy, Clone, Default)]
pub struct Transformation {
    /// how long to wait before this transform is started
    pub offset: f32,
    /// how long the tranform lasts
    pub duration: f32,
    pub trans_type: TransformType,
    pub easing_type: Easing,
    
    /// when was this transform crated? (ms)
    pub create_time: f32,
}
impl Transformation {
    pub fn new(offset: f32, duration: f32, trans_type: TransformType, easing_type: Easing, create_time: f32) -> Self {
        Self {
            offset,
            duration,
            trans_type,
            easing_type,
            create_time
        }
    }
    pub fn start_time(&self) -> f32 {
        self.create_time + self.offset
    }
    
    pub fn get_value(&self, current_game_time: f32) -> TransformValueResult {
        // when this transform should start
        let begin_time = self.start_time();
        // how long has elapsed? (minimum 0ms, max self.duration)
        let elapsed = (current_game_time - begin_time).clamp(0.0, self.duration);

        // % for interpolation
        let mut factor = elapsed / self.duration;
        if self.duration == 0.0 {
            factor = 1.0;
        }

        match self.trans_type {
            TransformType::Position { start, end }
            | TransformType::VectorScale { start, end } => 
                TransformValueResult::Vector2(self.easing_type.run_easing(start, end, factor)),

            TransformType::Scale { start, end }
            | TransformType::BorderSize { start, end } 
            | TransformType::Rotation { start, end }
            | TransformType::Transparency { start, end } 
            | TransformType::BorderTransparency { start, end }
            | TransformType::PositionX { start, end }
            | TransformType::PositionY { start, end }
            => TransformValueResult::F64(self.easing_type.run_easing(start, end, factor) as f64),

            TransformType::Color { start, end } 
            => TransformValueResult::Color(self.easing_type.run_easing( start, end, factor)),

            TransformType::None => TransformValueResult::None,
        }
    }
}

#[derive(Copy, Clone)]
pub enum TransformValueResult {
    None,
    Vector2(Vector2),
    F64(f64),
    Color(Color)
}
impl From<TransformValueResult> for Vector2 {
    fn from(val: TransformValueResult) -> Self {
        if let TransformValueResult::Vector2(v) = val {
            v
        } else {
            // we want to crash here
            // if we get here its an issue in my code, and must be fixed
            panic!("NOT A VECTOR2!!")
        }
    }
}
impl From<TransformValueResult> for f64 {
    fn from(val: TransformValueResult) -> Self {
        if let TransformValueResult::F64(v) = val {
            v
        } else {
            // we want to crash here
            // if we get here its an issue in my code, and must be fixed
            panic!("NOT AN f64!!")
        }
    }
}
impl From<TransformValueResult> for Color {
    fn from(val: TransformValueResult) -> Self {
        if let TransformValueResult::Color(v) = val {
            v
        } else {
            // we want to crash here
            // if we get here its an issue in my code, and must be fixed
            panic!("NOT AN f64!!")
        }
    }
}


#[derive(Copy, Clone, Debug, Default)]
pub enum TransformType {
    #[default]
    None,
    VectorScale { start: Vector2, end: Vector2 },
    Scale {start: f32, end: f32},
    Rotation {start: f32, end: f32},
    Color {start: Color, end: Color},
    BorderSize {start: f32, end: f32},
    Transparency {start: f32, end: f32},
    Position {start: Vector2, end: Vector2},
    PositionX {start: f32, end: f32},
    PositionY {start: f32, end: f32},
    BorderTransparency {start: f32, end: f32},
}


pub trait Transformable: TatakuRenderable {
    fn apply_transform(&mut self, transform: &Transformation, value: TransformValueResult);

    /// is this item visible
    fn visible(&self) -> bool;

    /// should this item be removed from the draw list?
    fn should_remove(&self) -> bool {false}
}
