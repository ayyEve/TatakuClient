/**
 * derived from beyley's ptyping game: https://github.com/Beyley/pTyping
 * src: https://github.com/Beyley/pTyping/blob/master/pTyping/Songs/SongLoaders/UTypingSongHandler.cs
 */

use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct PTypingBeatmap {
    // paths etc
    pub hash: Md5Hash,
    pub file_path: String,
    pub parent_dir: String,

    pub artist: PTypingMetadataText,
    pub title: PTypingMetadataText,

    pub def: PTypingBeatmapDef,
    
    // /// when does the first note happen?
    // start_time: f32,
    /// how long is the map from first note to last
    duration: f32,
}
impl PTypingBeatmap {
    pub fn load_multiple(path: impl AsRef<Path>) -> TatakuResult<Vec<Self>> {
        let path = path.as_ref();
        let parent_dir = path.parent().unwrap().to_string_lossy().to_string();
        let file_path = path.to_string_lossy().to_string();

        let mut data = std::fs::read(path)?;
        // while data[0] != 0x7b {data = data[1..].to_vec()}
        // if theres random useless bom data, remove it
        if data[0..3] == [0xEF, 0xBB, 0xBF] {
            data = data[3..].to_vec();
        }

        // println!("got data {}", String::from_utf8(data[0..100].to_vec()).unwrap());
        let data: PTypingMapDef = serde_json::from_slice(&data)?;

        let maps = data.beatmaps.into_iter().map(|def| {
            let start_time = def.hit_objects.first().map(|n|n.time).unwrap_or_default() as f32;
            let end_time = def.hit_objects.last().map(|n|n.time).unwrap_or_default() as f32;
            let duration = end_time - start_time;
            
            PTypingBeatmap {
                file_path: file_path.clone(),
                parent_dir: parent_dir.clone(),
                artist: data.artist.clone(),
                title: data.title.clone(),
                hash: (&def.id).try_into().unwrap(),
                def,
                // start_time,
                duration,
            }

        }).collect();

        Ok(maps)
    }

    pub fn load_single(path:impl AsRef<Path>, meta: &BeatmapMeta) -> TatakuResult<Self> {
        let maps = Self::load_multiple(path)?;

        maps.into_iter().find(|m| m.hash == meta.beatmap_hash).ok_or_else(|| BeatmapError::InvalidFile.into())
    }
}
impl TatakuBeatmap for PTypingBeatmap {
    fn hash(&self) -> Md5Hash {
        self.hash
        // self.hash.clone()
    }
    fn playmode(&self, _incoming:String) -> String { "utyping".to_owned() }

    fn get_timing_points(&self) -> Vec<TimingPoint> {
        let point = self.def.timing_points.first().unwrap();
        vec![TimingPoint {
            time: point.time as f32,
            beat_length: point.tempo as f32,
            volume: 100,
            meter: point.time_signature as u8,
            kiai: false,
            skip_first_barline: false,
            sample_set: 0,
            sample_index: 0,
        }]
    }

    fn get_beatmap_meta(&self) -> Arc<BeatmapMeta> {
        // TODO: this
        let bpm = 100.0; //60_000.0 / self.beat_length;
        Arc::new(BeatmapMeta { 
            file_path: self.file_path.clone(), 
            beatmap_hash: self.hash(), 
            beatmap_type: BeatmapType::UTyping,
            mode: "utyping".to_owned(), 


            artist: self.artist.ascii.clone().unwrap_or_default(), 
            title: self.title.ascii.clone().unwrap_or_default(), 
            artist_unicode: self.artist.unicode.clone().unwrap_or_default(), 
            title_unicode: self.title.unicode.clone().unwrap_or_default(), 

            creator: self.def.info.mapper.username.clone(), 
            version: self.def.info.difficulty_name.get_string(), 

            audio_filename: self.parent_dir.clone() + "/files/" + &self.def.file_collection.audio.hash, 
            image_filename: self.parent_dir.clone() + "/files/" + &self.def.file_collection.background.as_ref().map(|f|f.hash.clone()).unwrap_or("none.png".to_owned()),
            audio_preview: self.def.info.preview_time, 
            duration: self.duration, 
            hp: 0.0, 
            od: 0.0, 
            cs: 0.0, 
            ar: 0.0, 
            bpm_min: bpm, 
            bpm_max: bpm, 
        })
    }
}



#[test]
fn test() {
    let path = "C:/Users/Eve/Desktop/ptyping/song";
    let map = PTypingBeatmap::load_multiple(path).unwrap();
    let map = map.first().unwrap();
    println!("{:?}, {:?}", map.artist, map.title)
}
