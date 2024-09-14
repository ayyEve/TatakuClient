use serde::Deserialize;
use crate::prelude::*;


// default fns for serde
fn one() -> f64 {1.0}
fn nan64() -> f64 {f64::NAN}
fn nan32() -> f32 {f32::NAN}
fn default_diff_name() -> String {"default diff name".to_owned()}


#[derive(Deserialize)]
#[serde(rename_all="PascalCase")]
pub struct QuaverBeatmap {
    pub audio_file: String,
    
    #[serde(default)]
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
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub tags: String,
    pub creator: String,
    #[serde(default="default_diff_name")]
    pub difficulty_name: String,
    #[serde(default)]
    pub description: String,

    // pub editor_layers: Vec<?>,
    // pub audio_samples: Vec<?>,
    // pub sound_effects: Vec<?>,
    
    pub timing_points: Vec<QuaverTimingPoint>,
    
    #[serde(default="one")]
    pub initial_scroll_velocity: f64,
    pub slider_velocities: Vec<QuaverSliderVelocity>,
    pub hit_objects: Vec<QuaverNote>,

    // extra info added later
    #[serde(default)]
    hash: Md5Hash,
    #[serde(default)]
    path: String,
}
impl QuaverBeatmap {
    pub fn load(path: String) -> TatakuResult<Self> {
        let lines = std::fs::read_to_string(&path)?;
        let mut s:QuaverBeatmap = serde_yaml::from_str(&lines).map_err(|e| {
            error!("error parsing quaver beatmap: {:?}", e);
            BeatmapError::InvalidFile
        })?;

        // fix svs
        for sv in s.slider_velocities.iter_mut().filter(|s| s.multiplier.is_nan()) {
            sv.multiplier = s.initial_scroll_velocity;
        }

        // fix bpms
        // skip any NaN bpms before a valid point, as we need a valid bpm to base any future bpms off of
        while !s.timing_points.is_empty() && s.timing_points[0].bpm.is_nan() { s.timing_points.remove(0); }
        if s.timing_points.is_empty() {return Err(BeatmapError::NoTimingPoints.into())}

        let first_bpm = s.timing_points.first().unwrap().bpm;
        for tp in s.timing_points.iter_mut().filter(|t|t.bpm.is_nan()) {
            tp.bpm = first_bpm;
        }

        // fix note times
        let first_timingpoint_time = s.timing_points.first().unwrap().start_time;
        for note in s.hit_objects.iter_mut().filter(|n|n.start_time.is_nan()) {
            note.start_time = first_timingpoint_time;
        }


        s.hash = Io::get_file_hash(&path)?;
        s.path = path.clone();

        let parent_dir = Path::new(&path).parent().unwrap().to_str().unwrap();
        s.audio_file = format!("{}/{}", parent_dir, s.audio_file);
        s.background_file = format!("{}/{}", parent_dir, s.background_file);
        // debug!("bg: {}", s.background_file);

        Ok(s)
    }
}
impl TatakuBeatmap for QuaverBeatmap {
    fn hash(&self) -> Md5Hash {self.hash}
    fn playmode(&self, _incoming:String) -> String {"mania".to_owned()}

    fn get_timing_points(&self) -> Vec<TimingPoint> {
        self.timing_points
            .iter()
            .map(|t| (*t).into())
            .collect()
    }

    fn get_beatmap_meta(&self) -> Arc<BeatmapMeta> {
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

        let mut meta = BeatmapMeta { 
            file_path: self.path.clone(), 
            beatmap_hash: self.hash, 
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

        Arc::new(meta)
    }

}



#[derive(Deserialize, Copy, Clone)]
pub enum QuaverKeys {
    Keys4,
    Keys5,
    Keys7,
}
impl From<QuaverKeys> for u8 {
    fn from(val: QuaverKeys) -> Self {
        match val {
            QuaverKeys::Keys4 => 4,
            QuaverKeys::Keys5 => 5,
            QuaverKeys::Keys7 => 7,
        }
    }
}


#[derive(Deserialize, Copy, Clone)]
#[serde(rename_all="PascalCase")]
pub struct QuaverTimingPoint {
    #[serde(default)]
    pub start_time: f32,
    #[serde(default="nan32")]
    pub bpm: f32
}
impl From<QuaverTimingPoint> for TimingPoint {
    fn from(val: QuaverTimingPoint) -> Self {
        TimingPoint {
            time: val.start_time,
            beat_length: 60_000.0 / val.bpm,
            ..Default::default()
        }
    }
}



#[derive(Deserialize)]
#[serde(rename_all="PascalCase")]
pub struct QuaverNote {
    #[serde(default="nan32")]
    pub start_time: f32,
    pub lane: u8,
    #[serde(default)]
    pub end_time: Option<f32>
    // key_sounds: Vec<?>
}

#[derive(Deserialize, Copy, Clone)]
#[serde(rename_all="PascalCase")]
pub struct QuaverSliderVelocity {
    #[serde(default)]
    pub start_time: f32,
    
    #[serde(default="nan64")]
    pub multiplier: f64
}