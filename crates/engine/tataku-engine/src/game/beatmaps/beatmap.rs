use super::*;
pub use super::common::*;

use crate::*;
use tataku_common::Md5Hash;
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
    #[default] None,

    /// osu file
    Osu(Box<osu::Beatmap>),

    /// quaver file
    Quaver(Box<quaver::Beatmap>),
    
    /// adofai file
    Adofai(Box<adofai::Beatmap>),

    /// uTyping beatmap
    UTyping(Box<utyping::Beatmap>),

    /// uTyping beatmap
    PTyping(Box<ptyping::Beatmap>),

    /// Tja beatmap
    Tja(Box<tja::Beatmap>),

    /// Stepmania beatmap
    Stepmania(Box<stepmania::Beatmap>),
}
impl Beatmap {
    pub fn load_multiple(path: impl AsRef<Path>) -> tataku::Result<Vec<Self>> {
        let path = path.as_ref();
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            // check for ptyping file (it has no extention)
            if path.file_name().unwrap().to_string_lossy() == "song" {
                return Ok(ptyping::Beatmap::load_multiple(path)?
                    .into_iter()
                    .map(Self::from)
                    .collect()
                );
            } else {
                return Err(errors::beatmap::Error::InvalidFile.into())
            }
        };

        match ext {
            "osu" => Ok(vec![osu::Beatmap::load(path)?.into()]),
            "qua" => Ok(vec![quaver::Beatmap::load(path)?.into()]),
            "adofai" => Ok(vec![adofai::Beatmap::load(path).into()]),
            "txt" => Ok(vec![utyping::Beatmap::load(path)?.into()]),
            "ssc" | "sm" => Ok(stepmania::Beatmap::load_multiple(path)?
                .into_iter()
                .map(Self::from)
                .collect()
            ),
            "tja" => Ok(tja::Beatmap::load_multiple(path)?
                .into_iter()
                .map(Self::from)
                .collect()
            ),

            _ => Err(errors::beatmap::Error::InvalidFile.into()),
        }
    }
    pub fn load_single(
        path: impl AsRef<Path>, 
        meta: &BeatmapMeta,
    ) -> tataku::Result<Self> {
        let path = path.as_ref();
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            // check for ptyping file (it has no extention)
            if path.file_name().and_then(|a| a.to_str()).filter(|a| a.ends_with("song")).is_some() {
                return Ok(ptyping::Beatmap::load_single(path, meta)?.into())
            } else {
                return Err(errors::beatmap::Error::InvalidFile.into())
            }
        };

        match ext {
            "osu" => Ok(osu::Beatmap::load(path)?.into()),
            "qua" => Ok(quaver::Beatmap::load(path)?.into()),
            "adofai" => Ok(adofai::Beatmap::load(path).into()),
            "txt" => Ok(utyping::Beatmap::load(path)?.into()),
            "ssc" | "sm" => Ok(stepmania::Beatmap::load_single(path, meta)?.into()),
            "tja" => Ok(tja::Beatmap::load_single(path, meta)?.into()),
            
            _ => Err(errors::beatmap::Error::InvalidFile.into()),
        }
    }

    /// loading metadata only is way faster if only the meta is needed
    pub fn load_multiple_metadata(
        path: impl AsRef<Path>
    ) -> tataku::Result<Vec<Arc<BeatmapMeta>>> {
        let path = path.as_ref();
        if path.extension().is_none() {
            // check for ptyping file (it has no extention)
            if path.file_name().and_then(|a|a.to_str()).filter(|a|a.ends_with("song")).is_some() {
                return Ok(ptyping::PTypingBeatmap::load_multiple(path)?.into_iter().map(|b|b.get_beatmap_meta()).collect())
            } else {
                return Err(errors::beatmap::BeatmapError::InvalidFile.into())
            }
        } 
        
        match path.extension().unwrap().to_str().unwrap() {
            "osu" => Ok(vec![osu::OsuBeatmap::load_metadata(path.to_str().unwrap())?]),
            "qua" => Ok(vec![quaver::QuaverBeatmap::load(path.to_str().unwrap())?.get_beatmap_meta()]),
            "adofai" => Ok(vec![adofai::AdofaiBeatmap::load(path.to_str().unwrap()).get_beatmap_meta()]),
            "txt" => Ok(vec![utyping::UTypingBeatmap::load(path)?.get_beatmap_meta()]),
            "ssc" | "sm" => Ok(stepmania::StepmaniaBeatmap::load_multiple(path)?.into_iter().map(|b|b.get_beatmap_meta()).collect()),
            "tja" => Ok(tja::TjaBeatmap::load_multiple(path)?.into_iter().map(|b|b.get_beatmap_meta()).collect()),

            _ => Err(errors::beatmap::BeatmapError::InvalidFile.into()),
        }
    }
    
    pub fn from_path_and_hash(path: impl AsRef<Path>, hash: Md5Hash) -> tataku::Result<Self> {
        Self::load_multiple(path)?
            .into_iter()
            .find(|b| b.get_beatmap_meta().beatmap_hash == hash)
            .ok_or(errors::beatmap::Error::NotFoundInSet.into())
    }
    pub fn from_metadata(meta: &BeatmapMeta) -> tataku::Result<Self> {
        Self::load_single(&*meta.file_path, meta)
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


impl From<osu::Beatmap> for Beatmap {
    fn from(value: osu::Beatmap) -> Self {
        Self::Osu(Box::new(value))
    }
}
impl From<quaver::Beatmap> for Beatmap {
    fn from(value: quaver::Beatmap) -> Self {
        Self::Quaver(Box::new(value))
    }
}
impl From<adofai::Beatmap> for Beatmap {
    fn from(value: adofai::Beatmap) -> Self {
        Self::Adofai(Box::new(value))
    }
}
impl From<utyping::Beatmap> for Beatmap {
    fn from(value: utyping::Beatmap) -> Self {
        Self::UTyping(Box::new(value))
    }
}
impl From<ptyping::Beatmap> for Beatmap {
    fn from(value: ptyping::Beatmap) -> Self {
        Self::PTyping(Box::new(value))
    }
}
impl From<tja::Beatmap> for Beatmap {
    fn from(value: tja::Beatmap) -> Self {
        Self::Tja(Box::new(value))
    }
}
impl From<stepmania::Beatmap> for Beatmap {
    fn from(value: stepmania::Beatmap) -> Self {
        Self::Stepmania(Box::new(value))
    }
}
