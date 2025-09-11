use crate::*;

pub trait TatakuBeatmap: Send+Sync {
    fn hash(&self) -> common::Md5Hash;
    fn playmode(&self, incoming: String) -> String;

    fn get_timing_points(&self) -> Vec<beatmaps::TimingPoint>;
    fn get_beatmap_meta(&self) -> Arc<BeatmapMeta>;

    fn slider_velocity(&self) -> f32 { 1.0 }

    fn get_events(&self) -> Vec<gameplay::helpers::BeatmapEvent> { Vec::new() }

    #[cfg(feature="graphics")]
    fn get_animation(&self, _skin_manager: &mut dyn graphics::SkinProvider) -> Option<Box<dyn engine::BeatmapAnimation>> { None }
}
