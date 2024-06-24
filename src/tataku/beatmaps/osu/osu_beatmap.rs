use std::str::FromStr;
use crate::prelude::*;

#[derive(Clone, Default)]
pub struct OsuBeatmap {
    // meta info
    pub metadata: Arc<BeatmapMeta>,
    pub hash: Md5Hash,
    pub beatmap_version: u8,
    pub slider_multiplier: f32,
    pub slider_tick_rate: f32,
    pub stack_leniency: f32,

    // notes
    pub notes: Vec<NoteDef>,
    pub sliders: Vec<SliderDef>,
    pub spinners: Vec<SpinnerDef>,
    pub holds: Vec<HoldDef>,

    // events
    pub timing_points: Vec<OsuTimingPoint>,
    pub combo_colors: Vec<Color>,
    pub events: Vec<OsuEvent>,

    pub storyboard: Option<StoryboardDef>
}
impl OsuBeatmap { 
    pub fn load(file_path:String) -> TatakuResult<OsuBeatmap> {
        Self::base_loader(file_path, false)
    }

    pub fn load_metadata(filepath: impl AsRef<Path>) -> TatakuResult<Arc<BeatmapMeta>> {
        Ok(Self::base_loader(filepath, true)?.metadata)
    }

    /// loader for both metadata only and full map. removes duplicate code
    fn base_loader(filepath: impl AsRef<Path>, metadata_only: bool) -> TatakuResult<OsuBeatmap> {
        let file_path = filepath.as_ref();
        let parent_dir = file_path.parent().unwrap();
        let hash = Io::get_file_hash(file_path).unwrap();

        let mut start_time = 0.0;
        let mut end_time = 0.0;

        /// helper enum
        #[derive(Debug)]
        enum BeatmapSection {
            Version,
            General,
            Editor,
            Metadata,
            Difficulty,
            Events,
            TimingPoints,
            Colors,
            HitObjects,
        }

        let file_path = file_path.as_os_str().to_string_lossy().to_string();
        let mut current_area = BeatmapSection::Version;
        let mut metadata = BeatmapMeta::new(file_path.clone(), hash.clone(), BeatmapType::Osu);

        let mut storyboard_lines = Vec::new();

        let mut beatmap = Self {
            metadata: Arc::new(metadata.clone()),
            hash,
            notes: Vec::new(),
            sliders: Vec::new(),
            spinners: Vec::new(),
            holds: Vec::new(),
            timing_points: Vec::new(),
            combo_colors: Vec::new(),
            events: Vec::new(),
            storyboard: None,

            beatmap_version: 0,
            slider_multiplier: 1.4,
            slider_tick_rate: 1.0,
            stack_leniency: 1.0,
        };

        for line in Io::read_lines_resolved(&file_path)? {
            // ignore empty lines
            if line.len() < 2 { continue }

            // check for section change
            if line.starts_with("[") {
                match &*line {
                    // this one isnt really necessary
                    "[General]" => current_area = BeatmapSection::General,
                    "[Editor]" => current_area = BeatmapSection::Editor,
                    "[Metadata]" => current_area = BeatmapSection::Metadata,
                    "[Difficulty]" => current_area = BeatmapSection::Difficulty,
                    "[Events]" => current_area = BeatmapSection::Events,
                    "[Colours]" => current_area = BeatmapSection::Colors,
                    "[TimingPoints]" => current_area = BeatmapSection::TimingPoints,
                    "[HitObjects]" => {
                        // sort timing points before moving onto hitobjects
                        beatmap.timing_points.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
                        current_area = BeatmapSection::HitObjects; 
                    }
                    _ => {}
                }

                continue;
            }

            // not a change in area, check line
            match current_area {
                BeatmapSection::Version => {
                    match line.split("v").last().unwrap().trim().parse::<u8>() {
                        Ok(v) => beatmap.beatmap_version = v,
                        Err(e) => warn!("[{file_path}] error parsing beatmap version: {e} (map will still load)"),
                    }
                }
                BeatmapSection::General => {
                    let mut split = line.split(":");
                    let key = split.next().unwrap().trim();
                    let val = split.next().unwrap().trim();

                    match &*key {
                        "AudioFilename" => metadata.audio_filename = parent_dir.join(val).to_str().unwrap().to_owned(),
                        "PreviewTime" => metadata.audio_preview = val.parse().unwrap_or(0.0),
                        "StackLeniency" => beatmap.stack_leniency = val.parse().unwrap_or(0.0),
                        "Mode" => metadata.mode = playmode_from_u8(val.parse::<u8>().unwrap()).to_owned(),

                        _ => {}
                    }
                }
                BeatmapSection::Metadata => {
                    let mut split = line.split(":");
                    let key = split.next().unwrap().trim();
                    let val = split.collect::<Vec<&str>>().join(":");
                    
                    match &*key {
                        "Title" => metadata.title = val.to_owned(), 
                        "TitleUnicode" => metadata.title_unicode = val.to_owned(), 
                        "Artist" => metadata.artist = val.to_owned(), 
                        "ArtistUnicode" => metadata.artist_unicode = val.to_owned(), 
                        "Creator" => metadata.creator = val.to_owned(), 
                        "Version" => metadata.version = val.to_owned(), 
                        _ => {}
                    }
                }
                BeatmapSection::Difficulty => {
                    let mut split = line.split(":");
                    let key = split.next().unwrap().trim();
                    let val = split.next().unwrap().trim().parse::<f32>().unwrap();

                    match &*key {
                        "HPDrainRate" => metadata.hp = val, 
                        "CircleSize" => metadata.cs = val, 
                        "OverallDifficulty" => metadata.od = val, 
                        "ApproachRate" => metadata.ar = val, 
                        "SliderMultiplier" => beatmap.slider_multiplier = val, 
                        "SliderTickRate" => beatmap.slider_tick_rate = val, 
                        _ => {}
                    }
                }
                BeatmapSection::Events => {
                    // let mut split = line.split(',');
                    // // eventType,startTime,eventParams
                    // // 0,0,filename,xOffset,yOffset
                    // let event_type = split.next().unwrap();

                    // if event_type == "0" && split.next().unwrap() == "0" {
                    //     let filename = split.next().unwrap().to_owned();
                    //     let filename = filename.trim_matches('"');
                    //     metadata.image_filename = parent_dir.join(filename).to_str().unwrap().to_owned();
                    // }

                    if line.starts_with("//") { continue }
                    
                    match OsuEvent::from_str(&line) {
                        Ok(event) => {
                            if let OsuEvent::Background { filename, start_time: 0, .. } = &event {
                                // background
                                let filename = filename.trim_matches('"');
                                metadata.image_filename = parent_dir.join(filename).to_str().unwrap().to_owned();
                            }

                            if !metadata_only {
                                beatmap.events.push(event);
                            }
                        } 
                        Err(_e) => {
                            if !metadata_only {
                                storyboard_lines.push(line);
                            }

                            // if !metadata_only {
                            //     error!("error parsing event: {e}")
                            // }
                        }
                    }
                }
                BeatmapSection::TimingPoints => {
                    beatmap.timing_points.push(OsuTimingPoint::from_str(&line));
                }
                BeatmapSection::HitObjects => {
                    let mut split = line.split(",");
                    if split.clone().count() < 2 { continue } // skip empty lines

                    let x = split.next().unwrap().parse::<f32>().unwrap();
                    let y = split.next().unwrap().parse::<f32>().unwrap();
                    let time = split.next().unwrap().parse::<f32>().unwrap();

                    if time < start_time {
                        start_time = time
                    }
                    if time > end_time {
                        end_time = time
                    }

                    if metadata_only { continue; }

                    let read_type = split.next().unwrap().parse::<u64>().unwrap_or(0); // see below

                    let hitsound_raw = split.next().unwrap();
                    let hitsound = hitsound_raw.parse::<i8>();
                    if let Err(e) = &hitsound {
                        warn!("error parsing hitsound: {} (line: {})", e, line)
                    }
                    
                    let hitsound = hitsound.unwrap_or(0).abs() as u8; // 0 = normal, 2 = whistle, 4 = finish, 8 = clap
                    let hitsound_str = &*hitsound.to_string();

                    // read type:
                    // abcdefgh
                    // a = note
                    // b = slider
                    // c = new combo
                    // d, e, f = combo color skip count
                    // g = spinner
                    // h = mania hold
                    let new_combo = (read_type & 4) > 0;
                    let color_skip = 
                          if (read_type & 16) > 0 {1} else {0} 
                        + if (read_type & 32) > 0 {2} else {0} 
                        + if (read_type & 64) > 0 {4} else {0};

                    if (read_type & 2) > 0 { // slider
                        let curve_raw = split.next().unwrap();
                        let mut curve = curve_raw.split('|');
                        let slides = split.next().unwrap().parse::<u64>().unwrap();
                        let length = split.next().unwrap().parse::<f32>().unwrap();
                        let edge_sounds = split
                            .next()
                            .unwrap_or(hitsound_str)
                            .split("|")
                            .map(|s|s.parse::<u8>().unwrap_or(hitsound)).collect();
                        let edge_sets = split
                            .next()
                            .unwrap_or("0:0")
                            .split("|")
                            .map(|s| {
                                let mut s2 = s.split(':');
                                [
                                    s2.next().unwrap_or("0").parse::<u8>().unwrap_or(0),
                                    s2.next().unwrap_or("0").parse::<u8>().unwrap_or(0),
                                ]
                            })
                            .collect();


                        let curve_type = match &*curve.next().unwrap() {
                            "B" => CurveType::Bézier,
                            "P" => CurveType::Perfect,
                            "C" => CurveType::Catmull,
                            "L" => CurveType::Linear,
                            _ => CurveType::Linear
                        };

                        let mut curve_points = Vec::new();
                        while let Some(pair) = curve.next() {
                            let mut s = pair.split(':');
                            curve_points.push(Vector2::new(
                                s.next().unwrap().parse().unwrap(),
                                s.next().unwrap().parse().unwrap()
                            ))
                        }

                        beatmap.sliders.push(SliderDef {
                            raw: line.clone(),
                            pos: Vector2::new(x, y),
                            time,
                            curve_type,
                            curve_points,
                            slides,
                            length,
                            hitsound,
                            hitsamples: HitSamples::from_str(split.next()),
                            edge_sounds,
                            edge_sets,
                            new_combo,
                            color_skip
                        });

                    } else if (read_type & 8) > 0 { // spinner
                        // x,y,time,type,hitSound,...
                        // endTime,hitSample
                        let end_time = split.next().unwrap().parse::<f32>().unwrap();

                        beatmap.spinners.push(SpinnerDef {
                            pos: Vector2::new(x, y),
                            time,
                            end_time,
                            hitsound,
                            hitsamples: HitSamples::from_str(split.next()),
                            new_combo,
                            color_skip
                        });
                        // let diff_map = map_difficulty_range(beatmap.metadata.od as f64, 3.0, 5.0, 7.5);
                        // let hits_required:u16 = ((length / 1000.0 * diff_map) * 1.65).max(1.0) as u16; // ((this.Length / 1000.0 * this.MapDifficultyRange(od, 3.0, 5.0, 7.5)) * 1.65).max(1.0)
                        // let spinner = Spinner::new(time, end_time, sv, hits_required);
                        // beatmap.notes.lock().push(Box::new(spinner));
                    } else if (read_type & 2u64.pow(7)) > 0 { // mania hold
                        let end_time = split.next().unwrap().split(":").next().unwrap().parse::<f32>().unwrap();
                        beatmap.holds.push(HoldDef {
                            pos: Vector2::new(x, y),
                            time,
                            end_time,
                            hitsound,
                            hitsamples: HitSamples::from_str(split.next()),
                        });
                    } else { // note
                        beatmap.notes.push(NoteDef {
                            pos: Vector2::new(x, y),
                            time,
                            hitsound,
                            hitsamples: HitSamples::from_str(split.next()),
                            new_combo,
                            color_skip
                        });
                    }
                }

                BeatmapSection::Colors => {
                    if metadata_only { continue; }

                    // Combo[n] : r,g,b
                    // SliderTrackOverride : r,g,b
                    // SliderBorder : r,g,b
                    let mut split = line.split(":");
                    let key = split.next().unwrap().trim();

                    if key.starts_with("Combo") {
                        let mut val_split = split.next().unwrap().trim().split(",");
                        let r:u8 = val_split.next().unwrap_or_default().parse().unwrap_or_default();
                        let g:u8 = val_split.next().unwrap_or_default().parse().unwrap_or_default();
                        let b:u8 = val_split.next().unwrap_or_default().parse().unwrap_or_default();
                        let c = |a| {a as f32 / 255.0};
                        let color = Color::new(c(r), c(g), c(b), 1.0);

                        beatmap.combo_colors.push(color);
                    }
                }
                BeatmapSection::Editor => {},
            }
        }

        // storyboard
        #[cfg(feature="storyboards")]
        if !metadata_only {
            // see if theres a .osb in the parent folder.
            // idk if this is how its supposed to be done but theres no documentation on it in the wiki
            let osb_file = std::fs::read_dir(parent_dir).ok().and_then(|files|files.filter_map(|f|f.ok()).find(|f|f.file_name().to_string_lossy().ends_with(".osb")));
            if let Some(storyboard_file) = osb_file {
                storyboard_lines.extend(Io::read_lines_resolved(storyboard_file.path()).unwrap())
            }
            
            match StoryboardDef::read(storyboard_lines) {
                Ok(s) => beatmap.storyboard = Some(s),
                Err(e) => error!("error reading storyboard file: {e}")
            }
        }


        // verify we have a valid beatmap
        if !metadata_only && beatmap.notes.is_empty() && beatmap.sliders.is_empty() && beatmap.spinners.is_empty() {
            // no notes
            return Err(BeatmapError::NoNotes)?;
        }
        if beatmap.timing_points.is_empty() {
            // no timing points
            return Err(BeatmapError::NoTimingPoints)?;
        }



        // metadata bpm
        let mut bpm_min = 9999999999.9;
        let mut bpm_max = 0.0;
        for i in beatmap.timing_points.iter() {
            if i.is_inherited() { continue }

            if i.beat_length < bpm_min {
                bpm_min = i.beat_length;
            }
            if i.beat_length > bpm_max {
                bpm_max = i.beat_length;
            }
        }
        metadata.bpm_min = 60_000.0 / bpm_min;
        metadata.bpm_max = 60_000.0 / bpm_max;

        // metadata duration (scuffed bc .osu is trash)
        metadata.duration = end_time - start_time;

        // make sure we have the ar set
        metadata.do_checks();

        beatmap.metadata = Arc::new(metadata);

        Ok(beatmap)
    }

    pub fn from_metadata(metadata: &Arc<BeatmapMeta>) -> OsuBeatmap {
        // load the betmap
        let mut b = Self::load(metadata.file_path.clone()).unwrap();
        // overwrite the loaded meta with the old meta, this maintains calculations etc
        b.metadata = metadata.clone();
        b
    }

    // pub fn bpm_multiplier_at(&self, time:f32) -> f32 {
    //     self.control_point_at(time).bpm_multiplier()
    // }

}
#[async_trait]
impl TatakuBeatmap for OsuBeatmap {
    fn hash(&self) -> Md5Hash { self.hash }
    fn get_beatmap_meta(&self) -> Arc<BeatmapMeta> { self.metadata.clone() }

    fn get_timing_points(&self) -> Vec<TimingPoint> {
        self.timing_points
            .iter()
            .map(|t|t.clone().into())
            .collect()
    }

    fn playmode(&self, incoming:String) -> String {
        match &*self.metadata.mode {
            "osu" => incoming,
            "adofai" => panic!("osu map has adofai mode !?"),
            m => m.to_owned()
        }
    }

    fn slider_velocity(&self) -> f32 {
        self.slider_multiplier
    }


    
    fn get_events(&self) -> Vec<InGameEvent> {
        self.events.iter().filter_map(|i| match i {
            OsuEvent::Break { start_time, end_time } => Some(InGameEvent::Break { start: *start_time as f32, end: *end_time as f32 }),
            _ => None
        }).collect()
    }
    async fn get_animation(&self, skin_manager: &mut SkinManager) -> Option<Box<dyn BeatmapAnimation>> {
        if let Some(storyboard) = &self.storyboard {
            let parent_dir = Path::new(&self.metadata.file_path).parent()?.to_string_lossy().to_string();
            
            match OsuStoryboard::new(storyboard.clone(), parent_dir, skin_manager).await {
                Ok(sb) => Some(Box::new(sb)),
                Err(_e) => None
            }
        } else {
            None
        }
    }
}


///https://osu.ppy.sh/wiki/en/osu%21_File_Formats/Osu_%28file_format%29#timing-points
#[derive(Clone, Copy)]
pub struct OsuTimingPoint {
    /// Start time of the timing section, in milliseconds from the beginning of the beatmap's audio. The end of the timing section is the next timing point's time (or never, if this is the last timing point).
    pub time: f32,
    /// This property has two meanings:
    ///     For uninherited timing points, the duration of a beat, in milliseconds.
    ///     For inherited timing points, a negative inverse slider velocity multiplier, as a percentage. For example, -50 would make all sliders in this timing section twice as fast as SliderMultiplier.
    pub beat_length: f32,
    /// Volume percentage for hit objects
    pub volume: u8,
    /// Amount of beats in a measure. Inherited timing points ignore this property.
    pub meter: u8,

    // effects

    /// Whether or not kiai time is enabled
    pub kiai: bool,
    /// Whether or not the first barline is omitted in osu!taiko and osu!mania
    pub skip_first_barline: bool,

    // samples

    /// Default sample set for hit objects (0 = beatmap default, 1 = normal, 2 = soft, 3 = drum)
    pub sample_set: u8,
    /// Custom sample index for hit objects. 0 indicates osu!'s default hitsounds
    pub sample_index: u8
}
impl OsuTimingPoint {
    pub fn from_str(str:&str) -> Self {
        // time,beatLength,meter,sampleSet,sampleIndex,volume,uninherited,effects
        // debug!("{}", str.clone());
        let mut split = str.split(',');
        let time = split.next().unwrap_or("0").parse::<f32>().unwrap_or(0.0);
        let beat_length = split.next().unwrap_or("0").parse::<f32>().unwrap_or(0.0);
        let meter = split.next().unwrap_or("4").parse::<u8>().unwrap_or(4);
        let sample_set = split.next().unwrap_or("0").parse::<u8>().unwrap_or(0);
        let sample_index = split.next().unwrap_or("0").parse::<u8>().unwrap_or(0);

        let volume = match split.next() {
            Some(str) => str.parse::<u8>().unwrap_or(50),
            None => 50
        };
        let _uninherited = split.next();
        let effects = match split.next() {
            Some(str) => str.parse::<u8>().unwrap_or(0),
            None => 0
        };

        let kiai = (effects & 1) == 1;
        let skip_first_barline = (effects & 8) == 1;

        Self {
            time, 
            beat_length, 
            volume, 
            meter,

            sample_set,
            sample_index,

            kiai,
            skip_first_barline
        }
    }

    pub fn is_inherited(&self) -> bool {
        return self.beat_length < 0.0;
    }
    
    pub fn bpm_multiplier(&self) -> f32 {
        if !self.is_inherited() {1.0}
        else {self.beat_length.abs().clamp(10.0, 1000.0) / 100.0}
    }
}
impl Into<TimingPoint> for OsuTimingPoint {
    fn into(self) -> TimingPoint {
        TimingPoint {
            time: self.time,
            beat_length: self.beat_length,
            volume: self.volume,
            meter: self.meter,
            kiai: self.kiai,
            skip_first_barline: self.skip_first_barline,
            sample_set: self.sample_set,
            sample_index: self.sample_index,
        }
    }
}


#[derive(Clone, Debug)]
pub enum OsuEvent {
    Background {
        start_time: i32,
        filename: String,
        x_offset: i32,
        y_offset: i32
    },

    Video {
        start_time: i32,
        filename: String,
        x_offset: i32,
        y_offset: i32,
    },

    Break {
        start_time: i32,
        end_time: i32, 
    }
}
impl FromStr for OsuEvent {
    type Err = TatakuError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(",");
        match split.next() {
            Some("0") | Some("Background") => {
                let start_time = split.next().ok_or_else(||TatakuError::String("missing time".to_owned()))?.parse::<i32>().map_err(|_|TatakuError::String("bad time value".to_owned()))?;
                let filename = split.next().ok_or_else(||TatakuError::String("missing filename".to_owned()))?.to_owned();

                let x_offset = split.next().unwrap_or("0").parse::<i32>().map_err(|_|TatakuError::String("bad x_offset".to_owned()))?;
                let y_offset = split.next().unwrap_or("0").parse::<i32>().map_err(|_|TatakuError::String("bad y_offset".to_owned()))?;
                Ok(OsuEvent::Background { start_time, filename, x_offset, y_offset })
            }

            Some("1") | Some("Video") => {
                let start_time = split.next().ok_or_else(||TatakuError::String("missing time".to_owned()))?.parse::<i32>().map_err(|_|TatakuError::String("bad time value".to_owned()))?;
                let filename = split.next().ok_or_else(||TatakuError::String("missing filename".to_owned()))?.to_owned();

                let x_offset = split.next().unwrap_or("0").parse::<i32>().map_err(|_|TatakuError::String("bad x_offset".to_owned()))?;
                let y_offset = split.next().unwrap_or("0").parse::<i32>().map_err(|_|TatakuError::String("bad y_offset".to_owned()))?;
                Ok(OsuEvent::Video { start_time, filename, x_offset, y_offset })
            }

            Some("2") | Some("Break") => {
                let start_time = split.next().ok_or_else(||TatakuError::String("missing time".to_owned()))?.parse::<i32>().map_err(|_|TatakuError::String("bad start time value".to_owned()))?;
                let end_time = split.next().ok_or_else(||TatakuError::String("missing time".to_owned()))?.parse::<i32>().map_err(|_|TatakuError::String("bad end time value".to_owned()))?;
                Ok(OsuEvent::Break { start_time, end_time })
            }

            Some(other) => {
                Err(TatakuError::String(format!("unknown event '{other}'")))
            }

            None => Err(TatakuError::String("bad event".to_owned()))
        }
    }
}


fn playmode_from_u8(p:u8) -> &'static str {
    match p {
        0 => "osu",
        1 => "taiko",
        2 => "catch",
        3 => "mania",
        _ => "osu",
    }
}