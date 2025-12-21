use crate::prelude::*;
// use engine::gameplay::mods::ModManager;

#[derive(Default, Debug, Clone)]
pub(crate) struct SelectBeatmapConfig {
    pub restart_song: bool,
    pub use_preview_time: bool,
    // pub mods: ModManager,
    pub playmode: ArcStr,
}
impl SelectBeatmapConfig {
    pub fn new(
        // mods: ModManager,
        playmode: ArcStr,
        restart_song: bool,
        use_preview_time: bool,
    ) -> Self {
        Self {
            // mods,
            restart_song,
            use_preview_time,
            playmode
        }
    }
}
