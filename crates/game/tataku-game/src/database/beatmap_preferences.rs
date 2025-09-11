/*
 * This is used to store map and map-mode preferences. 
 */

use crate::prelude::*;
use common::reflect::*;

#[derive(Settings, Reflect)]
#[derive(Serialize, Deserialize)]
#[derive(Clone, Default2, Debug, PartialEq)]
#[serde(default)]
pub struct BeatmapPreferences {
    #[setting(text = "Audio Offset", range(-500.0, 500.0))]
    pub audio_offset: f32,
    
    #[default(true)]
    #[setting(text = "Storyboard")]
    pub storyboard: bool,

    #[default(true)]
    #[setting(text = "Beatmap Skin")]
    pub beatmap_skin: bool,

    // not yet implemented
    pub background_video: bool,
}
impl BeatmapPreferences {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            audio_offset: row.get("audio_offset")?,
            background_video: row.get("background_video")?,
            beatmap_skin: row.get("beatmap_skin").unwrap_or_default(),
            storyboard: row.get("storyboard")?,
        })
    }
}

#[derive(Settings, Reflect)]
#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug, Default2, PartialEq)]
#[serde(default)]
pub struct BeatmapPlaymodePreferences {
    #[default(1.0)]
    pub scroll_speed: f32,
}
impl BeatmapPlaymodePreferences {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            scroll_speed: row.get("scroll_speed")?,
        })
    }
}


impl Database {
    pub fn get_beatmap_prefs(map_hash: common::Md5Hash) -> BeatmapPreferences {
        let db = Self::get();

        let query = format!("SELECT * FROM beatmap_preferences WHERE beatmap_hash='{map_hash}'");
        let mut s = db.prepare(&query).unwrap();
        let res = s.query_map([], BeatmapPreferences::from_row);

        if let Ok(mut rows) = res {
            rows.find_map(|r| r.ok()).unwrap_or_default()
        } else {
            BeatmapPreferences::default()
        }
    }
    pub fn save_beatmap_prefs(
        map_hash: common::Md5Hash, 
        prefs: &BeatmapPreferences,
    ) {
        let BeatmapPreferences { 
            audio_offset, 
            background_video, 
            storyboard, 
            beatmap_skin,
        } = prefs;
        let map_hash = Box::new(map_hash.to_string());

        Self::insert_or_update(
            "beatmap_preferences", 
            SqlOperation::new(
                "INSERT INTO beatmap_preferences (beatmap_hash, audio_offset, background_video, storyboard, beatmap_skin) VALUES (?1, ?2, ?3, ?4, ?5)", 
                "INSERT",
                vec![
                    map_hash.clone().into(), 
                    audio_offset.into(),
                    background_video.into(),
                    storyboard.into(),
                    beatmap_skin.into(),
                ]
            ),
            Some(SqlOperation::new(
                "UPDATE beatmap_preferences 
                    SET audio_offset=?2, background_video=?3, storyboard=?4, beatmap_skin=?5
                    WHERE beatmap_hash=?1
                ", 
                "UPDATE",
                vec![
                    map_hash.clone().into(), 
                    audio_offset.into(),
                    background_video.into(),
                    storyboard.into(),
                    beatmap_skin.into(),
                ]
            ))
        );
    }

    pub fn get_beatmap_mode_prefs(
        map_hash: common::Md5Hash, 
        playmode: &str
    ) -> BeatmapPlaymodePreferences {
        let db = Self::get();
        let map_hash = map_hash.to_string();

        let query = "SELECT * FROM beatmap_mode_preferences WHERE beatmap_hash=?1 AND playmode=?2";
        let mut s = db.prepare(query).unwrap();
        let res = s.query_map(
            [
                &map_hash,
                playmode
            ], 
            BeatmapPlaymodePreferences::from_row
        );

        if let Ok(mut rows) = res {
            rows.find_map(|r| r.ok()).unwrap_or_default()
        } else {
            BeatmapPlaymodePreferences::default()
        }
    }
    pub fn save_beatmap_mode_prefs(
        map_hash: common::Md5Hash, 
        playmode: &str, 
        prefs: &BeatmapPlaymodePreferences
    ) {
        let BeatmapPlaymodePreferences { scroll_speed } = prefs;
        let map_hash = Box::new(map_hash.to_string());
        // let playmode = Box::new(playmode.to_string());

        Self::insert_or_update(
            "beatmap_preferences", 
            SqlOperation::new(
                "INSERT INTO beatmap_mode_preferences (beatmap_hash, playmode, scroll_speed) VALUES (?1, ?2, ?3, ?4, ?5)", 
                "INSERT",
                vec![
                    (&map_hash).into(), 
                    (&playmode).into(),
                    scroll_speed.into(),
                ]
            ),
            Some(SqlOperation::new(
                "UPDATE beatmap_mode_preferences 
                    SET scroll_speed=?3, 
                    WHERE beatmap_hash=?1 AND playmode=?2", 
                "UPDATE",
                vec![
                    (&map_hash).into(), 
                    (&playmode).into(),
                    scroll_speed.into(),
                ]
            ))
        );
    }

}
