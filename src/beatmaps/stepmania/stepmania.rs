use crate::prelude::*;

#[derive(Default, Clone)]
pub struct StepmaniaBeatmap {
    hash: String,
    file_path: String,

    title: String,
    subtitle: String,
    artist: String,

    // these are options to make them easier to unwrap_or()
    title_translated:    Option<String>,
    subtitle_translated: Option<String>,
    artist_translated:   Option<String>,

    genre: String,
    credit: String,

    /// renamed from "music"
    audio_file: String,
    banner: String,
    background: String,
    cd_title: String,

    /// preview time start
    sample_start: f32,
    /// preview time length
    sample_length: f32,

    /// should this map be visible in the menu?
    selectable: bool,
    /// offset of audio file (in seconds)
    audio_offset: f32,

    /// (beat, bpm)
    /// not sure if beat is actual beats, or ms, or s
    pub bpms: Vec<(f32, f32)>,
    /// bpms converted to (ms, beatlength)
    pub beat_lengths: Vec<(f32, f32)>,
    /// (beat, len in seconds)
    /// not sure if beat is actual beats, or ms, or s
    pub breaks: Vec<(f32, f32)>,

    pub chart_info: StepmaniaChart
}

impl StepmaniaBeatmap {
    pub fn load_multiple<P:AsRef<Path>>(path:P) -> TatakuResult<Vec<Self>> {
        let mut map = Self::default();
        map.file_path = path.as_ref().to_string_lossy().to_string();

        let mut maps = Vec::new();
        let parent = path.as_ref().parent().unwrap();


        // ssc support because they changed how per-chart info is added
        let mut chart_type = None;
        let mut description = None;
        let mut difficulty = None;
        let mut meter = None;
        let mut groove_radar_values = None;
        
        let mut lines = read_lines_resolved(&path)?;
        while let Some(line) = lines.next() {
            // trim out comments
            let line = line.split("//").next().unwrap();
            if line.len() == 0 {continue}

            if line.starts_with("#") {
                let mut split = line.trim_end_matches(";").split(":");
                let key = split.next().unwrap();
                let value = split.next().unwrap_or_default();

                match key {
                    "#TITLE" => map.title = value.to_owned(),
                    "#SUBTITLE" => map.subtitle = value.to_owned(),
                    "#ARTIST" => map.artist = value.to_owned(),

                    "#TITLETRANSLIT" if value.len() > 0 => map.title_translated = Some(value.to_owned()),
                    "#SUBTITLETRANSLIT" if value.len() > 0 => map.subtitle_translated = Some(value.to_owned()),
                    "#ARTISTTRANSLIT" if value.len() > 0 => map.artist_translated = Some(value.to_owned()),

                    "#GENRE" => map.genre = value.to_owned(),
                    "#CREDIT" => map.credit = value.to_owned(),
                    "#MUSIC" => map.audio_file = parent.join(value).to_string_lossy().to_string(),
                    "#BANNER" => map.banner = parent.join(value).to_string_lossy().to_string(),
                    "#BACKGROUND" => map.background = parent.join(value).to_string_lossy().to_string(),
                    "#OFFSET" => map.audio_offset = value.parse().unwrap_or_default(),
                    "#BPMS" => {
                        // bpms are a list of beat=bpm separated by commas
                        for entry in value.split(",") {
                            let mut split = entry.split("=");
                            let beat = split.next().unwrap().parse().unwrap();
                            let bpm = split.next().unwrap().parse().unwrap();

                            map.bpms.push((beat, bpm));
                        }
                    }


                    // ssc chart things
                    "#NOTEDATA" => {
                        // something probably
                        // use this to init other info for now
                        chart_type = Some(String::new());
                        description = Some(String::new());
                        difficulty = Some(String::new());
                        meter = Some(String::new());
                        groove_radar_values = Some(String::new());
                    }
                    "#STEPSTYPE" => chart_type = Some(value.to_owned()),
                    "#DIFFICULTY" => difficulty = Some(value.to_owned()),
                    "#METER" => meter = Some(value.to_owned()),
                    "#RADARVALUES" => groove_radar_values = Some(value.to_owned()),

                    "#NOTES" => {
                        // read chart into lines, ensures split is correct
                        let mut chart_info = value.to_owned();
                        while let Some(line) = lines.next() {
                            chart_info += &line;
                            if line.ends_with(";") {break}
                        }

                        // remove final semicolon
                        chart_info = chart_info.trim_end_matches(";").to_owned();

                        // println!("lines: {}", chart_info);
                        let mut chart_split = chart_info.split(":");

                        let mut chart = StepmaniaChart::default();

                        let is_ssc = path.as_ref().extension().unwrap() == "ssc";
                        macro_rules! get {
                            ($name: ident) => {
                                if is_ssc {
                                    std::mem::take(&mut $name).unwrap_or_default()
                                } else {
                                    chart_split.next().unwrap().to_owned()
                                }
                            }
                        }

                        // first entries are meta (if sm, otherwise meta was already loaded)
                        chart.chart_type          = get!(chart_type);
                        chart.description         = get!(description);
                        chart.difficulty          = get!(difficulty);
                        chart.diff_value               = get!(meter).parse().unwrap_or_default();
                        chart.groove_radar_values = get!(groove_radar_values).split(",").map(|r|r.parse().unwrap_or_default()).collect();
                        
                        let note_data = chart_split.next().unwrap();
                        let bars = note_data.split(",");

                        // (time, beat_length)
                        let mut beat_lengths:Vec<(f32,f32)> = map.bpms.iter().map(|(beat, bpm)| (*beat, 60_000.0 / *bpm)).collect();
                        let beat_lens_clone = beat_lengths.clone();
                        for (i, (time, _)) in beat_lengths.iter_mut().enumerate() {
                            // time is actually the beat number
                            // need to convert it to ms
                            *time = beat_lens_clone.get(i).unwrap_or(&(0.0, -map.audio_offset * 1000.0)).1 * *time;
                        }
                        map.beat_lengths = beat_lengths.clone();
                        let mut beat_length_index = 0;

                        let mut current_time = -map.audio_offset * 1000.0;
                        let mut columns:[Vec<(f32, StepmaniaTempNoteType)>; 4] = [
                            Vec::new(),
                            Vec::new(),
                            Vec::new(),
                            Vec::new()
                        ];

                        // turn the bars into columns of known note types at their specified times
                        for bar in bars {
                            let notes:Vec<StepmaniaTempNoteType> = bar.chars().map(|c|StepmaniaTempNoteType::from(c)).collect();

                            let note_snapping = notes.len() as f32 / 16.0;
                            let mut time_step = beat_lengths[beat_length_index].1 / note_snapping;
                            
                            for i in (0..notes.len()).step_by(4) {
                                // push notes into column
                                for n in 0..4 {
                                    columns[n].push((current_time, notes[i+n]));
                                }

                                // check for bpm change
                                if let Some((next_time, next_beat_length)) = beat_lengths.get(beat_length_index + 1) {
                                    if *next_time <= current_time {
                                        beat_length_index += 1;
                                        time_step = *next_beat_length / note_snapping;
                                    }
                                }

                                current_time += time_step;
                            }
                        }

                        // turn the column types into actual note types
                        for (num, col) in columns.iter().enumerate() {
                            let mut last_hold_start = None;

                            for (time, note_type) in col {
                                match note_type {
                                    StepmaniaTempNoteType::None => continue,

                                    StepmaniaTempNoteType::HoldStart
                                    | StepmaniaTempNoteType::RollStart => {
                                        let note_type = match note_type {
                                            StepmaniaTempNoteType::RollStart => StepmaniaNoteType::Roll,
                                            StepmaniaTempNoteType::HoldStart => StepmaniaNoteType::Hold,
                                            _ => panic!("literally impossible")
                                        };
                                        last_hold_start = Some(StepmaniaNote {
                                            column: num as u8,
                                            start: *time,
                                            end: None,
                                            note_type,
                                        });
                                    }

                                    StepmaniaTempNoteType::HoldEnd => {
                                        let note = std::mem::take(&mut last_hold_start);
                                        if let Some(mut note) = note {
                                            note.end = Some(*time);
                                            chart.notes.push(note);
                                        } else {
                                            return Err(BeatmapError::InvalidFile.into())
                                        }
                                    }

                                    StepmaniaTempNoteType::Note
                                    | StepmaniaTempNoteType::Mine
                                    | StepmaniaTempNoteType::KeySound
                                    | StepmaniaTempNoteType::LiftNote
                                    | StepmaniaTempNoteType::FakeNote => {
                                        let note_type = match note_type {
                                            StepmaniaTempNoteType::Note => StepmaniaNoteType::Note,
                                            StepmaniaTempNoteType::Mine => StepmaniaNoteType::Mine,
                                            StepmaniaTempNoteType::KeySound => StepmaniaNoteType::KeySound,
                                            StepmaniaTempNoteType::LiftNote => StepmaniaNoteType::LiftNote,
                                            StepmaniaTempNoteType::FakeNote => StepmaniaNoteType::FakeNote,
                                            _ => panic!("literally impossible")
                                        };
                                        chart.notes.push(StepmaniaNote {
                                            column: num as u8,
                                            start: *time,
                                            end: None,
                                            note_type
                                        });
                                    }
                                }
                            }
                        }

                        let mut map = map.clone();
                        map.chart_info = chart;
                        map.hash = md5(chart_info);

                        maps.push(map);
                    }

                    _ => {}
                }

            } else {
                continue
            }
        }

        Ok(maps)
    }

    pub fn load_single<P:AsRef<Path>>(path:P, meta: &BeatmapMeta) -> TatakuResult<Self> {
        let maps = Self::load_multiple(path)?;

        for map in maps {
            if map.chart_info.difficulty == meta.version {
                return Ok(map)
            }
        }

        Err(BeatmapError::InvalidFile.into())
    }
}

impl TatakuBeatmap for StepmaniaBeatmap {
    fn hash(&self) -> String {self.hash.clone()}
    fn playmode(&self, _incoming:PlayMode) -> PlayMode {"mania".to_owned()}
    fn slider_velocity_at(&self, _time:Frequency) -> Frequency {400.0}

    fn get_timing_points(&self) -> Vec<TimingPoint> {
        self.beat_lengths.iter().map(|&(time, beat_length)| {
            TimingPoint {
                time, 
                beat_length,
                volume: 100,
                meter: 4,
                kiai: false,
                skip_first_barline: false,
                sample_set: 0,
                sample_index: 0,
            }
        }).collect()
    }

    fn get_beatmap_meta(&self) -> BeatmapMeta {
        BeatmapMeta {
            file_path: self.file_path.clone(),
            beatmap_hash: self.hash.clone(),
            beatmap_type: BeatmapType::Stepmania,
            mode: self.playmode(String::new()),
            artist: self.artist.clone(),
            title: self.title.clone(),
            artist_unicode: self.artist_translated.as_ref().unwrap_or(&self.artist).clone(),
            title_unicode: self.title_translated.as_ref().unwrap_or(&self.title).clone(),
            creator: self.chart_info.description.clone(),
            version: self.chart_info.difficulty.clone(),
            audio_filename: self.audio_file.clone(),
            image_filename: self.background.clone(),
            audio_preview: self.sample_start * 1000.0,
            duration: 0.0,
            hp: 1.0,
            od: 1.0,
            cs: 1.0,
            ar: 1.0,
            bpm_min: 0.0,
            bpm_max: 0.0,
            diff: 0.0,
        }
    }


    fn beat_length_at(&self, time:f32, _allow_multiplier:bool) -> f32 {
        for (mut i, &(beat_time, _)) in self.beat_lengths.iter().enumerate() {
            if beat_time >= time {
                if i == 0 {i += 1}
                return self.beat_lengths[i - 1].1
            }
        }
        
        0.0
    }

    fn control_point_at(&self, time:f32) -> TimingPoint {
        let points = self.get_timing_points();
        for (mut i, point) in points.iter().enumerate() {
            if point.time >= time {
                if i == 0 {i += 1}
                return points[i - 1].clone()
            }
        }
        points.last().unwrap().clone()
    }
}


#[derive(Copy, Clone, Debug)]
enum StepmaniaTempNoteType {
    None, // 0
    Note, // 1
    HoldStart, // 2
    /// also roll end
    HoldEnd, // 3
    RollStart, // 4
    Mine, // M
    KeySound, // K
    LiftNote, // L
    FakeNote, // F
}
impl From<char> for StepmaniaTempNoteType {
    fn from(c: char) -> Self {
        match c {
            '0' => Self::None,
            '1' => Self::Note,
            '2' => Self::HoldStart,
            '3' => Self::HoldEnd,
            '4' => Self::RollStart,
            'M' => Self::Mine,
            'K' => Self::KeySound,
            'L' => Self::LiftNote,
            'F' => Self::FakeNote,
            _ => panic!("unknown stepmania note type '{}'", c)
        }
    }
}


#[derive(Default, Clone)]
pub struct StepmaniaChart {
    pub chart_type: String, // not sure this matters at all
    /// usually difficulty name
    pub description: String,
    pub difficulty: String,
    pub diff_value: u32, 
    pub groove_radar_values: Vec<u32>,
    pub notes: Vec<StepmaniaNote>,
}

#[derive(Copy, Clone)]
pub struct StepmaniaNote {
    pub column: u8,
    pub start: f32,
    pub end: Option<f32>,
    pub note_type: StepmaniaNoteType,
}

#[derive(Copy, Clone)]
pub enum StepmaniaNoteType {
    Note,
    Hold,
    Roll,
    Mine,
    KeySound,
    LiftNote,
    FakeNote
}
