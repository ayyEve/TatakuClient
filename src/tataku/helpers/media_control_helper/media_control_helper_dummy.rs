use crate::prelude::*;

pub struct MediaControlHelper;
impl MediaControlHelper {
    pub fn new(_: AsyncUnboundedSender<MediaControlHelperEvent>) -> Self { Self }
    pub async fn update(&mut self, _: MediaPlaybackState) {}
    pub fn update_info(&self, _: &Option<Arc<BeatmapMeta>>, _: f32) {}
}