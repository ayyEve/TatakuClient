use crate::prelude::*;

#[derive(Clone)]
pub struct Hitsound {
    pub volume: f32,
    pub filename: String,
    pub filename_backup: Option<String>,
    pub allowed_sources: Vec<HitsoundSource>
}
impl Hitsound {
    pub fn new(filename: String, filename_backup: Option<String>, volume: f32, allowed_sources: Vec<HitsoundSource>) -> Self {
        Self {
            filename,
            filename_backup,
            volume,
            allowed_sources
        }
    }
    
    pub fn new_simple(filename: impl ToString) -> Self {
        Self {
            filename: filename.to_string(),
            filename_backup: None,
            volume: 1.0,
            allowed_sources: all_allowed_sources()
        }
    }

    
    pub fn from_hitsamples(hitsound: u8, mut hitsamples:HitSamples, normal_by_default: bool, timing_point: &TimingPoint) -> Vec<Self> {
        let mut play_normal = normal_by_default || (hitsound & 1) > 0; // 0: Normal
        let mut play_whistle = (hitsound & 2) > 0; // 1: Whistle
        let mut play_finish = (hitsound & 4) > 0; // 2: Finish
        let mut play_clap = (hitsound & 8) > 0; // 3: Clap
        let vol = if hitsamples.volume == 0 { timing_point.volume } else { hitsamples.volume } as f32 / 100.0;

        
        if hitsamples.normal_set == 0 {
            hitsamples.normal_set = timing_point.sample_set;
            hitsamples.index = timing_point.sample_index;
        }
        if hitsamples.addition_set == 0 {
            hitsamples.addition_set = hitsamples.normal_set;
        }

        // https://osu.ppy.sh/wiki/en/osu%21_File_Formats/Osu_%28file_format%29#hitsounds

        // normalSet and additionSet can be any of the following:
        // 0: No custom sample set
        // For normal sounds, the set is determined by the timing point's sample set.
        // For additions, the set is determined by the normal sound's sample set.
        // 1: Normal set
        // 2: Soft set
        // 3: Drum set

        // The filename is <sampleSet>-hit<hitSound><index>.wav, where:

        // sampleSet is normal, soft, or drum, determined by either normalSet or additionSet depending on which hitsound is playing
        const SAMPLE_SETS:&[&str] = &["normal", "normal", "soft", "drum"];
        // hitSound is normal, whistle, finish, or clap
        // index is the same index as above, except it is not written if the value is 0 or 1
        
        let check_beatmap = hitsamples.index != 0;
        let mut suffix = String::new();

        if check_beatmap && hitsamples.index > 1 {
            suffix = hitsamples.index.to_string();
        }

        let mut allowed_sources = vec![HitsoundSource::Skin, HitsoundSource::Default];
        if check_beatmap { allowed_sources.push(HitsoundSource::Beatmap); }

        let mut list = Vec::new();

        // if the hitsound is being overridden
        if let Some(name) = hitsamples.filename {
            if name.len() > 0 {
                #[cfg(feature="debug_hitsounds")]
                debug!("got custom sound: {}", name);

                let allowed_sources = vec![HitsoundSource::Skin, HitsoundSource::Default, HitsoundSource::Beatmap];
                list.push(Self::new(name, None, vol, allowed_sources));

                play_normal = (hitsound & 1) > 0;
                play_whistle = false;
                play_clap = false;
                play_finish = false;
            }
        }

        for (check, set, infix) in [
            (play_normal, hitsamples.normal_set, "-hitnormal"),
            (play_whistle, hitsamples.addition_set, "-hitwhistle"),
            (play_finish, hitsamples.addition_set, "-hitfinish"),
            (play_clap, hitsamples.addition_set, "-hitclap"),
        ] {
            if check {
                let sample_set = SAMPLE_SETS[set as usize % 4]; // % 4 to un-break broken maps
                let backup = if suffix.is_empty() { None } else { Some(format!("{sample_set}{infix}")) };
                list.push(Hitsound::new(format!("{sample_set}{infix}{suffix}"), backup, vol, allowed_sources.clone()));
            }
        }

        list
    }
}


fn all_allowed_sources() -> Vec<HitsoundSource> {
    vec![
        HitsoundSource::Skin,
        HitsoundSource::Beatmap,
        HitsoundSource::Default,
    ]
}