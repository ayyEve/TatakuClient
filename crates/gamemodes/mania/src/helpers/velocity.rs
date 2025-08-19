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

#[derive(Debug, Clone, Default2)]
pub struct PositionPoint {
    #[default(-LEAD_IN_TIME)] pub time: f32,

    #[default(-LEAD_IN_TIME)] pub position: f32
}
