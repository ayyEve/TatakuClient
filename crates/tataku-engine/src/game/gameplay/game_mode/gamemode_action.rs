use crate::prelude::*;

pub enum GamemodeAction {
    /// Add a stat
    AddStat { stat: GameModeStat, value: f32 },

    /// Play a hitsound
    PlayHitsounds(Vec<Hitsound>),

    /// add a hit judgment
    AddJudgment(HitJudgment),

    /// removes the last judgment
    RemoveLastJudgment,

    /// add a hit timing
    AddTiming {
        hit_time: f32,
        note_time: f32,
    },

    /// add a hit indicator
    AddIndicator(Box<dyn JudgementIndicator>),

    /// perform a combo break
    ComboBreak,

    /// request to fail the game
    FailGame,

    /// A replay action
    ReplayAction(ReplayFrame),

    /// reset health to default
    ResetHealth,

    /// replace the health with a custom health manager
    ReplaceHealth(Box<dyn HealthManager>),

    /// let the manager know the map has no more notes
    MapComplete,
}

impl GamemodeAction {
    pub fn replace_health(health: impl HealthManager + 'static) -> Self {
        Self::ReplaceHealth(Box::new(health))
    }

    pub fn play_hitsounds(sounds: Vec<Hitsound>) -> Self {
        Self::PlayHitsounds(sounds)
    }
}