use crate::prelude::*;
use common::{
    Md5Hash,
    reflect::*,
};
use engine::gameplay::{
    GamemodeInfos,
    mods::ModManager,
};

#[derive(Reflect)]
#[reflect(display = "debug")]
#[derive(Default, Debug, Clone)]
pub struct GlobalValues {
    pub mods: ModManager,
    pub mod_groups: Vec<ReflectModGroup>,

    #[reflect(alias("infos"))]
    pub gamemode_infos: GamemodeInfos,

    pub playmode: ArcStr,
    pub playmode_display: ArcStr,
    pub playmode_actual: ArcStr,
    pub playmode_actual_display: ArcStr,

    pub username: String,
    pub menu_list: Vec<ArcStr>,
    pub dialog_list: Vec<ArcStr>,

    pub new_beatmap_hash: Option<Md5Hash>,
}
impl GlobalValues {
    pub fn new(
        infos: GamemodeInfos,
        settings: &engine::Settings,
    ) -> Self {
        let mut s = Self {
            gamemode_infos: infos,
            username: settings.connection().tataku_username.clone(),
            ..Default::default()
        };
        let a: ArcStr = settings.last_played_mode.clone().into();
        s.update_playmode(a.clone());
        s.update_playmode_actual(a);
        
        s
    }

    pub fn update_playmode(
        &mut self, 
        playmode: impl Into<ArcStr>,
    ) {
        let playmode = playmode.into();
        self.playmode = playmode.clone();
        let Ok(info) = self.gamemode_infos.get_info(&playmode) 
        else { return };

        self.playmode_display = info.display_name.to_owned().into();
    }
    pub fn update_playmode_actual(
        &mut self, 
        playmode: impl Into<ArcStr>,
    ) {
        let playmode = playmode.into();
        self.playmode_actual = playmode.clone();
        let Ok(info) = self.gamemode_infos.get_info(&playmode) 
        else { return };
        self.playmode_actual_display = info.display_name.to_owned().into();


        // update mod groups
        let mod_groups = ModManager::mod_groups_for_playmode(info);
        self.mod_groups = mod_groups
            .iter()
            .map(|group| ReflectModGroup::from_group(group, &self.mods))
            .collect();
    }

    pub fn update_mods(&mut self) {
        let mode = self.gamemode_infos
            .get_info(&self.playmode_actual)
            .unwrap();
        self.mods.update_score_multiplier(mode);

        for group in self.mod_groups.iter_mut() {
            group.update(&self.mods);
        }
    }
}
