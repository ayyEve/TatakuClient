use crate::prelude::*;
use tataku_interface::gameplay_widgets::HIT_TIMING_DURATION;
use tataku::{
    Vector2,
    Bounds,
};
use engine::{
    data::BeatmapPreferences,
    actions::game::GameplayId,
    gameplay::gameplay_manager::{ 
        HitTiming, 
        LEAD_IN_TIME,
    }
};

#[derive(Default2)]
pub struct GameplayState {
    #[default(GameplayId::new(u32::MAX))]
    pub id: GameplayId,

    /// Current time of the song
    pub song_time: f32,

    /// Used for discord rich presence
    pub start_time: i64,

    /// Have we started playing?
    pub started: bool,

    /// Have we completed?
    pub completed: bool,

    /// Have we failed? if so when?
    pub failed: Option<f32>,

    pub lead_in_time: f32,
    pub lead_in_timer: tataku::Instant,

    /// Is this score rankable? if not it will not be uploaded
    pub unrankable: bool,

    /// List of hit timings
    pub hit_timings: Vec<HitTiming>,

    pub end_time: f32,
    pub global_offset: f32,

    pub pending_time_jump: Option<f32>,

    /// Current window size
    pub window_size: Vector2,

    /// Bounds to fit to
    pub fit_to_bounds: Option<Bounds>,


    /// should the manager be paused?
    pub should_pause: bool,

    /// is a pause pending?
    /// used for breaks. if the user tabs out during a break, a pause is pending, but we shouldnt pause until the break is over (or almost over i guess)
    pub pause_pending: bool,
    pub pause_start: Option<i64>,

    pub restart_hold_start: Option<crate::prelude::tataku::Instant>,
}
impl GameplayState {
    #[inline]
    pub fn time(&self, beatmap_preferences: &BeatmapPreferences) -> f32 {
        self.song_time - (
            self.lead_in_time 
            + beatmap_preferences.audio_offset 
            + self.global_offset
        )
    }

    pub fn update_hit_timings(&mut self, time: f32) {
        self.hit_timings
            .retain(|hit| time - hit.map_time < HIT_TIMING_DURATION);
    }

    pub fn reset(&mut self, speed: f32) {
        self.hit_timings.clear();
        self.restart_hold_start = None;

        self.failed = None;
        self.started = false;
        self.completed = false;
        self.lead_in_time = LEAD_IN_TIME / speed;
        self.lead_in_timer = tataku::Instant::now();
    }
    pub fn reset_lead_in(&mut self, is_preview: bool) {
        if is_preview {
            // dont do lead in
            self.lead_in_time = 0.0;
        } else {
            self.lead_in_timer = tataku::Instant::now();
            self.lead_in_time = LEAD_IN_TIME;
        }
    }

}
