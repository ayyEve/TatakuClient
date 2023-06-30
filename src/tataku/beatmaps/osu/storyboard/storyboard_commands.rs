#![allow(unused)]
use crate::prelude::*;

/// https://osu.ppy.sh/wiki/en/Storyboard/Scripting/Commands
#[derive(Copy, Clone, Debug)]
pub struct StoryboardCommand {
    pub event: StoryboardEvent,
    pub easing: StoryboardEasing,
    pub start_time: f32,
    pub end_time: f32,

    // params are included in StoryboardEvent
}


#[derive(Copy, Clone, Debug)]
pub enum StoryboardEasing {
    Linear,
    EaseOut,
    EaseIn,

    QuadIn,
    QuadOut,
    QuadInOut,

    CubicIn,
    CubicOut,
    CubicInOut,
    
    QuarticIn,
    QuarticOut,
    QuarticInOut,
    
    QuinticIn,
    QuinticOut,
    QuinticInOut,
    
    SineIn,
    SineOut,
    SineInOut,
    
    ExponentialIn,
    ExponentialOut,
    ExponentialInOut,

    CircularIn,
    CircularOut,
    CircularInOut,

    ElasticIn,
    ElasticOut,
    ElasticHalfOut,
    ElasticQuarterOut,
    ElasticInOut,

    BackIn,
    BackOut,
    BackInOut,

    BounceIn,
    BounceOut,
    BounceInOut,
}
impl StoryboardEasing {
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "0" => Some(Self::Linear),

            "1" => Some(Self::EaseOut),
            "2" => Some(Self::EaseIn),

            "3" => Some(Self::QuadIn),
            "4" => Some(Self::QuadOut),
            "5" => Some(Self::QuadInOut),

            "6" => Some(Self::CubicIn),
            "7" => Some(Self::CubicOut),
            "8" => Some(Self::CubicInOut),
            
            "9" => Some(Self::QuarticIn),
            "10" => Some(Self::QuarticOut),
            "11" => Some(Self::QuarticInOut),
            
            "12" => Some(Self::QuinticIn),
            "13" => Some(Self::QuinticOut),
            "14" => Some(Self::QuinticInOut),
            
            "15" => Some(Self::SineIn),
            "16" => Some(Self::SineOut),
            "17" => Some(Self::SineInOut),
            
            "18" => Some(Self::ExponentialIn),
            "19" => Some(Self::ExponentialOut),
            "20" => Some(Self::ExponentialInOut),

            "21" => Some(Self::CircularIn),
            "22" => Some(Self::CircularOut),
            "23" => Some(Self::CircularInOut),

            "24" => Some(Self::ElasticIn),
            "25" => Some(Self::ElasticOut),
            "26" => Some(Self::ElasticHalfOut),
            "27" => Some(Self::ElasticQuarterOut),
            "28" => Some(Self::ElasticInOut),

            "29" => Some(Self::BackIn),
            "30" => Some(Self::BackOut),
            "31" => Some(Self::BackInOut),

            "32" => Some(Self::BounceIn),
            "33" => Some(Self::BounceOut),
            "34" => Some(Self::BounceInOut),

            _ => None,
        }
    }
}

impl Into<Easing> for StoryboardEasing {
    fn into(self) -> Easing {
        match self {
            StoryboardEasing::Linear => Easing::Linear,
            StoryboardEasing::EaseOut => Easing::EaseOutCubic,
            StoryboardEasing::EaseIn => Easing::EaseInCubic,
            StoryboardEasing::QuadIn => Easing::EaseInQuadratic,
            StoryboardEasing::QuadOut => Easing::EaseOutQuadratic,
            StoryboardEasing::QuadInOut => Easing::EaseInOutQuadratic,
            StoryboardEasing::CubicIn => Easing::EaseInCubic,
            StoryboardEasing::CubicOut => Easing::EaseOutCubic,
            StoryboardEasing::CubicInOut => Easing::EaseInOutCubic,
            StoryboardEasing::QuarticIn => Easing::EaseInQuartic,
            StoryboardEasing::QuarticOut => Easing::EaseOutQuartic,
            StoryboardEasing::QuarticInOut => Easing::EaseInOutQuartic,
            StoryboardEasing::QuinticIn => Easing::EaseInQuintic,
            StoryboardEasing::QuinticOut => Easing::EaseOutQuintic,
            StoryboardEasing::QuinticInOut => Easing::EaseInOutQuintic,
            StoryboardEasing::SineIn => Easing::EaseInSine,
            StoryboardEasing::SineOut => Easing::EaseOutSine,
            StoryboardEasing::SineInOut => Easing::EaseInOutSine,
            StoryboardEasing::ExponentialIn => Easing::EaseInExponential,
            StoryboardEasing::ExponentialOut => Easing::EaseOutExponential,
            StoryboardEasing::ExponentialInOut => Easing::EaseInOutExponential,
            StoryboardEasing::CircularIn => Easing::EaseInCircular,
            StoryboardEasing::CircularOut => Easing::EaseOutCircular,
            StoryboardEasing::CircularInOut => Easing::EaseInOutCircular,
            // StoryboardEasing::ElasticIn => todo!(),
            // StoryboardEasing::ElasticOut => todo!(),
            // StoryboardEasing::ElasticHalfOut => todo!(),
            // StoryboardEasing::ElasticQuarterOut => todo!(),
            // StoryboardEasing::ElasticInOut => todo!(),
            // StoryboardEasing::BackIn => todo!(),
            // StoryboardEasing::BackOut => todo!(),
            // StoryboardEasing::BackInOut => todo!(),
            // StoryboardEasing::BounceIn => todo!(),
            // StoryboardEasing::BounceOut => todo!(),
            // StoryboardEasing::BounceInOut => todo!(),

            _ => Easing::Linear,
        }
    }
}


#[derive(Copy, Clone, Debug)]
pub enum StoryboardEvent {
    Fade { start: f32, end: f32 }, // F

    Move { start: Vector2, end: Vector2 }, // M
    MoveX { start_x: f32, end_x: f32 }, // MX
    MoveY { start_y: f32, end_y: f32 }, // MY

    Scale { start_scale: f32, end_scale: f32 }, // S
    VectorScale { start_scale: Vector2, end_scale: Vector2 }, // V

    /// radians
    Rotate { start_rotation: f32, end_rotation: f32 }, // R
    Color { start_color: Color, end_color: Color }, // C
    Parameter { param: Param }, // P

    /// this isnt used here, its automatically calculated during parsing
    Loop { loop_count: u32 }, // L

    // fuck this shit oh my god
    // Trigger { trigger_type: TriggerType}, // T
}

#[derive(Copy, Clone, Debug)]
pub enum Param {
    FlipHorizontal,
    FlipVertial,
    AdditiveBlending
}
impl Param {
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "0" | "H" => Some(Self::FlipHorizontal),
            "1" | "V" => Some(Self::FlipVertial),
            "2" | "A" => Some(Self::AdditiveBlending),
            _ => None,
        }
    }
}

// #[derive(Copy, Clone, Debug)]
// pub enum TriggerType {
//     Hitsound {
//         sample_set: Option<>
//     }
// }

