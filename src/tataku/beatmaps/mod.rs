use crate::prelude::*;

mod common;

mod osu;
mod tja;
mod quaver;
mod adofai;
mod ptyping;
mod u_typing;
mod stepmania;

pub use tja::*;
pub use osu::*;
pub use common::*;
pub use quaver::*;
pub use adofai::*;
pub use ptyping::*;
pub use u_typing::*;
pub use stepmania::*;

pub const AVAILABLE_MAP_EXTENSIONS: &[&str] = &[
    ".osu", // osu
    ".qua", // quaver
    ".adofai", // a dance of fire and ice
    ".ssc", // step mania
    ".sm", // also step mania
    ".tja", // tja

    "info.txt", // utyping
    "song", // ptyping
];

#[derive(Default)]
pub enum Beatmap {
    /// used for defaults
    #[default]
    None,
    /// osu file
    Osu(Box<osu::OsuBeatmap>),
    /// quaver file
    Quaver(Box<quaver::QuaverBeatmap>),
    /// adofai file
    Adofai(Box<adofai::AdofaiBeatmap>),
    /// uTyping beatmap
    UTyping(Box<u_typing::UTypingBeatmap>),
    /// uTyping beatmap
    PTyping(Box<ptyping::PTypingBeatmap>),
    /// Tja beatmap
    Tja(Box<tja::TjaBeatmap>),
    /// Stepmania beatmap
    Stepmania(Box<stepmania::StepmaniaBeatmap>),
}
impl Beatmap {
    pub fn load_multiple<F:AsRef<Path>>(path: F) -> TatakuResult<Vec<Beatmap>> {
        let path = path.as_ref();
        if path.extension().is_none() {
            // check for ptyping file (it has no extention)
            // println!("path: {path:?}");
            if path.file_name().unwrap().to_string_lossy().to_string() == "song" {
                return Ok(ptyping::PTypingBeatmap::load_multiple(path)?.into_iter().map(|b|Beatmap::PTyping(Box::new(b))).collect())
            } else {
                return Err(TatakuError::Beatmap(BeatmapError::InvalidFile))
            }
        }
        
        match path.extension().unwrap().to_str().unwrap() {
            "osu" => Ok(vec![Beatmap::Osu(Box::new(osu::OsuBeatmap::load(path.to_str().unwrap().to_owned())?))]),
            "qua" => Ok(vec![Beatmap::Quaver(Box::new(quaver::QuaverBeatmap::load(path.to_str().unwrap().to_owned())?))]),
            "adofai" => Ok(vec![Beatmap::Adofai(Box::new(adofai::AdofaiBeatmap::load(path.to_str().unwrap().to_owned())))]),
            "txt" => Ok(vec![Beatmap::UTyping(Box::new(u_typing::UTypingBeatmap::load(path)?))]),
            "ssc" | "sm" => Ok(stepmania::StepmaniaBeatmap::load_multiple(path)?.into_iter().map(|b|Beatmap::Stepmania(Box::new(b))).collect()),
            "tja" => Ok(tja::TjaBeatmap::load_multiple(path)?.into_iter().map(|b|Beatmap::Tja(Box::new(b))).collect()),

            _ => Err(TatakuError::Beatmap(BeatmapError::InvalidFile)),
        }
    }
    pub fn load_single<F:AsRef<Path>>(path: F, meta: &BeatmapMeta) -> TatakuResult<Beatmap> {
        let path = path.as_ref();
        if path.extension().is_none() {
            // check for ptyping file (it has no extention)
            if path.file_name().and_then(|a|a.to_str()).filter(|a|a.ends_with("song")).is_some() {
                return Ok(Beatmap::PTyping(Box::new(ptyping::PTypingBeatmap::load_single(path, meta)?)))
            } else {
                return Err(TatakuError::Beatmap(BeatmapError::InvalidFile))
            }
        }
        
        match path.extension().unwrap().to_str().unwrap() {
            "osu" => Ok(Beatmap::Osu(Box::new(osu::OsuBeatmap::load(path.to_str().unwrap().to_owned())?))),
            "qua" => Ok(Beatmap::Quaver(Box::new(quaver::QuaverBeatmap::load(path.to_str().unwrap().to_owned())?))),
            "adofai" => Ok(Beatmap::Adofai(Box::new(adofai::AdofaiBeatmap::load(path.to_str().unwrap().to_owned())))),
            "txt" => Ok(Beatmap::UTyping(Box::new(u_typing::UTypingBeatmap::load(path.to_str().unwrap().to_owned())?))),
            "ssc" | "sm" => Ok(Beatmap::Stepmania(Box::new(stepmania::StepmaniaBeatmap::load_single(path, meta)?))),
            "tja" => Ok(Beatmap::Tja(Box::new(tja::TjaBeatmap::load_single(path, meta)?))),
            
            _ => Err(TatakuError::Beatmap(BeatmapError::InvalidFile)),
        }
    }

    /// loading metadata only is way faster if only the meta is needed
    pub fn load_multiple_metadata(path: impl AsRef<Path>) -> TatakuResult<Vec<Arc<BeatmapMeta>>> {
        let path = path.as_ref();
        if path.extension().is_none() {
            // check for ptyping file (it has no extention)
            if path.file_name().and_then(|a|a.to_str()).filter(|a|a.ends_with("song")).is_some() {
                return Ok(ptyping::PTypingBeatmap::load_multiple(path)?.into_iter().map(|b|b.get_beatmap_meta()).collect())
            } else {
                return Err(TatakuError::Beatmap(BeatmapError::InvalidFile))
            }
        } 
        
        match path.extension().unwrap().to_str().unwrap() {
            "osu" => Ok(vec![osu::OsuBeatmap::load_metadata(path.to_str().unwrap().to_owned())?]),
            "qua" => Ok(vec![quaver::QuaverBeatmap::load(path.to_str().unwrap().to_owned())?.get_beatmap_meta()]),
            "adofai" => Ok(vec![adofai::AdofaiBeatmap::load(path.to_str().unwrap().to_owned()).get_beatmap_meta()]),
            "txt" => Ok(vec![u_typing::UTypingBeatmap::load(path)?.get_beatmap_meta()]),
            "ssc" | "sm" => Ok(stepmania::StepmaniaBeatmap::load_multiple(path)?.into_iter().map(|b|b.get_beatmap_meta()).collect()),
            "tja" => Ok(tja::TjaBeatmap::load_multiple(path)?.into_iter().map(|b|b.get_beatmap_meta()).collect()),

            _ => Err(TatakuError::Beatmap(BeatmapError::InvalidFile)),
        }
    }
    
    pub fn from_path_and_hash(path: impl AsRef<Path>, hash: Md5Hash) -> TatakuResult<Beatmap> {
        let maps = Self::load_multiple(path)?;
        maps
            .into_iter()
            .find(|b| b.get_beatmap_meta().comp_hash(hash))
            .ok_or(TatakuError::Beatmap(BeatmapError::NotFoundInSet))
        // if maps.len() > 1 {
        // } else {

        // }
    }
    pub fn from_metadata(meta: &BeatmapMeta) -> TatakuResult<Beatmap> {
        Self::load_single(&meta.file_path, meta)
    }

    pub fn get_parent_dir(&self) -> Option<PathBuf> {
        if let Self::None = self { return None }

        (**self).get_beatmap_meta().get_parent_dir()
    }
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
            Beatmap::PTyping(map) => &**map,
            Beatmap::Stepmania(map) => &**map,
            Beatmap::Tja(map) => &**map,
        }
    }
}
