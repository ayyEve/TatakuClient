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

    // remove these at some point
    pub diff: f32
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
            bpm_max: 0.0,
            diff: -1.0
        }
    }

    pub fn do_checks(&mut self) {
        if self.ar < 0.0 {self.ar = self.od}
    }

    /// get the title string with the version
    pub fn version_string(&self) -> String {
        format!("{} - {} [{}]", self.artist, self.title, self.version)  
    }

    /// get the difficulty string (od, hp, sr, bpm, len)
    pub fn diff_string(&mut self, mods: &ModManager) -> String {
        let symb = if mods.speed > 1.0 {"+"} else if mods.speed < 1.0 {"-"} else {""};

        let mut secs = format!("{}", self.secs(mods.speed));
        if secs.len() == 1 {secs = format!("0{}", secs)}

        let mut txt = format!(
            "OD: {:.2}{} HP: {:.2}{}, Len: {}:{}", 
            self.get_od(mods), symb,
            self.get_hp(mods), symb,
            self.mins(mods.speed), secs
        );

        // make sure at least one has a value
        if self.bpm_min != 0.0 || self.bpm_max != 0.0 {
            // one bpm
            if self.bpm_min == self.bpm_max {
                txt += &format!(" BPM: {:.2}", self.bpm_min * mods.speed);
            } else { // multi bpm
                // i think i had it backwards when setting, just make sure its the right way :/
                let min = self.bpm_min.min(self.bpm_max);
                let max = self.bpm_max.max(self.bpm_min);
                txt += &format!(" BPM: {:.2}-{:.2}", min * mods.speed, max * mods.speed);
            }
        }

        let diff = if self.diff < 0.0 {"...".to_owned()} else {format!("{:.2}", self.diff)};
        txt += &format!(", Diff: {}", diff);

        txt
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
                "diff"|"stars" => do_comp!(self.diff),

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
    fn mins(&self, speed:f32) -> f32 {
        ((self.duration / speed) / 60000.0).floor() 
    }
    fn secs(&self, speed:f32) -> f32 {
        let mins = self.mins(speed);
        let remaining_ms = (self.duration / speed) - mins * 60000.0;
        (remaining_ms / 1000.0).floor()
    }


    pub fn get_hp(&self, mods: &ModManager) -> f32 {
        scale_by_mods(self.hp, 0.5, 1.4, mods).clamp(1.0, 10.0)
    }
    pub fn get_od(&self, mods: &ModManager) -> f32 {
        scale_by_mods(self.od, 0.5, 1.4, mods).clamp(1.0, 10.0)
    }
    pub fn get_cs(&self, mods: &ModManager) -> f32 {
        scale_by_mods(self.cs, 0.5, 1.3, mods).clamp(1.0, 10.0)
    }
    pub fn get_ar(&self, mods: &ModManager) -> f32 {
        scale_by_mods(self.ar, 0.5, 1.4, mods).clamp(1.0, 11.0)
    }

}

#[inline]
fn scale_by_mods<V:std::ops::Mul<Output=V>>(val:V, ez_scale: V, hr_scale: V, mods: &ModManager) -> V {
    if mods.easy {val * ez_scale} else if mods.hard_rock {val * hr_scale} else {val}
}

// might use this later idk
// pub trait IntoSets {
//     fn sort_into_sets(&self) -> Vec<Vec<BeatmapMeta>>;
// }
// impl IntoSets for Vec<BeatmapMeta> {
//     fn sort_into_sets(&self) -> Vec<Vec<BeatmapMeta>> {
//         todo!()
//     }
// }

