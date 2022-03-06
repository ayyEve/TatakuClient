#![allow(dead_code)]

use crate::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct AdofaiBeatmap {
    pub path_data: String,
    #[serde(default)]
    pub settings: AdofaiMapSettings,
    pub actions: Vec<AdofaiAction>,

    #[serde(default)]
    pub hash: String,

    #[serde(default)]
    pub file_path: String,
    
    #[serde(default)]
    pub notes: Vec<AdofaiNoteDef>,
    #[serde(default, skip)]
    pub timing_points: Vec<TimingPoint>,

    #[serde(default, skip)]
    audio_file: String,
}
impl AdofaiBeatmap {
    pub fn load(path: String) -> Self {
        let file_contents = std::fs::read_to_string(&path).unwrap();

        let allowed_chars = [
            '"', '[',']', ':', '{', '}', '\\', '/', '\'', ',', '\n', ' ', '_', '.', '-', '!'
        ];

        let file_contents:String = file_contents.chars().filter(|c|c.is_alphanumeric() || allowed_chars.contains(&c)).collect();

        let mut map:AdofaiBeatmap = match serde_json::from_str(&file_contents) {
            Ok(m) => m,
            Err(e) => panic!("error reading adofai map '{}': {}", path, e),
        };

        map.hash = get_file_hash(&path).unwrap();
        map.file_path = path.clone();
        
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
                if a.floor != num as u32 {continue}
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

            // println!("{:?}: {} -> {}, {} -> {} = {}/{}/{} ", current_direction, last_char, char, prev_len, note_len, diff, (1.0 + diff) % 2.0, beat);

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
        map.audio_file = format!("{}/{}", parent_dir.to_str().unwrap(), map.settings.song_filename).replace("\\\\", "/");

        map
    }
}
impl TatakuBeatmap for AdofaiBeatmap {
    fn hash(&self) -> String {self.hash.clone()}

    fn get_timing_points(&self) -> Vec<TimingPoint> {
        self.timing_points.clone()
    }

    fn get_beatmap_meta(&self) -> crate::beatmaps::common::BeatmapMeta {
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

        crate::beatmaps::common::BeatmapMeta {
            file_path: self.file_path.clone(),
            beatmap_hash: self.hash(),
            beatmap_version: 10,
            mode: "adofai".to_owned(),
            artist: self.settings.artist.clone(),
            title: self.settings.song.clone(),
            artist_unicode: self.settings.artist.clone(),
            title_unicode: self.settings.song.clone(),
            creator: self.settings.author.clone(),
            version: self.settings.song.clone(),
            audio_filename: self.audio_file.clone(),
            image_filename: format!("{}/{}", parent_dir, self.settings.bg_image),
            audio_preview: self.settings.preview_song_start,
            duration: 0.0,
            hp: 0.0,
            od: 0.0,
            cs: 0.0,
            ar: 0.0,
            slider_multiplier: 1.0,
            slider_tick_rate: 1.0,
            stack_leniency: 0.0,
            bpm_min: 0.0,
            bpm_max: 0.0,
            
            diff: -1.0,
        }
    }

    fn playmode(&self, _incoming:PlayMode) -> PlayMode {
        //TODO
        "taiko".to_owned()
    }

    fn slider_velocity_at(&self, time:f32) -> f32 {
        let bl = self.beat_length_at(time, true);
        100.0 * 1.4 * if bl > 0.0 {1000.0 / bl} else {1.0}
    }

    fn beat_length_at(&self, time:f32, allow_multiplier:bool) -> f32 {
        if self.timing_points.len() == 0 {return 0.0}

        let mut point: Option<TimingPoint> = Some(self.timing_points.as_slice()[0].clone());
        let mut inherited_point: Option<TimingPoint> = None;

        for tp in self.timing_points.as_slice() {
            if tp.time <= time {
                if tp.is_inherited() {
                    inherited_point = Some(tp.clone());
                } else {
                    point = Some(tp.clone());
                }
            }
        }

        let mut mult = 1.0;
        let p = point.unwrap();

        if allow_multiplier && inherited_point.is_some() {
            let ip = inherited_point.unwrap();

            if p.time <= ip.time && ip.beat_length < 0.0 {
                mult = (-ip.beat_length).clamp(10.0, 1000.0) / 100.0;
            }
        }

        p.beat_length * mult
    }

    fn control_point_at(&self, time:f32) -> crate::beatmaps::common::TimingPoint {
        if self.timing_points.len() == 0 {panic!("beatmap has no timing points!")}

        let mut point = self.timing_points[0];
        for tp in self.timing_points.iter() {
            if tp.time <= time {point = *tp}
        }
        
        point
    }
}

fn char2beat(c:char) -> f32 {
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
            println!("unknown char '{}'", &c);
            0.0
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
    artist: String,
    special_artist_type: String,
    artist_permission: String,
    /// song title
    song: String,
    author: String,
    separate_countdown_time: Enabled,

    preview_image: String,
    preview_icon: String,
    preview_icon_color: String,
    preview_song_start: f32,
    preview_song_duration: f32,
    seizure_warning: Enabled,

    level_desc: String,
    level_tags: String,
    artist_links: String,

    difficulty: f32,
    song_filename: String,
    bpm: f32,
    volume: u8,
    offset: f32,
    pitch: f32,

    hitsound: String,
    hitsound_volume: u8,
    countdown_ticks: u8,

    track_color_type: String,
    track_color: String,

    secondary_track_color: String,
    track_color_anim_duration: f32,
    track_color_pulse: String,
    track_color_pulse_length: f32,
    track_style: String,
    track_animation: String,
    beats_ahead: u8,
    track_dissapear_animation: String,

    beats_behind: u8,
    background_color: String,
    bg_image: String,
    bg_image_color: String,
    parallax: [f32;2],

    bg_display_mode: String,
    /// lock rotation
    lock_rot: Enabled,
    loop_bg: Enabled,

    unscaled_size: f32,
    relative_to: String,

    position: [f32; 2],
    rotation: f32,
    zoom: f32,
    bg_video: String,
    loop_video: Enabled,
    vid_offset: f32,
    floor_icon_outlines: Enabled,
    stick_to_floors: Enabled,
    planet_ease: String,
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

#[derive(Deserialize)]
pub enum Enabled {
    Enabled,
    Disabled
}
impl Into<bool> for Enabled {
    fn into(self) -> bool {
        match self {
            Enabled::Enabled => true,
            Enabled::Disabled => false,
        }
    }
}
impl Default for Enabled {
    fn default() -> Self {
        Enabled::Disabled
    }
}
