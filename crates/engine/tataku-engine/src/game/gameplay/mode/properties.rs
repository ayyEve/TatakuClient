use crate::*;

#[derive(Default2)]
pub struct GamemodeProperties {
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
impl GamemodeProperties {
    pub fn playmode(&self) -> &'static str {
        self.info.id
    }
}

// mods stuff
use common::ModDefinition;
use engine::gameplay::mods::{
    ModManager,
    GameplayMod,
    GameplayModGroup,
    default_mod_groups,
};
impl GamemodeProperties {
    fn iter_mod_groups(&self) -> impl Iterator<Item=GameplayModGroup> {
        default_mod_groups()
            .into_iter()
            .chain(self.info.mods.iter().map(GameplayModGroup::from_static))
    }
    fn iter_mods(&self) -> impl Iterator<Item=GameplayMod> {
        self.iter_mod_groups()
            .flat_map(|m| m.mods)
    }

    pub fn mods_as_hashmap(&self) -> HashMap<String, GameplayMod> {
        self.iter_mods()
            .map(|m| (m.id.to_owned(), m))
            .collect()
    }

    pub fn filter_mods(&self, mods: &ModManager) -> Vec<ModDefinition> {
        let ok_mods = self.mods_as_hashmap();

        mods.mods.iter()
            .filter_map(|m| ok_mods.get(m))
            .map(|m| (*m).into())
            .collect()
    }
}
