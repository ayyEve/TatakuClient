use crate::*;
use gameplay::health::Health;

pub enum GamemodeAction {
    /// Add a stat
    AddStat {
        stat: gameplay::stats::GameModeStat,
        value: f32
    },

    /// Play a hitsound
    PlayHitsound {
        id: String,
        volume: f32,
        repeat: bool,
    },

    /// add a hit judgment
    AddJudgment(gameplay::judgments::HitJudgment),

    /// removes the last judgment
    RemoveLastJudgment,

    /// add a hit timing
    AddTiming {
        hit_time: f32,
        note_time: f32,
    },

    /// add a hit indicator
    #[cfg(feature="graphics")]
    AddIndicator(Box<dyn gameplay::judgments::JudgementIndicator>),

    /// perform a combo break
    ComboBreak,

    /// request to fail the game
    FailGame,

    /// A replay action
    ReplayAction(common::replays::ReplayFrame),

    /// reset health to default
    ResetHealth,

    /// replace the health with a custom health manager
    ReplaceHealth(Box<dyn Health>),

    /// let the manager know the map has no more notes
    MapComplete,

    /// let the manager know the gamemode's playfield has changed
    PlayfieldChanged,
}

impl GamemodeAction {
    pub fn replace_health(health: impl Health + 'static) -> Self {
        Self::ReplaceHealth(Box::new(health))
    }
}
