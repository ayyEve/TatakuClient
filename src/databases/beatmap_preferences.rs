/**
 * This is used to store map and map-mode preferences. 
 */

use crate::prelude::*;

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct BeatmapPreferences {
    pub audio_offset: f32,

    // not yet implemented
    pub background_video: bool,
    // not yet implemented
    pub storyboard: bool,
}
impl BeatmapPreferences {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            audio_offset: row.get("audio_offset")?,
            background_video: row.get("background_video")?,
            storyboard: row.get("storyboard")?,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct BeatmapPlaymodePreferences {
    pub scroll_speed: f32,
}
impl BeatmapPlaymodePreferences {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            scroll_speed: row.get("scroll_speed")?,
        })
    }
}
impl Default for BeatmapPlaymodePreferences {
    fn default() -> Self {
        Self { 
            scroll_speed: 1.0,
        }
    }
}


impl Database {
    pub async fn get_beatmap_prefs(map_hash:&String) -> BeatmapPreferences {
        let db = Self::get().await;

        let query = format!("SELECT * FROM beatmap_preferences WHERE beatmap_hash='{map_hash}'");
        let mut s = db.prepare(&query).unwrap();
        let res = s.query_map([], BeatmapPreferences::from_row);

        if let Ok(mut rows) = res {
            rows.find_map(|r|r.ok()).unwrap_or_default()
        } else {
            Default::default()
        }
    }
    pub fn save_beatmap_prefs(map_hash:&String, prefs: &BeatmapPreferences) {
        let BeatmapPreferences{ audio_offset, background_video, storyboard } = prefs;

        Self::add_query(DatabaseQuery::InsertOrUpdate { 
            sql: format!("INSERT INTO beatmap_preferences (beatmap_hash, audio_offset, background_video, storyboard) VALUES ('{map_hash}', {audio_offset}, {background_video}, {storyboard})"), 
            table_name: "beatmap_preferences".to_owned(), 
            operation: "INSERT".to_owned(), 
            sql_if_failed: Some(format!("UPDATE beatmap_preferences SET audio_offset={audio_offset}, background_video={background_video}, storyboard={storyboard} WHERE beatmap_hash='{map_hash}'")), 
            operation_if_failed: Some("UPDATE".to_owned()) 
        });
    }

    pub async fn get_beatmap_mode_prefs(map_hash:&String, playmode:&PlayMode) -> BeatmapPlaymodePreferences {
        let db = Self::get().await;

        let query = format!("SELECT * FROM beatmap_mode_preferences WHERE beatmap_hash='{map_hash}' AND playmode='{playmode}'");
        let mut s = db.prepare(&query).unwrap();
        let res = s.query_map([], BeatmapPlaymodePreferences::from_row);

        if let Ok(mut rows) = res {
            rows.find_map(|r|r.ok()).unwrap_or_default()
        } else {
            Default::default()
        }
    }
    pub fn save_beatmap_mode_prefs(map_hash:&String, playmode:&PlayMode, prefs:&BeatmapPlaymodePreferences) {
        let BeatmapPlaymodePreferences { scroll_speed } = prefs;

        Self::add_query(DatabaseQuery::InsertOrUpdate { 
            sql: format!("INSERT INTO beatmap_mode_preferences (beatmap_hash, playmode, scroll_speed) VALUES ('{map_hash}', '{playmode}', {scroll_speed})"), 
            table_name: "beatmap_mode_preferences".to_owned(), 
            operation: "INSERT".to_owned(), 
            sql_if_failed: Some(format!("UPDATE beatmap_mode_preferences SET scroll_speed={scroll_speed} WHERE beatmap_hash='{map_hash}' AND playmode='{playmode}'")), 
            operation_if_failed: Some("UPDATE".to_owned()) 
        });
    }

}
