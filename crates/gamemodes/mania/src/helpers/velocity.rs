use crate::prelude::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct SliderVelocity {
    /// Start time of the timing section, in milliseconds from the beginning of the beatmap's audio. The end of the timing section is the next timing point's time (or never, if this is the last timing point).
    pub time: f32,
    
    /// Velocity multiplier
    pub slider_velocity: f32,
}
impl From<QuaverSliderVelocity> for SliderVelocity {
    fn from(s: QuaverSliderVelocity) -> Self {
        Self {
            time: s.start_time,
            slider_velocity: s.multiplier as f32,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PositionPoint {
    pub time: f32,
    pub position: f32
}

impl Default for PositionPoint {
    fn default() -> Self {
        Self {
            time: -LEAD_IN_TIME,
            position: -LEAD_IN_TIME,
        }
    }
}
