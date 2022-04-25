use serde::Deserialize;
use crate::prelude::*;

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
    
    pub initial_scroll_velocity: f64,
    pub slider_velocities: Vec<QuaverSliderVelocity>,
    pub hit_objects: Vec<QuaverNote>,

    // extra info added later
    #[serde(default)]
    hash: String,
    #[serde(default)]
    path: String,
}
impl QuaverBeatmap {
    pub fn load(path: String) -> TatakuResult<Self> {
        let lines = std::fs::read_to_string(&path)?;
        let mut s:QuaverBeatmap = serde_yaml::from_str(&lines).map_err(|e| {
            error!("asdjlfhjskdfhjdsfhksdfflhksdfhkjsdfhjdf: {:?}", e);
            BeatmapError::InvalidFile
        })?;

        for sv in s.slider_velocities.iter_mut().filter(|s| s.multiplier.is_none()) {
            sv.multiplier = Some(s.initial_scroll_velocity);
        }

        s.hash = get_file_hash(&path)?;
        s.path = path.clone();

        let parent_dir = Path::new(&path).parent().unwrap().to_str().unwrap();
        s.audio_file = format!("{}/{}", parent_dir, s.audio_file);
        s.background_file = format!("{}/{}", parent_dir, s.background_file);
        // debug!("bg: {}", s.background_file);

        Ok(s)
    }
}
impl TatakuBeatmap for QuaverBeatmap {
    fn hash(&self) -> String {self.hash.clone()}
    fn playmode(&self, _incoming:PlayMode) -> PlayMode {"mania".to_owned()}

    fn get_timing_points(&self) -> Vec<crate::beatmaps::common::TimingPoint> {
        self.timing_points
            .iter()
            .map(|t|t.clone().into())
            .collect()
    }

    fn get_beatmap_meta(&self) -> crate::beatmaps::common::BeatmapMeta {
        let cs:u8 = self.mode.into();
        let cs = cs as f32;

        let mut bpm_min = 9999999999.9;
        let mut bpm_max  = 0.0;
        for i in self.timing_points.iter() {
            if i.bpm < bpm_min {
                bpm_min = i.bpm;
            }
            if i.bpm > bpm_max {
                bpm_max = i.bpm;
            }
        }

        let mut meta = crate::beatmaps::common::BeatmapMeta { 
            file_path: self.path.clone(), 
            beatmap_hash: self.hash.clone(), 
            beatmap_type: BeatmapType::Quaver,
            mode: "mania".to_owned(), 
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
            hp: 0.0, 
            od: 0.0, 
            cs, 
            ar: 0.0, 

            bpm_min,
            bpm_max,
            diff: -1.0,
        };


        let mut start_time = 0.0;
        let mut end_time = 0.0;
        for note in self.hit_objects.iter() {
            if note.start_time < start_time {
                start_time = note.start_time
            }

            let et = note.end_time.unwrap_or(note.start_time);
            if et > end_time {
                end_time = et
            }
        }
        meta.duration = end_time - start_time;

        meta
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

#[derive(Deserialize, Copy, Clone)]
#[serde(rename_all="PascalCase")]
pub struct QuaverSliderVelocity {
    pub start_time: f32,
    pub multiplier: Option<f64>
}