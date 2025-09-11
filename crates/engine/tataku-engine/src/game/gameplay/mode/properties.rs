use crate::*;

#[derive(Default2)]
pub struct GameModeProperties {
    #[default(&gameplay::GamemodeInfo::DEFAULT)]
    pub info: &'static gameplay::GamemodeInfo,
    // pub playmode: CowStr,
    pub keys: Vec<(common::replays::KeyPress, &'static str)>,
    pub end_time: f32,
    pub show_cursor: bool,
    pub timing_bar_things: Vec<(f32, tataku::Color)>,

    pub audio_prefix: String,
    pub sound_list: Vec<(String, Vec<actions::audio::AudioLoadData>)>,
}
impl GameModeProperties {
    pub fn playmode(&self) -> &'static str {
        self.info.id
    }
}
