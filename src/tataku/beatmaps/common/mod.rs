mod timing_point;
mod beatmap_meta;
mod beatmap_type;
mod tataku_replay;
mod tataku_beatmap;

pub use timing_point::*;
pub use beatmap_meta::*;
pub use beatmap_type::*;
pub use tataku_replay::*;
pub use tataku_beatmap::*;




// stolen from peppy, /shrug
pub fn map_difficulty(diff:f32, min:f32, mid:f32, max:f32) -> f32 {
    if diff > 5.0 {
        mid + (max - mid) * (diff - 5.0) / 5.0
    } else if diff < 5.0 {
        mid - (mid - min) * (5.0 - diff) / 5.0
    } else {
        mid
    }
}


/// only used for diff calc
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NoteType {
    Note,
    Slider,
    Spinner,
    /// mania only
    Hold
}
