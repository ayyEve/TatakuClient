use crate::prelude::*;

#[async_trait]
pub trait TatakuBeatmap:Send+Sync {
    fn hash(&self) -> Md5Hash;
    fn playmode(&self, incoming: String) -> String;

    fn get_timing_points(&self) -> Vec<TimingPoint>;
    fn get_beatmap_meta(&self) -> Arc<BeatmapMeta>;

    fn slider_velocity(&self) -> f32 { 1.0 }

    fn get_events(&self) -> Vec<IngameEvent> { Vec::new() }

    #[cfg(feature="graphics")]
    async fn get_animation(&self, _skin_manager: &mut dyn SkinProvider) -> Option<Box<dyn BeatmapAnimation>> { None }
}

