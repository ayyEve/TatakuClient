#![allow(dead_code)]
use crate::prelude::*;


#[derive(Copy, Clone, Default)]
pub struct Transformation {
    /// how long to wait before this transform is started
    pub offset: f64,
    /// how long the tranform lasts
    pub duration: f64,
    pub trans_type: TransformType,
    pub easing_type: TransformEasing,
    
    /// when was this transform crated? (ms)
    pub create_time: f64,
}
impl Transformation {
    pub fn new(offset: f64, duration: f64, trans_type: TransformType, easing_type: TransformEasing, create_time: f64) -> Self {
        Self {
            offset,
            duration,
            trans_type,
            easing_type,
            create_time
        }
    }
    pub fn start_time(&self) -> f64 {
        self.create_time + self.offset
    }
    
    pub fn get_value(&self, current_game_time: f64) -> TransformValueResult {
        // when this transform should start
        let begin_time = self.start_time();
        // how long has elapsed? (minimum 0ms, max self.duration)
        let elapsed = (current_game_time - begin_time).clamp(0.0, self.duration);

        // % for interpolation
        let factor = elapsed / self.duration;

        match self.trans_type {
            TransformType::Position { start, end } => 
                TransformValueResult::Vector2(self.run_easing_fn(start, end, factor)),

            TransformType::Scale { start, end }
            | TransformType::BorderSize { start, end } 
            | TransformType::Rotation { start, end }
            | TransformType::Transparency { start, end } 
            | TransformType::BorderTransparency { start, end }
            => TransformValueResult::F64(self.run_easing_fn( start, end, factor)),

            TransformType::Color { start, end } 
            => TransformValueResult::Color(self.run_easing_fn( start, end, factor)),

            TransformType::None => TransformValueResult::None,
        }
    }


    // thank god i got this working lmao
    pub fn run_easing_fn<I:Interpolation>(&self, current:I, target:I, factor: f64) -> I {
        match self.easing_type {
            TransformEasing::Linear => Interpolation::lerp(current, target, factor),

            TransformEasing::EaseInSine => Interpolation::easein_sine(current, target, factor),
            TransformEasing::EaseOutSine => Interpolation::easeout_sine(current, target, factor),
            TransformEasing::EaseInOutSine => Interpolation::easeinout_sine(current, target, factor),

            TransformEasing::EaseInQuadratic => Interpolation::easein_quadratic(current, target, factor),
            TransformEasing::EaseOutQuadratic => Interpolation::easeout_quadratic(current, target, factor),
            TransformEasing::EaseInOutQuadratic => Interpolation::easeinout_quadratic(current, target, factor),

            TransformEasing::EaseInCubic => Interpolation::easein_cubic(current, target, factor),
            TransformEasing::EaseOutCubic => Interpolation::easeout_cubic(current, target, factor),
            TransformEasing::EaseInOutCubic => Interpolation::easeinout_cubic(current, target, factor),

            TransformEasing::EaseInQuartic => Interpolation::easein_quartic(current, target, factor),
            TransformEasing::EaseOutQuartic => Interpolation::easeout_quartic(current, target, factor),
            TransformEasing::EaseInOutQuartic => Interpolation::easeinout_quartic(current, target, factor),

            TransformEasing::EaseInQuintic => Interpolation::easein_quintic(current, target, factor),
            TransformEasing::EaseOutQuintic => Interpolation::easeout_quintic(current, target, factor),
            TransformEasing::EaseInOutQuintic => Interpolation::easeinout_quintic(current, target, factor),

            TransformEasing::EaseInExponential => Interpolation::easein_exponential(current, target, factor),
            TransformEasing::EaseOutExponential => Interpolation::easeout_exponential(current, target, factor),
            TransformEasing::EaseInOutExponential => Interpolation::easeinout_exponential(current, target, factor),

            TransformEasing::EaseInCircular => Interpolation::easein_circular(current, target, factor),
            TransformEasing::EaseOutCircular => Interpolation::easeout_circular(current, target, factor),
            TransformEasing::EaseInOutCircular => Interpolation::easeinout_circular(current, target, factor),

            TransformEasing::EaseInBack(c1, c2) => Interpolation::easein_back(current, target, factor, c1, c2),
            TransformEasing::EaseOutBack(c1, c2) => Interpolation::easeout_back(current, target, factor, c1, c2),
            TransformEasing::EaseInOutBack(c1, c2) => Interpolation::easeinout_back(current, target, factor, c1, c2),
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
impl Into<Vector2> for TransformValueResult {
    fn into(self) -> Vector2 {
        if let Self::Vector2(v) = self {
            v
        } else {
            // we want to crash here
            // if we get here its an issue in my code, and must be fixed
            panic!("NOT A VECTOR2!!")
        }
    }
}
impl Into<f64> for TransformValueResult {
    fn into(self) -> f64 {
        if let Self::F64(v) = self {
            v
        } else {
            // we want to crash here
            // if we get here its an issue in my code, and must be fixed
            panic!("NOT AN f64!!")
        }
    }
}
impl Into<Color> for TransformValueResult {
    fn into(self) -> Color {
        if let Self::Color(v) = self {
            v
        } else {
            // we want to crash here
            // if we get here its an issue in my code, and must be fixed
            panic!("NOT AN f64!!")
        }
    }
}


#[derive(Copy, Clone)]
pub enum TransformType {
    None, // default
    Scale {start: f64, end: f64},
    Rotation {start: f64, end: f64},
    Color {start: Color, end: Color},
    BorderSize {start: f64, end: f64},
    Transparency {start: f64, end: f64},
    Position {start: Vector2, end: Vector2},
    BorderTransparency {start: f64, end: f64},
}
impl Default for TransformType {
    fn default() -> Self {
        TransformType::None
    }
}


/// values and equations taken from https://easings.net/
#[derive(Copy, Clone)]

pub enum TransformEasing {
    Linear,
    // sine
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    // quadratic
    EaseInQuadratic,
    EaseOutQuadratic,
    EaseInOutQuadratic,
    // cubic
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    // quartic
    EaseInQuartic,
    EaseOutQuartic,
    EaseInOutQuartic,
    // quintic
    EaseInQuintic,
    EaseOutQuintic,
    EaseInOutQuintic,
    // exponential
    EaseInExponential,
    EaseOutExponential,
    EaseInOutExponential,
    // circular
    EaseInCircular,
    EaseOutCircular,
    EaseInOutCircular,
    // back
    EaseInBack(f64, f64),
    EaseOutBack(f64, f64),
    EaseInOutBack(f64, f64),
}
impl Default for TransformEasing {
    fn default() -> Self {
        TransformEasing::Linear
    }
}

pub trait Transformable: Renderable {
    fn apply_transform(&mut self, transform: &Transformation, value: TransformValueResult);

    /// is this item visible
    fn visible(&self) -> bool;

    /// should this item be removed from the draw list?
    fn should_remove(&self) -> bool {false}
}