use crate::prelude::*;

#[derive(Reflect)]
#[reflect(display = "debug")]
#[derive(Default, Debug, Clone)]
pub struct GlobalValues {
    pub mods: ModManager,
    pub mod_groups: Vec<ReflectModGroup>,

    #[reflect(alias("infos"))]
    pub gamemode_infos: GamemodeInfos,

    pub playmode: String,
    pub playmode_display: String,
    pub playmode_actual: String,
    pub playmode_actual_display: String,

    pub username: String,
    pub menu_list: Vec<String>,
    pub dialog_list: Vec<String>,

    pub new_beatmap_hash: Option<Md5Hash>,
}
impl GlobalValues {
    pub fn new(
        infos: GamemodeInfos,
        settings: &Settings,
    ) -> Self {
        let mut s = Self {
            gamemode_infos: infos,
            username: settings.username.clone(),
            ..Default::default()
        };
        s.update_playmode(&settings.last_played_mode);
        s.update_playmode_actual(&settings.last_played_mode);
        
        s
    }

    pub fn update_playmode(
        &mut self, 
        playmode: &str,
    ) {
        self.playmode = playmode.to_owned();
        let Ok(info) = self.gamemode_infos.get_info(playmode) else { return };
        self.playmode_display = info.display_name.to_owned();
    }
    pub fn update_playmode_actual(
        &mut self, 
        playmode: &str,
    ) {
        self.playmode_actual = playmode.to_owned();
        let Ok(info) = self.gamemode_infos.get_info(playmode) else { return };
        self.playmode_actual_display = info.display_name.to_owned();


        // update mod groups
        let mod_groups = ModManager::mod_groups_for_playmode(info);
        self.mod_groups = mod_groups
            .iter()
            .map(|group| ReflectModGroup::from_group(group, &self.mods))
            .collect();
    }

    pub fn update_mods(&mut self) {
        for group in self.mod_groups.iter_mut() {
            group.update(&self.mods);
        }
    }
}
