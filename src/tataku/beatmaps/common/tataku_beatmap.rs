use crate::prelude::*;

pub trait TatakuBeatmap {
    fn hash(&self) -> String;

    fn get_timing_points(&self) -> Vec<TimingPoint>;
    fn get_beatmap_meta(&self) -> Arc<BeatmapMeta>;

    fn playmode(&self, incoming:PlayMode) -> PlayMode;

    fn slider_velocity_at(&self, time:f32) -> f32;
    fn beat_length_at(&self, time:f32, allow_multiplier:bool) -> f32;
    fn control_point_at(&self, time:f32) -> TimingPoint;
}