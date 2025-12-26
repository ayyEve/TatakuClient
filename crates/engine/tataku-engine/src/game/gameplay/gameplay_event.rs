use crate::*;



// TODO: do we want to add kiai here?
#[derive(Clone, Debug)]
pub enum BeatmapEvent {
    Break { start: f32, end: f32 }
}


pub enum GameplayEvent {
    Paused,
    UnPaused,

    /// happens right when kiai changes
    KiaiChanged {
        enabled: bool,
    },

    /// happens right when a beat occurs (or a bit after if theres lag/stutter)
    BeatHappened {
        pulse_length: f32,
    },

    SetBounds {
        bounds: tataku::Bounds,
        full_window: bool,
    },

    ApplyMods(Arc<gameplay::mods::Mods>),
}
