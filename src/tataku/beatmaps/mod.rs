use crate::prelude::*;

mod common;

mod osu;
mod quaver;
mod adofai;
mod u_typing;
mod stepmania;

use osu::OsuBeatmap;
use quaver::QuaverBeatmap;
use adofai::AdofaiBeatmap;
use u_typing::UTypingBeatmap;
use stepmania::StepmaniaBeatmap;


pub use osu::*;
pub use common::*;
pub use quaver::*;
pub use adofai::*;
pub use u_typing::*;
pub use stepmania::*;

pub enum Beatmap {
    /// used for defaults
    None,
    /// osu file
    Osu(Box<osu::OsuBeatmap>),
    /// quaver file
    Quaver(Box<quaver::QuaverBeatmap>),
    /// adofai file
    Adofai(Box<adofai::AdofaiBeatmap>),
    /// uTyping beatmap
    UTyping(Box<u_typing::UTypingBeatmap>),

    Stepmania(Box<stepmania::StepmaniaBeatmap>),
}
impl Beatmap {
    pub fn load_multiple<F:AsRef<Path>>(path: F) -> TatakuResult<Vec<Beatmap>> {
        let path = path.as_ref();
        if path.extension().is_none() {return Err(TatakuError::Beatmap(BeatmapError::InvalidFile))}
        
        match path.extension().unwrap().to_str().unwrap() {
            "osu" => Ok(vec![Beatmap::Osu(Box::new(OsuBeatmap::load(path.to_str().unwrap().to_owned())?))]),
            "qua" => Ok(vec![Beatmap::Quaver(Box::new(QuaverBeatmap::load(path.to_str().unwrap().to_owned())?))]),
            "adofai" => Ok(vec![Beatmap::Adofai(Box::new(AdofaiBeatmap::load(path.to_str().unwrap().to_owned())))]),
            "txt" => Ok(vec![Beatmap::UTyping(Box::new(UTypingBeatmap::load(path)?))]),
            "ssc" | "sm" => Ok(StepmaniaBeatmap::load_multiple(path)?.into_iter().map(|b|Beatmap::Stepmania(Box::new(b))).collect()),

            _ => Err(TatakuError::Beatmap(BeatmapError::InvalidFile)),
        }
    }
    pub fn load_single<F:AsRef<Path>>(path: F, meta: &BeatmapMeta) -> TatakuResult<Beatmap> {
        let path = path.as_ref();
        if path.extension().is_none() {return Err(TatakuError::Beatmap(BeatmapError::InvalidFile))}
        
        match path.extension().unwrap().to_str().unwrap() {
            "osu" => Ok(Beatmap::Osu(Box::new(OsuBeatmap::load(path.to_str().unwrap().to_owned())?))),
            "qua" => Ok(Beatmap::Quaver(Box::new(QuaverBeatmap::load(path.to_str().unwrap().to_owned())?))),
            "adofai" => Ok(Beatmap::Adofai(Box::new(AdofaiBeatmap::load(path.to_str().unwrap().to_owned())))),
            "txt" => Ok(Beatmap::UTyping(Box::new(UTypingBeatmap::load(path.to_str().unwrap().to_owned())?))),
            "ssc" | "sm" => Ok(Beatmap::Stepmania(Box::new(StepmaniaBeatmap::load_single(path, meta)?))),
            
            _ => Err(TatakuError::Beatmap(BeatmapError::InvalidFile)),
        }
    }

    pub fn from_metadata(meta: &BeatmapMeta) -> TatakuResult<Beatmap> {
        Self::load_single(&meta.file_path, meta)
    }
}
impl Default for Beatmap {
    fn default() -> Self {Beatmap::None}
}
impl TatakuBeatmap for Beatmap {
    fn hash(&self) -> String {
        match self {
            Beatmap::None => todo!(),
            Beatmap::Osu(map) => map.hash(),
            Beatmap::Quaver(map) => map.hash(),
            Beatmap::Adofai(map) => map.hash(),
            Beatmap::UTyping(map) => map.hash(),
            Beatmap::Stepmania(map) => map.hash(),
        }
    }

    fn get_timing_points(&self) -> Vec<TimingPoint> {
        match self {
            Beatmap::None => todo!(),
            Beatmap::Osu(map) => map.get_timing_points(),
            Beatmap::Quaver(map) => map.get_timing_points(),
            Beatmap::Adofai(map) => map.get_timing_points(),
            Beatmap::UTyping(map) => map.get_timing_points(),
            Beatmap::Stepmania(map) => map.get_timing_points(),
        }
    }

    fn get_beatmap_meta(&self) -> Arc<BeatmapMeta> {
        match self {
            Beatmap::None => todo!(),
            Beatmap::Osu(map) => map.get_beatmap_meta(),
            Beatmap::Quaver(map) => map.get_beatmap_meta(),
            Beatmap::Adofai(map) => map.get_beatmap_meta(),
            Beatmap::UTyping(map) => map.get_beatmap_meta(),
            Beatmap::Stepmania(map) => map.get_beatmap_meta(),
        }
    }

    fn playmode(&self, incoming: PlayMode) -> PlayMode {
        match self {
            Beatmap::None => todo!(),
            Beatmap::Osu(map) => map.playmode(incoming),
            Beatmap::Quaver(map) => map.playmode(incoming),
            Beatmap::Adofai(map) => map.playmode(incoming),
            Beatmap::UTyping(map) => map.playmode(incoming),
            Beatmap::Stepmania(map) => map.playmode(incoming),
        }
    }

    fn slider_velocity_at(&self, time:f32) -> f32 {
        match self {
            Beatmap::None => todo!(),
            Beatmap::Osu(map) => map.slider_velocity_at(time),
            Beatmap::Quaver(map) => map.slider_velocity_at(time),
            Beatmap::Adofai(map) => map.slider_velocity_at(time),
            Beatmap::UTyping(map) => map.slider_velocity_at(time),
            Beatmap::Stepmania(map) => map.slider_velocity_at(time),
        }
    }

    fn beat_length_at(&self, time:f32, allow_multiplier:bool) -> f32 {
        match self {
            Beatmap::None => todo!(),
            Beatmap::Osu(map) => map.beat_length_at(time, allow_multiplier),
            Beatmap::Quaver(map) => map.beat_length_at(time, allow_multiplier),
            Beatmap::Adofai(map) => map.beat_length_at(time, allow_multiplier),
            Beatmap::UTyping(map) => map.beat_length_at(time, allow_multiplier),
            Beatmap::Stepmania(map) => map.beat_length_at(time, allow_multiplier),
        }
    }

    fn control_point_at(&self, time:f32) -> TimingPoint {
        match self {
            Beatmap::None => todo!(),
            Beatmap::Osu(map) => map.control_point_at(time),
            Beatmap::Quaver(map) => map.control_point_at(time),
            Beatmap::Adofai(map) => map.control_point_at(time),
            Beatmap::UTyping(map) => map.control_point_at(time),
            Beatmap::Stepmania(map) => map.control_point_at(time),
        }
    }
}
