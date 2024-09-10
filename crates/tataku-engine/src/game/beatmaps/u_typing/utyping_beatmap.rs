/**
 * derived from beyley's ptyping game: https://github.com/Beyley/pTyping
 * src: https://github.com/Beyley/pTyping/blob/master/pTyping/Songs/SongLoaders/UTypingSongHandler.cs
 */

use crate::prelude::*;

#[derive(Clone, Default, Debug)]
pub struct UTypingBeatmap {
    // paths etc
    pub hash: Md5Hash,
    pub file_path: String,
    pub audio_path: String,

    // meta
    pub title: String,
    pub artist: String,
    pub creator: String,
    pub difficulty: String,

    // notes and events
    pub notes: Vec<UTypingNoteDef>,
    pub events: Vec<UTypingEvent>,
    

    // timing info
    
    /// how long between beats (ms)
    beat_length: f32,
    /// when does the first note happen?
    start_time: f32,
    /// how long is the map from first note to last
    map_duration: f32,
}
impl UTypingBeatmap {
    pub fn load<P:AsRef<Path>>(path: P) -> TatakuResult<Self> {
        if let Some(path) = path.as_ref().file_name() {
            if path.to_str() != Some("info.txt") {
                return Err(TatakuError::Beatmap(BeatmapError::InvalidFile));
            }
        }

        let parent_folder = path.as_ref().parent().unwrap().to_string_lossy().to_string();

        let lines = encoding_rs::SHIFT_JIS.decode(Io::read_file(path.as_ref())?.as_slice()).0.to_string().replace("\r","");
        let mut lines = lines.split("\n");

        macro_rules! next {
            ($lines:expr) => {
                $lines.next().ok_or(TatakuError::Beatmap(BeatmapError::InvalidFile))?.to_owned()
            }
        }

        let mut map = Self::default();
        map.title =      next!(lines);
        map.artist =     next!(lines);
        map.creator =    next!(lines);
        map.difficulty = next!(lines);

        let data_filename = next!(lines); 
        let data_filepath = Path::new(&parent_folder).join(data_filename);
        let map_data = encoding_rs::SHIFT_JIS.decode(Io::read_file(&data_filepath)?.as_slice()).0.to_string().replace("\r","");
        if map_data.chars().next() != Some('@') { return Err(TatakuError::Beatmap(BeatmapError::InvalidFile))}

        let map_data_lines = map_data.split("\n");
        for line in map_data_lines {
            if line.is_empty() {continue}

            let first_char = line.chars().next().unwrap();
            let line = line.split_at(1).1;

            match first_char {
                // Contains the relative path to the song file in the format of
                // @path
                // ex. @animariot.ogg
                '@' => map.audio_path = format!("{parent_folder}/{line}"),

                
                // Contains a note in the format of
                // TimeInSeconds CharacterToType
                // ex. +109.041176 だい
                '+' => {
                    let mut split = line.split(" ");
                    
                    let time = next!(split).parse::<f32>().unwrap_or(0.0) * 1000.0;
                    let text = next!(split).trim().to_owned();

                    map.notes.push(UTypingNoteDef {
                        time, 
                        text
                    });
                }


                // Contains the next set of lyrics in the format of
                // *TimeInSeconds Lyrics
                // ex. *16.100000 だいてだいてだいてだいて　もっと
                '*' => {
                    let mut split = line.split(" ");
                    
                    let time = next!(split).parse::<f32>().unwrap_or(0.0) * 1000.0;
                    let mut text = next!(split);
                    // add any cut off (NOTE: ptyping code doesnt do this)
                    while let Some(t) = split.next() {
                        text += &format!(" {t}")
                    }

                    map.events.push(UTypingEvent {
                        time,
                        text,
                        event_type: UTypingEventType::Lyric
                    })
                }


                // Prevents you from typing the previous note in the format of
                //  /TimeInSeconds
                // ex. /17.982353
                '/' => {
                    let mut split = line.split(" ");
                    let time = next!(split).parse::<f32>().unwrap_or(0.0) * 1000.0;
                    
                    map.events.push(UTypingEvent {
                        time,
                        text: String::new(),
                        event_type: UTypingEventType::CutOff
                    })
                }

                
                // A beatline beat (happens every 1/4th beat except for full beats)
                // -TimeInSeconds
                // ex. -17.747059
                '-' => {
                    let mut split = line.split(" ");
                    let time = next!(split).parse::<f32>().unwrap_or(0.0) * 1000.0;
                    
                    map.events.push(UTypingEvent {
                        time,
                        text: String::new(),
                        event_type: UTypingEventType::BeatlineBeat
                    })
                }

                
                // A beatline bar (happens every full beat)
                // =TimeInSeconds
                // ex. =4.544444
                '=' => {
                    let mut split = line.split(" ");
                    let time = next!(split).parse::<f32>().unwrap_or(0.0) * 1000.0;
                    
                    map.events.push(UTypingEvent {
                        time,
                        text: String::new(),
                        event_type: UTypingEventType::BeatlineBar
                    })
                }

                _ => {}
            }
        }


        // finish up processing

        // get the map's beat length
        let list = map.events.iter().filter_map(|e|if e.event_type == UTypingEventType::BeatlineBar {Some(e)} else {None}).collect::<Vec<&UTypingEvent>>();
        if list.len() < 2 {
            warn!("Map does not have enough bar lines?");
            return Err(TatakuError::Beatmap(BeatmapError::InvalidFile));
        }
        map.beat_length = list[1].time - list[0].time;

        // get the file hash
        map.hash = Io::get_file_hash(&data_filepath)?;

        // get the start time
        map.start_time = map.notes[0].time;

        // set the data file path
        map.file_path = path.as_ref().to_string_lossy().to_string();

        // get the map duration
        map.map_duration = map.notes.last().unwrap().time - map.notes[0].time;

        // all done
        Ok(map)
    }
}
impl TatakuBeatmap for UTypingBeatmap {
    fn hash(&self) -> Md5Hash {self.hash}
    fn playmode(&self, _incoming:String) -> String {"utyping".to_owned()}

    fn get_beatmap_meta(&self) -> Arc<BeatmapMeta> {
        let bpm = 60_000.0 / self.beat_length;
        Arc::new(BeatmapMeta { 
            file_path: self.file_path.clone(), 
            beatmap_hash: self.hash.clone(), 
            beatmap_type: BeatmapType::UTyping,
            mode: "utyping".to_owned(), 
            artist: self.artist.clone(), 
            title: self.title.clone(), 
            artist_unicode: self.artist.clone(), 
            title_unicode: self.title.clone(), 
            creator: self.creator.clone(), 
            version: self.difficulty.clone(), 
            audio_filename: self.audio_path.clone(), 
            image_filename: String::new(), // no images for utyping :C 
            audio_preview: 0.0, 
            duration: self.map_duration, 
            hp: 0.0, 
            od: 0.0, 
            cs: 0.0, 
            ar: 0.0, 
            bpm_min: bpm, 
            bpm_max: bpm, 
        })
    }

    fn get_timing_points(&self) -> Vec<TimingPoint> {
        vec![TimingPoint {
            time: self.start_time,
            beat_length: self.beat_length,
            volume: 100,
            meter: 4,
            kiai: false,
            skip_first_barline: false,
            sample_set: 0,
            sample_index: 0,
        }]
    }
}



#[derive(Clone, Default, Debug)]
pub struct UTypingNoteDef {
    pub time: f32,
    pub text: String
}

#[derive(Clone, Debug)]
pub struct UTypingEvent {
    pub time: f32,
    pub text: String,
    pub event_type: UTypingEventType
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum UTypingEventType {
    /// text contains a lyric for this time stamp
    Lyric,
    /// prevents notes before this time from being hit
    CutOff,
    /// indicates a 1/4 beat
    BeatlineBeat,
    /// indicates a full beat
    BeatlineBar,
}




#[test]
fn test() {
    let path = "C:/Users/Eve/Desktop/Projects/rust/tataku/tataku-client/songs/zento/info.txt";
    let map = UTypingBeatmap::load(path).unwrap();
    println!("map: {:?}", map)
}
