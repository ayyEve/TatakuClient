use tataku_engine::*;

use engine::data;
use crate::wrapper::Wrapper;

use redb::{
    TableDefinition,
    ReadableDatabase as _,
};

const BEATMAP_PREFERENCES:TableDefinition<String, Wrapper<data::BeatmapPreferences>> = TableDefinition::new("beatmap_preferences");
const BEATMAP_PLAYMODE_PREFERENCES:TableDefinition<String, Wrapper<data::BeatmapPlaymodePreferences>> = TableDefinition::new("beatmap_playmode_preferences");

fn playmode_pref_key(map: common::Md5Hash, playmode: &str) -> String {
    format!("{map}-{playmode}")
}
impl engine::database::BeatmapPreferencesProvider for crate::Database {
    fn get_beatmap_preferences(
        &self, 
        map: common::Md5Hash,
    ) -> tataku::Result<data::BeatmapPreferences> {
        let read = self.0
            .begin_read()
            .map_err(Self::transaction_err)?;

        let table = read
            .open_table(BEATMAP_PREFERENCES)
            .map_err(Self::table_err)?;

        let key = map.to_string();
        let pref = table
            .get(key)
            .map_err(Self::storage_err)?
            .map(|i| i.value())
            .unwrap_or_default();

        Ok(pref)
    }

    fn set_beatmap_preferences(
        &mut self, 
        map: common::Md5Hash,
        prefs: &data::BeatmapPreferences,
    ) -> tataku::Result<()> {
        let write = self.0
            .begin_write()
            .map_err(Self::transaction_err)?;

        let mut table = write
            .open_table(BEATMAP_PREFERENCES)
            .map_err(Self::table_err)?;

        let key = map.to_string();
        table
            .insert(key, prefs)
            .map_err(Self::storage_err)?;

        Ok(())
    }

    fn get_beatmap_playmode_preferences(
        &self, 
        map: common::Md5Hash,
        playmode: &str,
    ) -> tataku::Result<data::BeatmapPlaymodePreferences> {
        let read = self.0
            .begin_read()
            .map_err(Self::transaction_err)?;

        let table = read
            .open_table(BEATMAP_PLAYMODE_PREFERENCES)
            .map_err(Self::table_err)?;

        let key = playmode_pref_key(map, playmode);
        let pref = table
            .get(key)
            .map_err(Self::storage_err)?
            .map(|i| i.value())
            .unwrap_or_default();

        Ok(pref)
    }

    fn set_beatmap_playmode_preferences(
        &mut self, 
        map: common::Md5Hash,
        playmode: &str,
        prefs: &data::BeatmapPlaymodePreferences
    ) -> tataku::Result<()> {
        let write = self.0
            .begin_write()
            .map_err(Self::transaction_err)?;

        let mut table = write
            .open_table(BEATMAP_PLAYMODE_PREFERENCES)
            .map_err(Self::table_err)?;

        let key = playmode_pref_key(map, playmode);
        table
            .insert(key, prefs)
            .map_err(Self::storage_err)?;

        Ok(())
    }
}
