use crate::prelude::*;

/// this is technically a single course
#[derive(Default, Debug)]
pub struct TjaBeatmap {
    pub hash: Md5Hash,
    pub filename: String,
    pub directory: String,

    pub title: String,
    pub title_unicode: String,

    /// generally artist
    pub subtitle: String,
    pub subtitle_unicode: String,

    pub bpm: f32,
    pub offset: f32,
    pub audio_path: String,
    pub image_path: String,

    // pub offset: f32,
    pub preview_time: f32,


    pub course_name: String,
    pub course_level: u8,
    pub course_creator: String,

    pub course_events: Vec<TjaCourseEvent>,

    pub branches: Vec<TjaBranchGroup>,
    pub circles: Vec<TjaCircle>,
    pub drumrolls: Vec<TjaDrumroll>,
    pub balloons: Vec<TjaBalloon>,
}

impl TjaBeatmap {
    pub fn load_multiple(path: impl AsRef<Path>) -> TatakuResult<Vec<Self>> {
        let path = path.as_ref();


        let mut data = std::fs::read(path)?;
        // if theres random useless bom data, remove it
        if &data[0..3] == &[0xEF, 0xBB, 0xBF] {
            data = data[3..].to_vec();
        }

        let lines = String::from_utf8(data).map_err(|_|BeatmapError::InvalidFile)?;
        let lines = lines.lines();

        let filename = path.to_string_lossy().to_string();
        let parent = path.parent().unwrap().to_string_lossy().to_string();

        let mut maps = super::tja_parser::TjaParser::default().parse(lines)?;
        maps.iter_mut().for_each(|map| {
            map.directory = parent.clone();
            map.filename = filename.clone();
        });

        Ok(maps)
    }

    pub fn load_single(path: impl AsRef<Path>, meta: &BeatmapMeta) -> TatakuResult<Self> {
        let maps = Self::load_multiple(path)?;

        for map in maps {
            if map.hash == meta.beatmap_hash {
                return Ok(map)
            }
        }

        Err(BeatmapError::NotFoundInSet.into())
    }

}

impl TatakuBeatmap for TjaBeatmap {
    fn hash(&self) -> Md5Hash { self.hash }
    fn playmode(&self, _incoming:String) -> String { "taiko".to_owned() }

    fn get_timing_points(&self) -> Vec<TimingPoint> {
        let mut timing_points = Vec::new();

        let mut timing_point = TimingPoint {
            time: self.offset,
            beat_length: 60_000.0 / self.bpm,
            ..Default::default()
        };
        timing_points.push(timing_point);
        let mut last_scroll = -100.0;

        for event in self.course_events.iter() {
            match event.event {
                TjaCourseEventType::Bpm(bpm) => {
                    last_scroll = -100.0;
                    timing_point.beat_length = 60_000.0 / bpm;
                    timing_points.push(timing_point);
                }
                TjaCourseEventType::Kiai(kiai) => {
                    timing_point.beat_length = last_scroll;
                    timing_point.kiai = kiai;
                    timing_points.push(timing_point);
                }
                TjaCourseEventType::Scroll(scroll) => {
                    last_scroll = -scroll * 100.0;
                    timing_point.beat_length = last_scroll;
                    timing_points.push(timing_point);
                }

                _ => {}
            }
        }

        timing_points
    }
    fn get_beatmap_meta(&self) -> Arc<BeatmapMeta> {
        let start = [
            self.circles.first().map(|b|b.time).unwrap_or(999999.9) as i32,
            self.drumrolls.first().map(|b|b.time).unwrap_or(999999.9) as i32,
            self.balloons.first().map(|b|b.time).unwrap_or(999999.9) as i32,
        ].iter().min().map(|i|*i).unwrap_or(0) as f32;

        let end = [
            self.circles.last().map(|b|b.time).unwrap_or_default() as i32,
            self.drumrolls.last().map(|b|b.end_time).unwrap_or_default() as i32,
            self.balloons.last().map(|b|b.end_time).unwrap_or_default() as i32,
        ].iter().max().map(|i|*i).unwrap_or(0) as f32;
        let duration = end - start;

        let timing_points = self.get_timing_points();
        let mut bl_min:f32 = 99999.0;
        let mut bl_max:f32 = 0.0;
        for tp in timing_points.iter().filter(|i|i.beat_length > 0.0) {
            bl_min = bl_min.min(tp.beat_length);
            bl_max = bl_max.max(tp.beat_length);
        }

        Arc::new(BeatmapMeta { 
            file_path: self.filename.clone(), 
            beatmap_hash: self.hash, 
            beatmap_type: BeatmapType::Tja, 
            mode: "taiko".to_owned(), 
            artist: self.subtitle.clone(), 
            title: self.title.clone(), 
            artist_unicode: self.subtitle_unicode.clone(), 
            title_unicode: self.title_unicode.clone(), 
            creator: self.course_creator.clone(), 
            version: self.course_name.clone(), 
            audio_filename: format!("{}/{}", self.directory, self.audio_path), 
            image_filename: format!("{}/{}", self.directory, self.image_path), 
            audio_preview: self.preview_time, 
            duration, 
            bpm_min: 60_000.0 / bl_min, 
            bpm_max: 60_000.0 / bl_max,

            ..Default::default()
        })
    }


    fn get_events(&self) -> Vec<IngameEvent> { Vec::new() }
}


#[derive(Copy, Clone, Debug)]
pub struct TjaCourseEvent {
    pub time: f32,
    pub event: TjaCourseEventType,
}

#[derive(Copy, Clone, Debug)]
pub enum TjaCourseEventType {
    /// bpm changed
    Bpm(f32),
    /// scroll changed
    Scroll(f32),
    /// kiai changed
    Kiai(bool),
    /// measure changed
    Measure(u8),
    /// bar line visibility changed
    BarLine(bool),

    /// section start
    Section,
    /// branch start
    Branch,
}



#[test]
fn test() {
    let path = "C:/Users/Eve/Desktop/tja/The Magician/The Magician.tja";
    
    let res = TjaBeatmap::load_multiple(path);
    println!("{res:?}");
}
