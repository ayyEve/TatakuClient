use crate::prelude::*;
use common::{
    Md5Hash,
    serialization::{
        Serializable,
        SerializationReader,
        SerializationWriter,
        SerializationResult,
    },
};
use engine::{
    beatmaps::BeatmapMeta,
    gameplay::mods::Mods,
    database::DifficultyProvider,
};


const DIFF_FILE:&str = "diffs.db2";

#[derive(Default)]
pub struct DifficultyManager;
impl DifficultyManager {
    pub fn save_diff(
        map: &Arc<BeatmapMeta>,
        playmode: &str,
        mods: &Mods,
        diff: f32
    ) -> tataku::Result<()> {
        Self::save_diff_entry(
            DifficultyEntry::new(
            tataku::Cryptography::md5(playmode),
            map.beatmap_hash,
                mods
            ),
            diff
        )
    }

    pub fn save_diff_entry(
        entry: DifficultyEntry,
        diff: f32
    ) -> tataku::Result<()> {
        let key = entry.as_key();

        cacache::write_sync(DIFF_FILE, key, diff.to_le_bytes())
            .map_err(|e| tataku::Error::String(e.to_string()))
            .map(|_| ())
    }
}

impl DifficultyProvider for DifficultyManager {
    fn get_diff(
        &mut self,
        map: &Arc<BeatmapMeta>,
        playmode: &str,
        mods: &Mods
    ) -> tataku::Result<f32> {
        let diff_entry = DifficultyEntry::new(
            tataku::Cryptography::md5(playmode),
            map.beatmap_hash,
            mods
        );

        let val = match cacache::read_sync(DIFF_FILE, diff_entry.as_key()) {
            Ok(v) => v,
            Err(cacache::Error::EntryNotFound(_, _)) => {
                return Err(tataku::Error::DiffCalcError(errors::diffcalc::DiffCalcError::NoDiff));
            }
            Err(e) => return Err(tataku::Error::String(e.to_string())),
        };

        if val.len() < 4 { return Err(tataku::Error::String("not enough bytes".to_owned())) }

        Ok(f32::from_le_bytes(val[0..4].try_into().unwrap()))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct DifficultyEntry {
    pub playmode: Md5Hash,
    pub map_hash: Md5Hash,
    pub mods: Md5Hash,
}
impl DifficultyEntry {
    pub fn new(playmode: impl Into<Md5Hash>, map_hash: Md5Hash, mods: &Mods) -> Self {
        Self {
            playmode: playmode.into(),
            map_hash,
            mods: mods.as_md5()
        }
    }

    pub fn as_key(&self) -> String {
        format!(
            "{}-{}-{}",
            self.playmode,
            self.map_hash,
            self.mods
        )
    }
}

impl Serializable for DifficultyEntry {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> where Self: Sized {
        let playmode = sr.read::<u128>("playmode")?.into();
        let map_hash = sr.read::<u128>("map_hash")?.into();
        let mods = sr.read::<u128>("mods")?.into();

        Ok(Self {
            playmode,
            map_hash,
            mods
        })
    }

    fn write(&self, sw: &mut SerializationWriter) {
        let playmode = self.playmode.as_ref();
        let map_hash = self.map_hash.as_ref();
        let mods = self.mods.as_ref();
        sw.write(playmode);
        sw.write(map_hash);
        sw.write(mods);
    }
}
