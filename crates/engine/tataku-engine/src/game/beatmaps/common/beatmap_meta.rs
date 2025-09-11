use crate::*;
use common::reflect::*;
use common::Md5Hash;

// contains beatmap info unrelated to notes and timing points, etc
#[derive(Reflect)]
#[reflect(display="debug")]
#[derive(Clone, Debug, Default)]
pub struct BeatmapMeta {
    #[reflect(alias("path"))] pub file_path: ArcStr,
    #[reflect(alias("hash"))] pub beatmap_hash: Md5Hash,
    #[reflect(alias("type"))] pub beatmap_type: beatmaps::BeatmapType,

    #[reflect(alias("playmode"))] pub mode: ArcStr,
    pub artist: ArcStr,
    pub title: ArcStr,
    pub artist_unicode: ArcStr,
    pub title_unicode: ArcStr,
    pub creator: ArcStr,
    pub version: ArcStr,
    #[reflect(alias("audio_path"))] pub audio_filename: ArcStr,
    #[reflect(alias("image_path"))] pub image_filename: ArcStr,
    #[reflect(alias("preview", "preview_time"))] pub audio_preview: f32,

    pub duration: f32, // time in ms from first note to last note

    pub hp: f32,
    pub od: f32,
    pub cs: f32,
    pub ar: f32,
    pub bpm_min: f32,
    pub bpm_max: f32,
}
impl BeatmapMeta {
    pub fn new(
        file_path: String, 
        beatmap_hash: Md5Hash, 
        beatmap_type: beatmaps::BeatmapType,
    ) -> Self {
        Self {
            file_path: file_path.into(),
            beatmap_hash,
            beatmap_type,
            mode: "osu".into(),
            artist: ArcStr::unknown(),
            title: ArcStr::unknown(),
            artist_unicode: ArcStr::unknown(),
            title_unicode: ArcStr::unknown(),
            creator: ArcStr::unknown(),
            version: ArcStr::unknown(),
            hp: -1.0,
            od: -1.0,
            ar: -1.0,
            cs: -1.0,

            ..Self::default()
        }
    }

    pub fn do_checks(&mut self) {
        if self.ar < 0.0 { self.ar = self.od }
    }

    /// get the title string with the version
    pub fn version_string(&self) -> String {
        let artist = if self.artist.is_empty() { &self.artist_unicode } else { &self.artist };
        let title = if self.title.is_empty() { &self.title_unicode } else { &self.title };
        format!("{artist} - {title} [{}]", self.version)  
    }

    
    /// helper function for checking hashes
    pub fn comp_hash(&self, other: Md5Hash) -> bool {
        self.beatmap_hash == other
    }


    pub fn get_parent_dir(&self) -> Option<PathBuf> {
        Some(Path::new(&self.file_path).parent()?.to_path_buf())
    }

    pub fn filter(&self, filter_str: &str, diff: f32) -> bool {
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
                "diff"|"stars" => do_comp!(diff),

                // strings
                "game" => format!("{:?}", self.beatmap_type).to_lowercase() == val.to_lowercase(),
                "mode"|"playmode" => self.mode.to_lowercase() == val.to_lowercase(),
                "title" => self.title.to_lowercase() == val.to_lowercase() || self.title_unicode.to_lowercase() == val.to_lowercase(),
                "artist" => self.artist.to_lowercase() == val.to_lowercase() || self.artist_unicode.to_lowercase() == val.to_lowercase(),
                "creator" => self.creator.to_lowercase() == val.to_lowercase(),
                
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

// getter helpers
impl BeatmapMeta {
    pub fn mins(&self, speed: f32) -> f32 {
        ((self.duration / speed) / 60000.0).floor() 
    }
    pub fn secs(&self, speed: f32) -> f32 {
        let mins = self.mins(speed);
        let remaining_ms = (self.duration / speed) - mins * 60_000.0;
        (remaining_ms / 1000.0).floor()
    }
    
    pub fn get_hp(&self, _mods: &gameplay::mods::ModManager) -> f32 {
        self.hp
        // scale_by_mods(self.hp, 0.5, 1.4, mods).clamp(1.0, 10.0)
    }
}

impl std::cmp::PartialEq for BeatmapMeta {
    fn eq(&self, other: &Self) -> bool {
        self.beatmap_hash == other.beatmap_hash 
    }
}
