use std::path::Path;

use serde::Deserialize;
use taiko_rs_common::types::PlayMode;

use crate::beatmaps::common::TaikoRsBeatmap;



#[derive(Deserialize)]
#[serde(rename_all="PascalCase")]
pub struct QuaverBeatmap {
    pub audio_file: String,
    pub song_preview_time: f32,
    pub background_file: String,

    // dunno if they can be negative
    #[serde(default)]
    pub map_id: i32,
    #[serde(default)]
    pub set_id: i32,

    pub mode: QuaverKeys,

    pub title: String,
    pub artist: String,
    pub source: String,
    pub tags: String,
    pub creator: String,
    pub difficulty_name: String,
    pub description: String,

    // pub editor_layers: Vec<?>,
    // pub audio_samples: Vec<?>,
    // pub sound_effects: Vec<?>,
    
    pub timing_points: Vec<QuaverTimingPoint>,
    // pub slider_velocities: Vec<?>,
    pub hit_objects: Vec<QuaverNote>,

    // extra info added later
    #[serde(default)]
    hash: String,
    #[serde(default)]
    path: String,
}
impl QuaverBeatmap {
    pub fn load(path: String) -> Self {
        let lines = std::fs::read_to_string(&path).unwrap();
        let mut s:QuaverBeatmap = serde_yaml::from_str(&lines).unwrap();

        s.hash = crate::helpers::io::get_file_hash(&path).unwrap();
        s.path = path.clone();

        let parent_dir = Path::new(&path).parent().unwrap();
        s.audio_file = format!("{}/{}", parent_dir.to_str().unwrap(), s.audio_file);

        s
    }
}
impl TaikoRsBeatmap for QuaverBeatmap {
    fn hash(&self) -> String {self.hash.clone()}

    fn get_timing_points(&self) -> Vec<crate::beatmaps::common::TimingPoint> {
        self.timing_points
            .iter()
            .map(|t|t.clone().into())
            .collect()
    }

    fn get_beatmap_meta(&self) -> crate::beatmaps::common::BeatmapMeta {
        crate::beatmaps::common::BeatmapMeta { 
            file_path: self.path.clone(), 
            beatmap_hash: self.hash.clone(), 
            beatmap_version: 0, 
            mode: PlayMode::Mania, 
            artist: self.artist.clone(), 
            title: self.title.clone(), 
            artist_unicode: self.artist.clone(), 
            title_unicode: self.title.clone(), 
            creator: self.creator.clone(), 
            version: self.difficulty_name.clone(), 
            audio_filename: self.audio_file.clone(), 
            image_filename: self.background_file.clone(), 
            audio_preview: self.song_preview_time, 
            duration: 0.0, 
            mins: 0, 
            secs: 0, 
            hp: 0.0, 
            od: 0.0, 
            cs: 0.0, 
            ar: 0.0, 
            slider_multiplier: 1.0, 
            slider_tick_rate: 1.0
        }
    }

    fn playmode(&self, _incoming:taiko_rs_common::types::PlayMode) -> taiko_rs_common::types::PlayMode {
        taiko_rs_common::types::PlayMode::Mania
    }

    fn slider_velocity_at(&self, time:f32) -> f32 {
        let bl = self.beat_length_at(time, true);
        100.0 * 1.4 * if bl > 0.0 {1000.0 / bl} else {1.0}
    }

    fn beat_length_at(&self, time:f32, _allow_multiplier:bool) -> f32 {
        for qtp in self.timing_points.iter() {
            if qtp.start_time >= time {
                return 60_000.0 / qtp.bpm
            }
        }
        60_000.0 / self.timing_points[0].bpm
    }

    fn control_point_at(&self, time:f32) -> crate::beatmaps::common::TimingPoint {
        for qtp in self.timing_points.iter() {
            if qtp.start_time >= time {
                return (*qtp).into()
            }
        }

        self.timing_points[0].into()
    }
}



#[derive(Deserialize, Copy, Clone)]
pub enum QuaverKeys {
    Keys4,
    Keys5,
    Keys7,
}
impl Into<u8> for QuaverKeys {
    fn into(self) -> u8 {
        match self {
            QuaverKeys::Keys4 => 4,
            QuaverKeys::Keys5 => 5,
            QuaverKeys::Keys7 => 7,
        }
    }
}


#[derive(Deserialize, Copy, Clone)]
#[serde(rename_all="PascalCase")]
pub struct QuaverTimingPoint {
    pub start_time: f32,
    pub bpm: f32
}
impl Into<crate::beatmaps::common::TimingPoint> for QuaverTimingPoint {
    fn into(self) -> crate::beatmaps::common::TimingPoint {
        crate::beatmaps::common::TimingPoint {
            time: self.start_time,
            beat_length: 60_000.0 / self.bpm,
            ..Default::default()
        }
    }
}



#[derive(Deserialize)]
#[serde(rename_all="PascalCase")]
pub struct QuaverNote {
    pub start_time: f32,
    pub lane: u8,
    #[serde(default)]
    pub end_time: Option<f32>
    // key_sounds: Vec<?>
}