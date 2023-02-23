use crate::prelude::*;

mod common;

mod osu;
mod quaver;
mod adofai;
mod u_typing;
mod stepmania;

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
            "osu" => Ok(vec![Beatmap::Osu(Box::new(osu::OsuBeatmap::load(path.to_str().unwrap().to_owned())?))]),
            "qua" => Ok(vec![Beatmap::Quaver(Box::new(quaver::QuaverBeatmap::load(path.to_str().unwrap().to_owned())?))]),
            "adofai" => Ok(vec![Beatmap::Adofai(Box::new(adofai::AdofaiBeatmap::load(path.to_str().unwrap().to_owned())))]),
            "txt" => Ok(vec![Beatmap::UTyping(Box::new(u_typing::UTypingBeatmap::load(path)?))]),
            "ssc" | "sm" => Ok(stepmania::StepmaniaBeatmap::load_multiple(path)?.into_iter().map(|b|Beatmap::Stepmania(Box::new(b))).collect()),

            _ => Err(TatakuError::Beatmap(BeatmapError::InvalidFile)),
        }
    }
    pub fn load_single<F:AsRef<Path>>(path: F, meta: &BeatmapMeta) -> TatakuResult<Beatmap> {
        let path = path.as_ref();
        if path.extension().is_none() {return Err(TatakuError::Beatmap(BeatmapError::InvalidFile))}
        
        match path.extension().unwrap().to_str().unwrap() {
            "osu" => Ok(Beatmap::Osu(Box::new(osu::OsuBeatmap::load(path.to_str().unwrap().to_owned())?))),
            "qua" => Ok(Beatmap::Quaver(Box::new(quaver::QuaverBeatmap::load(path.to_str().unwrap().to_owned())?))),
            "adofai" => Ok(Beatmap::Adofai(Box::new(adofai::AdofaiBeatmap::load(path.to_str().unwrap().to_owned())))),
            "txt" => Ok(Beatmap::UTyping(Box::new(u_typing::UTypingBeatmap::load(path.to_str().unwrap().to_owned())?))),
            "ssc" | "sm" => Ok(Beatmap::Stepmania(Box::new(stepmania::StepmaniaBeatmap::load_single(path, meta)?))),
            
            _ => Err(TatakuError::Beatmap(BeatmapError::InvalidFile)),
        }
    }

    /// loading metadata only is way faster if only the meta is needed
    pub fn load_multiple_metadata(path: impl AsRef<Path>) -> TatakuResult<Vec<Arc<BeatmapMeta>>> {
        let path = path.as_ref();
        if path.extension().is_none() { return Err(TatakuError::Beatmap(BeatmapError::InvalidFile)) }
        
        match path.extension().unwrap().to_str().unwrap() {
            "osu" => Ok(vec![osu::OsuBeatmap::load_metadata(path.to_str().unwrap().to_owned())?]),
            "qua" => Ok(vec![quaver::QuaverBeatmap::load(path.to_str().unwrap().to_owned())?.get_beatmap_meta()]),
            "adofai" => Ok(vec![adofai::AdofaiBeatmap::load(path.to_str().unwrap().to_owned()).get_beatmap_meta()]),
            "txt" => Ok(vec![u_typing::UTypingBeatmap::load(path)?.get_beatmap_meta()]),
            "ssc" | "sm" => Ok(stepmania::StepmaniaBeatmap::load_multiple(path)?.into_iter().map(|b|b.get_beatmap_meta()).collect()),

            _ => Err(TatakuError::Beatmap(BeatmapError::InvalidFile)),
        }
    }
    
    pub fn from_metadata(meta: &BeatmapMeta) -> TatakuResult<Beatmap> {
        Self::load_single(&meta.file_path, meta)
    }
}
impl Default for Beatmap {
    fn default() -> Self { Beatmap::None }
}

impl Deref for Beatmap {
    type Target = dyn TatakuBeatmap;

    fn deref(&self) -> &Self::Target {
        match self {
            Beatmap::None => unimplemented!(),
            Beatmap::Osu(map) => &**map,
            Beatmap::Quaver(map) => &**map,
            Beatmap::Adofai(map) => &**map,
            Beatmap::UTyping(map) => &**map,
            Beatmap::Stepmania(map) => &**map,
        }
    }
}
