use crate::prelude::*;

pub trait TatakuBeatmap: Send+Sync {
    fn hash(&self) -> Md5Hash;
    fn playmode(&self, incoming: String) -> String;

    fn get_timing_points(&self) -> Vec<TimingPoint>;
    fn get_beatmap_meta(&self) -> Arc<BeatmapMeta>;

    fn slider_velocity(&self) -> f32 { 1.0 }

    fn get_events(&self) -> Vec<BeatmapEvent> { Vec::new() }

    #[cfg(feature="graphics")]
    fn get_animation(&self, _skin_manager: &mut dyn SkinProvider) -> Option<Box<dyn BeatmapAnimation>> { None }
}

