use crate::prelude::*;

#[derive(Clone, Default)]
pub struct OsuBeatmap {
    pub hash: String,
    
    // meta info
    pub metadata: BeatmapMeta,
    pub timing_points: Vec<OsuTimingPoint>,

    pub notes: Vec<NoteDef>,
    pub sliders: Vec<SliderDef>,
    pub spinners: Vec<SpinnerDef>,
    pub holds: Vec<HoldDef>,

    pub combo_colors: Vec<Color>,

    // other metadata 
    pub beatmap_version: u8,
    pub slider_multiplier: f32,
    pub slider_tick_rate: f32,
    pub stack_leniency: f32,
}
impl OsuBeatmap {
    pub fn load(file_path:String) -> TatakuResult<OsuBeatmap> {
        let parent_dir = Path::new(&file_path).parent().unwrap();
        let hash = crate::get_file_hash(&file_path).unwrap();

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

        let mut current_area = BeatmapSection::Version;
        let mut beatmap = Self {
            metadata: BeatmapMeta::new(file_path.clone(), hash.clone(), BeatmapType::Osu),
            hash,
            notes: Vec::new(),
            sliders: Vec::new(),
            spinners: Vec::new(),
            holds: Vec::new(),
            timing_points: Vec::new(),
            combo_colors: Vec::new(),

            beatmap_version: 0,
            slider_multiplier: 1.4,
            slider_tick_rate: 1.0,
            stack_leniency: 1.0,
        };

        for line in read_lines_resolved(&file_path)? {
            // ignore empty lines
            if line.len() < 2 {continue}

            // check for section change
            if line.starts_with("[") {
                // this one isnt really necessary
                if line == "[General]" {current_area = BeatmapSection::General}
                if line == "[Editor]" {current_area = BeatmapSection::Editor}
                if line == "[Metadata]" {current_area = BeatmapSection::Metadata}
                if line == "[Difficulty]" {current_area = BeatmapSection::Difficulty}
                if line == "[Events]" {current_area = BeatmapSection::Events}
                if line == "[Colours]" {current_area = BeatmapSection::Colors}
                if line == "[TimingPoints]" {current_area = BeatmapSection::TimingPoints}
                if line == "[HitObjects]" {
                    // sort timing points before moving onto hitobjects
                    beatmap.timing_points.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

                    current_area = BeatmapSection::HitObjects; 
                }
                continue;
            }

            // not a change in area, check line
            match current_area {
                BeatmapSection::Version => {
                    match line.split("v").last().unwrap().trim().parse::<u8>() {
                        Ok(v) => beatmap.beatmap_version = v,
                        Err(e) => warn!("error parsing beatmap version: {}", e),
                    }
                }
                BeatmapSection::General => {
                    let mut split = line.split(":");
                    let key = split.next().unwrap().trim();
                    let val = split.next().unwrap().trim();

                    if key == "AudioFilename" {beatmap.metadata.audio_filename = parent_dir.join(val).to_str().unwrap().to_owned()}
                    if key == "PreviewTime" {beatmap.metadata.audio_preview = val.parse().unwrap_or(0.0)}
                    if key == "StackLeniency" {beatmap.stack_leniency = val.parse().unwrap_or(0.0)}
                    if key == "Mode" {
                        let m = val.parse::<u8>().unwrap();
                        beatmap.metadata.mode = playmode_from_u8(m);
                    }
                }
                BeatmapSection::Metadata => {
                    let mut split = line.split(":");
                    let key = split.next().unwrap().trim();
                    let val = split.next().unwrap_or("").trim();
                    
                    if key == "Title" {beatmap.metadata.title = val.to_owned()}
                    if key == "TitleUnicode" {beatmap.metadata.title_unicode = val.to_owned()}
                    if key == "Artist" {beatmap.metadata.artist = val.to_owned()}
                    if key == "ArtistUnicode" {beatmap.metadata.artist_unicode = val.to_owned()}
                    if key == "Creator" {beatmap.metadata.creator = val.to_owned()}
                    if key == "Version" {beatmap.metadata.version = val.to_owned()}
                }
                BeatmapSection::Difficulty => {
                    let mut split = line.split(":");
                    let key = split.next().unwrap().trim();
                    let val = split.next().unwrap().trim().parse::<f32>().unwrap();

                    if key == "HPDrainRate" {beatmap.metadata.hp = val}
                    if key == "CircleSize" {beatmap.metadata.cs = val}
                    if key == "OverallDifficulty" {beatmap.metadata.od = val}
                    if key == "ApproachRate" {beatmap.metadata.ar = val}
                    if key == "SliderMultiplier" {beatmap.slider_multiplier = val}
                    if key == "SliderTickRate" {beatmap.slider_tick_rate = val}
                }
                BeatmapSection::Events => {
                    let mut split = line.split(',');
                    // eventType,startTime,eventParams
                    // 0,0,filename,xOffset,yOffset
                    let event_type = split.next().unwrap();

                    if event_type == "0" && split.next().unwrap() == "0" {
                        let filename = split.next().unwrap().to_owned();
                        let filename = filename.trim_matches('"');
                        beatmap.metadata.image_filename = parent_dir.join(filename).to_str().unwrap().to_owned();
                    
                    }
                }
                BeatmapSection::TimingPoints => {
                    beatmap.timing_points.push(OsuTimingPoint::from_str(&line));
                }
                BeatmapSection::HitObjects => {
                    let mut split = line.split(",");
                    if split.clone().count() < 2 {continue} // skip empty lines

                    let x = split.next().unwrap().parse::<f64>().unwrap();
                    let y = split.next().unwrap().parse::<f64>().unwrap();
                    let time = split.next().unwrap().parse::<f32>().unwrap();
                    let read_type = split.next().unwrap().parse::<u64>().unwrap_or(0); // see below

                    let hitsound_raw = split.next().unwrap();
                    let hitsound = hitsound_raw.parse::<i8>();
                    if let Err(e) = &hitsound {
                        warn!("error parsing hitsound: {} (line: {})", e, line)
                    }
                    
                    let hitsound = hitsound.unwrap_or(0).abs() as u8; // 0 = normal, 2 = whistle, 4 = finish, 8 = clap

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
                            .unwrap_or("0")
                            .split("|")
                            .map(|s|s.parse::<u8>().unwrap_or(0)).collect();
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
                    // Combo[n] : r,g,b
                    // SliderTrackOverride : r,g,b
                    // SliderBorder : r,g,b
                    let mut split = line.split(":");
                    let key = split.next().unwrap().trim();
                    let mut val_split = split.next().unwrap().trim().split(",");
                    let r:u8 = val_split.next().unwrap_or_default().parse().unwrap_or_default();
                    let g:u8 = val_split.next().unwrap_or_default().parse().unwrap_or_default();
                    let b:u8 = val_split.next().unwrap_or_default().parse().unwrap_or_default();
                    let c = |a| {a as f32 / 255.0};
                    let color = Color::new(c(r), c(g), c(b), 1.0);
                    
                    if key.starts_with("Combo") {
                        beatmap.combo_colors.push(color);
                    }
                },
                BeatmapSection::Editor => {},
            }
        }

        // metadata bpm
        let mut bpm_min = 9999999999.9;
        let mut bpm_max = 0.0;
        for i in beatmap.timing_points.iter() {
            if i.is_inherited() {continue}

            if i.beat_length < bpm_min {
                bpm_min = i.beat_length;
            }
            if i.beat_length > bpm_max {
                bpm_max = i.beat_length;
            }
        }
        beatmap.metadata.bpm_min = 60_000.0 / bpm_min;
        beatmap.metadata.bpm_max = 60_000.0 / bpm_max;

        // metadata duration (scuffed bc .osu is trash)
        let mut start_time = 0.0;
        let mut end_time = 0.0;
        for note in beatmap.notes.iter() {
            if note.time < start_time {
                start_time = note.time
            }
            if note.time > end_time {
                end_time = note.time
            }
        }
        for note in beatmap.sliders.iter() {
            if note.time < start_time {
                start_time = note.time
            }
            if note.time > end_time {
                end_time = note.time
            }
        }
        for note in beatmap.spinners.iter() {
            if note.time < start_time {
                start_time = note.time
            }
            if note.time > end_time {
                end_time = note.time
            }
        }
        beatmap.metadata.duration = end_time - start_time;


        // make sure we have the ar set
        beatmap.metadata.do_checks();

        Ok(beatmap)
    }

    pub fn from_metadata(metadata: &BeatmapMeta) -> OsuBeatmap {
        // load the betmap
        let mut b = Self::load(metadata.file_path.clone()).unwrap();
        // overwrite the loaded meta with the old meta, this maintains calculations etc
        b.metadata = metadata.clone();
        b
    }

    pub fn bpm_multiplier_at(&self, time:f32) -> f32 {
        self.control_point_at(time).bpm_multiplier()
    }
}
impl TatakuBeatmap for OsuBeatmap {
    fn hash(&self) -> String {self.hash.clone()}
    fn get_timing_points(&self) -> Vec<crate::beatmaps::common::TimingPoint> {
        self.timing_points
            .iter()
            .map(|t|t.clone().into())
            .collect()
    }
    fn get_beatmap_meta(&self) -> BeatmapMeta {self.metadata.clone()}

    fn playmode(&self, incoming:PlayMode) -> PlayMode {
        match &*self.metadata.mode {
            "osu" => incoming,
            "adofai" => panic!("osu map has adofai mode !?"),
            m => m.to_owned()
        }
    }


    fn beat_length_at(&self, time:f32, allow_multiplier:bool) -> f32 {
        if self.timing_points.len() == 0 {return 0.0}

        let mut point: Option<OsuTimingPoint> = Some(self.timing_points.as_slice()[0].clone());
        let mut inherited_point: Option<OsuTimingPoint> = None;

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
    fn slider_velocity_at(&self, time:f32) -> f32 {
        let bl = self.beat_length_at(time, true);
        100.0 * (self.slider_multiplier * 1.4) * if bl > 0.0 {1000.0 / bl} else {1.0}
    }
    fn control_point_at(&self, time:f32) -> TimingPoint {
        // panic as this should be dealt with earlier in the code
        if self.timing_points.len() == 0 {panic!("beatmap has no timing points!")}

        let mut point = self.timing_points[0];
        for tp in self.timing_points.iter() {
            if tp.time <= time {point = *tp}
        }

        point.into()
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
impl Into<crate::beatmaps::common::TimingPoint> for OsuTimingPoint {
    fn into(self) -> crate::beatmaps::common::TimingPoint {
        crate::beatmaps::common::TimingPoint {
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