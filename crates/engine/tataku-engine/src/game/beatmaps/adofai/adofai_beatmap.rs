#![allow(dead_code)]

use crate::*;
use common::Md5Hash;
use serde::Deserialize;
use beatmaps::TimingPoint;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct AdofaiBeatmap {
    pub path_data: ArcStr,
    #[serde(default)]
    pub settings: AdofaiMapSettings,
    pub actions: Vec<AdofaiAction>,

    #[serde(default)]
    pub hash: Md5Hash,

    #[serde(default)]
    pub file_path: ArcStr,
    
    #[serde(default)]
    pub notes: Vec<AdofaiNoteDef>,
    #[serde(default, skip)]
    pub timing_points: Vec<TimingPoint>,

    #[serde(default, skip)]
    audio_file: ArcStr,
}
impl AdofaiBeatmap {
    pub fn load(path: &str) -> Self {
        let file_contents = std::fs::read_to_string(path).unwrap();

        let allowed_chars = [
            '"', '[',']', ':', '{', '}', '\\', '/', '\'', ',', '\n', ' ', '_', '.', '-', '!'
        ];

        let file_contents: String = file_contents
            .chars()
            .filter(|c| c.is_alphanumeric() || allowed_chars.contains(c))
            .collect();

        let mut map:AdofaiBeatmap = match serde_json::from_str(&file_contents) {
            Ok(m) => m,
            Err(e) => panic!("error reading adofai map '{path}': {e}"),
        };

        map.hash = tataku::Io::get_file_hash(path).unwrap();
        map.file_path = path.to_owned().into();
        
        let chars = map.path_data.chars().collect::<Vec<char>>();

        use AdofaiRotation::*;
        let mut current_time = map.settings.offset;
        let current_beatlength = 60_000.0 / map.settings.bpm;
        let mut current_direction = Clockwise;
        let mut last_char = chars[0];

        for (num, char) in chars.iter().enumerate() {
            if num == 0 {
                let _note = AdofaiNoteDef {
                    time: current_time,
                    direction: *char
                };
                continue
            }
            
            let prev_len = char2beat(last_char);
            let note_len = char2beat(*char);

            // look through events to find bpm change or direciton change
            for a in map.actions.iter() {
                if a.floor != num as u32 { continue }
                if let AdofaiEventType::Twirl = a.event_type {
                    current_direction = match current_direction {
                        Clockwise => CounterClockwise, 
                        CounterClockwise => Clockwise
                    };
                }
            }

            let diff = match current_direction {
                Clockwise => note_len - prev_len, 
                CounterClockwise => prev_len - note_len
            };
            let beat = current_beatlength * ((3.0 + diff) % 2.0);
            current_time += beat;

            // debug!("{:?}: {} -> {}, {} -> {} = {}/{}/{} ", current_direction, last_char, char, prev_len, note_len, diff, (1.0 + diff) % 2.0, beat);

            if *char == '!' {
                continue;
            }

            // add note
            let note = AdofaiNoteDef {
                time: current_time,
                direction: *char
            };
            map.notes.push(note);

            last_char = *char;
        }


        //TODO: properly add timing points
        map.timing_points.push(TimingPoint {
            time: map.settings.offset,
            beat_length: 60_000.0 / map.settings.bpm,
            ..Default::default()
        });

    
        let parent_dir = Path::new(&path).parent().unwrap();
        map.audio_file = format!(
            "{}/{}", 
            parent_dir.to_str().unwrap(), 
            map.settings.song_filename
        ).replace("\\\\", "/").into();

        map
    }
}
impl beatmaps::TatakuBeatmap for AdofaiBeatmap {
    fn hash(&self) -> Md5Hash { self.hash }

    fn get_timing_points(&self) -> Vec<TimingPoint> {
        self.timing_points.clone()
    }

    fn get_beatmap_meta(&self) -> Arc<BeatmapMeta> {
        let parent_dir = Path::new(&self.file_path);
        let parent_dir = parent_dir.parent().unwrap().to_str().unwrap();


        // let mut bpm_min = 9999999999.9;
        // let mut bpm_max  = 0.0;
        // for i in self.timing_points {
        //     if i. < bpm_min {
        //         bpm_min = i.bpm;
        //     }
        //     if i.bpm > bpm_max {
        //         bpm_max = i.bpm;
        //     }
        // }

        Arc::new(BeatmapMeta {
            file_path: self.file_path.clone(),
            beatmap_hash: self.hash(),
            beatmap_type: beatmaps::BeatmapType::Adofai,
            mode: "adofai".to_owned().into(),
            artist: self.settings.artist.clone(),
            title: self.settings.song.clone(),
            artist_unicode: self.settings.artist.clone(),
            title_unicode: self.settings.song.clone(),
            creator: self.settings.author.clone(),
            version: self.settings.song.clone(),
            audio_filename: self.audio_file.clone(),
            image_filename: format!("{}/{}", parent_dir, self.settings.bg_image).into(),
            audio_preview: self.settings.preview_song_start,
            ..Default::default()
        })
    }

    fn playmode(&self, _incoming: String) -> String {
        // TODO: 
        "taiko".to_owned()
    }

    fn slider_velocity(&self) -> f32 { 1.0 }
}

// https://github.com/Luxusio/ADOFAI-Map-Converter/blob/master/src/main/java/io/luxus/lib/adofai/type/LegacyTileAngle.java
fn char2beat(c: char) -> f32 {
    match c {
        '!' => -1.0, // hold, 8/8
        'R' => 0.0, // 8/8
        'M' => 0.128, // 1/8
        'C' => 0.25, // 2/8
        'B' => 0.375, // 3/8
        'D' => 0.5, // 4/8
        'V' => 0.625, // 5/8
        'Z' => 0.75, // 6/8
        'N' => 0.875, // 7/8
        'L' => 1.0, // 8/8 but backwards
        'H' => 1.125, // 9/8
        'Q' => 1.25, // 10/8
        'T' => 1.375, // 11/8
        'U' => 1.5, // 12/8
        'Y' => 1.625, // 13/8
        'E' => 1.75, // 14/8
        'J' => 1.875, // 15/8

        _ => {
            warn!("unknown char '{c}'");
            0.0
        }
    }
}

struct Angle {
    /// degrees
    angle: f32,
    relative: bool,
}
impl Angle {
    fn new(angle: f32, relative: bool) -> Self {
        Self {
            angle,
            relative
        }
    }

    fn from_char(c: char) -> Option<Self> {
        match c {
            'R' => Some(Self::new(0.0, false)),
            'p' => Some(Self::new(15.0, false)),
            'J' => Some(Self::new(30.0, false)),
            'E' => Some(Self::new(45.0, false)),
            'T' => Some(Self::new(60.0, false)),
            'o' => Some(Self::new(75.0, false)),
            'U' => Some(Self::new(90.0, false)),
            'q' => Some(Self::new(105.0, false)),
            'G' => Some(Self::new(120.0, false)),
            'Q' => Some(Self::new(135.0, false)),
            'H' => Some(Self::new(150.0, false)),
            'W' => Some(Self::new(165.0, false)),
            'L' => Some(Self::new(180.0, false)),
            'x' => Some(Self::new(195.0, false)),
            'N' => Some(Self::new(210.0, false)),
            'Z' => Some(Self::new(225.0, false)),
            'F' => Some(Self::new(240.0, false)),
            'V' => Some(Self::new(255.0, false)),
            'D' => Some(Self::new(270.0, false)),
            'Y' => Some(Self::new(285.0, false)),
            'B' => Some(Self::new(300.0, false)),
            'C' => Some(Self::new(315.0, false)),
            'M' => Some(Self::new(330.0, false)),
            'A' => Some(Self::new(345.0, false)),
            '5' => Some(Self::new(108.0, true)),
            '6' => Some(Self::new(252.0, true)),
            '7' => Some(Self::new(900.0 / 7.0, true)),
            '8' => Some(Self::new(360.0 - 900.0 / 7.0, true)),
            't' => Some(Self::new(60.0, true)),
            'h' => Some(Self::new(120.0, true)),
            'j' => Some(Self::new(240.0, true)),
            'y' => Some(Self::new(300.0, true)),
            '!' => Some(Self::new(0.0, true)),
            _ => None,
        }
    }
}



#[derive(Deserialize, Default)]
#[serde(rename_all="camelCase", default)]
pub struct AdofaiNoteDef {
    pub time: f32,
    pub direction: char
}


#[derive(Copy, Clone, Debug)]
pub enum AdofaiRotation {
    Clockwise,
    CounterClockwise
}


#[derive(Deserialize, Default)]
#[serde(rename_all="camelCase", default)]
pub struct AdofaiMapSettings {
    version: u8,
    artist: ArcStr,
    special_artist_type: ArcStr,
    artist_permission: ArcStr,
    /// song title
    song: ArcStr,
    author: ArcStr,
    separate_countdown_time: Enabled,

    preview_image: ArcStr,
    preview_icon: ArcStr,
    preview_icon_color: ArcStr,
    preview_song_start: f32,
    preview_song_duration: f32,
    seizure_warning: Enabled,

    level_desc: ArcStr,
    level_tags: ArcStr,
    artist_links: ArcStr,

    difficulty: f32,
    song_filename: ArcStr,
    bpm: f32,
    volume: u8,
    offset: f32,
    pitch: f32,

    hitsound: ArcStr,
    hitsound_volume: u8,
    countdown_ticks: u8,

    track_color_type: ArcStr,
    track_color: ArcStr,

    secondary_track_color: ArcStr,
    track_color_anim_duration: f32,
    track_color_pulse: ArcStr,
    track_color_pulse_length: f32,
    track_style: ArcStr,
    track_animation: ArcStr,
    beats_ahead: u8,
    track_dissapear_animation: ArcStr,

    beats_behind: u8,
    background_color: ArcStr,
    bg_image: ArcStr,
    bg_image_color: ArcStr,
    parallax: [f32;2],

    bg_display_mode: ArcStr,
    /// lock rotation
    lock_rot: Enabled,
    loop_bg: Enabled,

    unscaled_size: f32,
    relative_to: ArcStr,

    position: [f32; 2],
    rotation: f32,
    zoom: f32,
    bg_video: ArcStr,
    loop_video: Enabled,
    vid_offset: f32,
    floor_icon_outlines: Enabled,
    stick_to_floors: Enabled,
    planet_ease: ArcStr,
    planet_ease_parts: u8,
    legacy_flash: bool
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct AdofaiAction {
    floor: u32,
    event_type: AdofaiEventType,

    // for RepeatEvents event
    repetitions: Option<u32>,
    interval: Option<f32>,
    tag: Option<String>,


    // for SetSpeed event
    speed_type: Option<String>,
    beats_per_minute: Option<f32>,
    bpm_multiplier: Option<f32>
}

#[derive(Deserialize, Copy, Clone)]
pub enum AdofaiEventType {
    Twirl,
    RepeatEvents,
    SetConditionalEvents,
    Checkpoint,
    SetHitsound,
    SetPlanetRotation,
    SetSpeed,
    MoveCamera,
    MoveTrack,
    Flash,
    SetFilter,
    Bloom,
    PositionTrack,
    ShakeScreen,

    AddDecoration,
    MoveDecorations,

    ColorTrack,
    RecolorTrack,
    AnimateTrack,
}

#[derive(Deserialize, Default)]
pub enum Enabled {
    Enabled,
    
    #[default] Disabled
}
impl From<Enabled> for bool {
    fn from(val: Enabled) -> Self {
        match val {
            Enabled::Enabled => true,
            Enabled::Disabled => false,
        }
    }
}
