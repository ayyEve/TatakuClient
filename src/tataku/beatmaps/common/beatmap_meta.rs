use crate::prelude::*;

// contains beatmap info unrelated to notes and timing points, etc
#[derive(Clone, Debug, Default)]
pub struct BeatmapMeta {
    pub file_path: String,
    pub beatmap_hash: String,
    pub beatmap_type: BeatmapType,

    pub mode: String,
    pub artist: String,
    pub title: String,
    pub artist_unicode: String,
    pub title_unicode: String,
    pub creator: String,
    pub version: String,
    pub audio_filename: String,
    pub image_filename: String,
    pub audio_preview: f32,

    pub duration: f32, // time in ms from first note to last note

    pub hp: f32,
    pub od: f32,
    pub cs: f32,
    pub ar: f32,
    pub bpm_min: f32,
    pub bpm_max: f32,
}
impl BeatmapMeta {
    pub fn new(file_path:String, beatmap_hash:String, beatmap_type:BeatmapType) -> BeatmapMeta {
        let unknown = "Unknown".to_owned();

        BeatmapMeta {
            file_path,
            beatmap_hash,
            beatmap_type,
            mode: "osu".to_owned(),
            artist: unknown.clone(),
            title: unknown.clone(),
            artist_unicode: unknown.clone(),
            title_unicode: unknown.clone(),
            creator: unknown.clone(),
            version: unknown.clone(),
            audio_filename: String::new(),
            image_filename: String::new(),
            audio_preview: 0.0,
            hp: -1.0,
            od: -1.0,
            ar: -1.0,
            cs: -1.0,

            duration: 0.0,
            bpm_min: 0.0,
            bpm_max: 0.0
        }
    }

    pub fn do_checks(&mut self) {
        if self.ar < 0.0 { self.ar = self.od }
    }

    /// get the title string with the version
    pub fn version_string(&self) -> String {
        format!("{} - {} [{}]", self.artist, self.title, self.version)  
    }


    pub fn check_mode_override(&self, override_mode:String) -> String {
        if self.mode == "osu" {
            override_mode
        } else {
            self.mode.clone()
        }
    }
}

// getter helpers
impl BeatmapMeta {
    pub fn mins(&self, speed:f32) -> f32 {
        ((self.duration / speed) / 60000.0).floor() 
    }
    pub fn secs(&self, speed:f32) -> f32 {
        let mins = self.mins(speed);
        let remaining_ms = (self.duration / speed) - mins * 60000.0;
        (remaining_ms / 1000.0).floor()
    }
    
    pub fn get_hp(&self, _mods: &ModManager) -> f32 {
        self.hp
        // scale_by_mods(self.hp, 0.5, 1.4, mods).clamp(1.0, 10.0)
    }

}


pub struct BeatmapMetaWithDiff {
    meta: Arc<BeatmapMeta>,
    pub sort_pending: bool,
    
    pub diff: Option<f32>,
}
impl BeatmapMetaWithDiff {
    pub fn new(meta: Arc<BeatmapMeta>, diff: Option<f32>) -> Self {
        Self { 
            diff, 
            meta, 
            sort_pending: true,
        }
    }
    pub fn _set_diff(&mut self, new_diff: Option<f32>) {
        self.diff = new_diff
    }

    pub fn filter(&self, filter_str: &str) -> bool {
        const COMPS:&[&str] = &[">=","<=",">", "<", "="];
        let mut comp = None;
        for c in COMPS {
            if filter_str.contains(c) {
                comp = Some(*c);
                break;
            }
        }

        if let Some(comp) = comp {
            let mut split = filter_str.split(comp);
            let key = split.next().unwrap();
            let val = split.next().unwrap_or_default();

            macro_rules! do_comp {
                ($check:expr) => {{
                    let val = val.parse().unwrap_or_default();
                    match comp {
                        ">=" => $check >= val,
                        "<=" => $check <= val,
                        ">" => $check > val,
                        "<" => $check < val,
                        "=" => $check == val,
                        // anything else is wrong,
                        _ => false,
                    }
                }}
            }
            return match key {
                // numbers
                "bpm" => do_comp!(self.bpm_min),
                "diff"|"stars" => do_comp!(self.diff.unwrap_or_default()),

                // strings
                "game" => format!("{:?}", self.beatmap_type).to_lowercase() == val.to_lowercase(),
                "mode"|"playmode" => self.mode.to_lowercase() == val.to_lowercase(),
                
                // pain
                _ => true,
            }
        }

        self.artist.to_ascii_lowercase().contains(filter_str) 
        || self.artist_unicode.to_ascii_lowercase().contains(filter_str) 
        || self.title.to_ascii_lowercase().contains(filter_str) 
        || self.title_unicode.to_ascii_lowercase().contains(filter_str) 
        || self.creator.to_ascii_lowercase().contains(filter_str) 
        || self.version.to_ascii_lowercase().contains(filter_str) 
    }

}

impl Deref for BeatmapMetaWithDiff {
    type Target = Arc<BeatmapMeta>;

    fn deref(&self) -> &Self::Target {
        &self.meta
    }
}
