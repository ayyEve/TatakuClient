use crate::*;

#[derive(Clone, Copy, Debug, Default2)]
pub struct TimingPoint {
    /// Start time of the timing section, in milliseconds from the beginning of the beatmap's audio. The end of the timing section is the next timing point's time (or never, if this is the last timing point).
    pub time: f32,

    /// This property has two meanings:
    ///     For uninherited timing points, the duration of a beat, in milliseconds.
    ///     For inherited timing points, a negative inverse slider velocity multiplier, as a percentage. For example, -50 would make all sliders in this timing section twice as fast as SliderMultiplier.
    pub beat_length: f32,

    /// Volume percentage for hit objects
    #[default(100)]
    pub volume: u8,

    /// Amount of beats in a measure. Inherited timing points ignore this property.
    #[default(4)]
    pub meter: u8,

    // effects

    /// Whether or not kiai time is enabled
    pub kiai: bool,

    /// Whether or not the first barline is omitted in osu!taiko and osu!mania
    pub skip_first_barline: bool,

    // samples

    /// Default sample set for hit objects (0 = beatmap default, 1 = normal, 2 = soft, 3 = drum)
    pub sample_set: u8,

    /// Custom sample index for hit objects. 0 indicates osu!'s default hitsounds
    pub sample_index: u8
}
impl TimingPoint {
    pub fn is_inherited(&self) -> bool {
        self.beat_length < 0.0
    }

    pub fn bpm_multiplier(&self) -> f32 {
        if self.beat_length > 0.0 { 1.0 }
        else {(-self.beat_length).clamp(10.0, 1000.0) / 100.0}
    }
}


impl PartialEq for TimingPoint {
    fn eq(&self, other: &Self) -> bool {
        self.time.eq(&other.time)
    }
}
impl PartialOrd for TimingPoint {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Eq for TimingPoint {}
impl Ord for TimingPoint {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.partial_cmp(&other.time)
            // if the time is equal, whichever is the control point should be first
            .unwrap_or_else(|| if !self.is_inherited() {
                std::cmp::Ordering::Greater
            } else if !other.is_inherited() {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            })
    }
}


pub trait TimingPointSearch {
    fn control_point_at(&self, time: f32) -> &TimingPoint;
    fn timing_point_at(&self, time: f32) -> &TimingPoint;
}

impl TimingPointSearch for Vec<TimingPoint> {
    fn control_point_at(&self, time: f32) -> &TimingPoint {
        let mut tp = &self[0];
        for t in self.iter() {
            if t.is_inherited() { continue }

            if t.time <= time {
                tp = t;
            } else { 
                break;
            }
        }
        tp
    }
    fn timing_point_at(&self, time: f32) -> &TimingPoint {
        let mut tp = &self[0];
        for t in self.iter() {
            if t.time <= time {
                tp = t;
            } else { 
                break;
            }
        }
        tp
    }
}
