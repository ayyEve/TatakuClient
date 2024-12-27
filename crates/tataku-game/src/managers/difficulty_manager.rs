use crate::prelude::*;

const DIFF_FILE:&str = "diffs.db2";

#[derive(Default)]
pub struct DifficultyManager;
impl DifficultyManager {
    pub async fn save_diff(
        map: &Arc<BeatmapMeta>, 
        playmode: &str, 
        mods: &ModManager,
        diff: f32
    ) -> TatakuResult {
        Self::save_diff_entry(
            DifficultyEntry::new(
            md5(playmode), 
            map.beatmap_hash, 
                mods
            ), 
            diff
        ).await
    }

    pub async fn save_diff_entry(
        entry: DifficultyEntry,
        diff: f32
    ) -> TatakuResult {
        let key = entry.as_key();
        
        cacache::write(DIFF_FILE, key, diff.to_le_bytes())
            .await
            .map_err(|e| TatakuError::String(e.to_string()))
            .map(|_| ())
    }
}


// #[derive(Default)]
// pub struct DifficultyManager {
//     difficulties: HashMap<DifficultyEntry, f32>,
// }
// impl DifficultyManager {

//     pub fn save_all_diffs(&self) -> TatakuResult<()> {
//         let mut writer = SerializationWriter::new();

//         for (entry, value) in self.difficulties.iter() {
//             writer.write(entry);
//             writer.write(value);
//         }
        
//         std::fs::write(DIFF_FILE, writer.data())?;

//         Ok(())
//     }
// }


impl DifficultyProvider for DifficultyManager {
    fn get_diff(
        &mut self, 
        map: &Arc<BeatmapMeta>, 
        playmode: &str, 
        mods: &ModManager
    ) -> TatakuResult<f32> {
        let diff_entry = DifficultyEntry::new(
            md5(playmode), 
            map.beatmap_hash, 
            mods
        );

        let val = match cacache::read_sync(DIFF_FILE, diff_entry.as_key()) {
            Ok(v) => v,
            Err(cacache::Error::EntryNotFound(_, _)) => {
                return Err(TatakuError::DiffCalcError(DiffCalcError::NoDiff));
            }
            Err(e) => return Err(TatakuError::String(e.to_string())),
        };

        if val.len() < 4 { return Err(TatakuError::String("not enough bytes".to_owned())) }

        Ok(f32::from_le_bytes([
            val[0], val[1], val[2], val[3]
        ]))

        // self.difficulties
        //     .get(&diff_entry)
        //     .copied()
        //     .ok_or("No Diff".into())
        // if !AVAILABLE_PLAYMODES.contains(&&**playmode) { return Some(-1.0) }

        // // we dont have mod mutations setup yet so we need to clear mods before we get the diff for a map
        // let mut mods = mods.clone();
        // mods.mods.clear();

        // let diff_key = DifficultyEntry::new(map.beatmap_hash, &mods);
        // BEATMAP_DIFFICULTIES.get(playmode)?.read().unwrap().get(&diff_key).cloned()
    }
}


// // TODO: move to task
// pub async fn init_diffs(status: Option<Arc<RwLock<LoadingStatus>>>) {
//     // if let Some(status) = &status {
//     //     status.write().custom_message = "Reading file...".to_owned();
//     // }

//     // info!("loading diffs");
//     // let all_diffs = match load_all_diffs() {
//     //     Ok(d) => d,
//     //     Err(e) => {
//     //         error!("error loading diffs: {e}");
//     //         let _ = std::fs::rename(DIFF_FILE, DIFF_FILE.to_owned() + "_failed");
//     //         Default::default()
//     //     }
//     // };
//     // if let Some(status) = &status {
//     //     let mut status = status.write();
//     //     status.custom_message.clear();
        
//     //     for i in all_diffs.values() {
//     //         status.item_count += i.len();
//     //     }
//     // }

//     // #[cfg(feature="debug_perf_rating")]
//     // for (k, v) in &all_diffs {
//     //     info!("{k:?} -> {v}")
//     // }

//     // for (mode, diffs) in BEATMAP_DIFFICULTIES.iter() {
//     //     if !AVAILABLE_PLAYMODES.contains(&&**mode) { continue }
//     //     if let Some(loaded_diffs) = all_diffs.get(mode).cloned() {
//     //         let len = loaded_diffs.len();

//     //         *diffs.write().unwrap() = loaded_diffs;
                
//     //         if let Some(status) = &status {
//     //             status.write().items_complete += len;
//     //         }
//     //     }

//     // }

//     // // *BEATMAP_DIFFICULTIES.write().unwrap() = all_diffs;
//     // info!("loading diffs done")
// }



/*
pub async fn do_diffcalc(playmode: String) {
    if !AVAILABLE_PLAYMODES.contains(&&*playmode) { return }
    debug!("diffcalc initiated for mode {playmode}");

    let maps = BEATMAP_MANAGER
        .read()
        .await
        .beatmaps
        .clone();
    let maps = maps
        .iter()
        .filter(|m| m.check_mode_override(playmode.clone()) == playmode);


    let mod_mutations = vec![ModManager::default()];
    let existing = BEATMAP_DIFFICULTIES.get(&playmode).unwrap().read().unwrap().clone();
    let mut data = HashMap::new();

    debug!("diffcalc starting for mode {playmode}");
    for map in maps {
        let mut calc = None;
        let mut calc_failed = false;

        for speed in (50..=1000).step_by(5) { // 0.05..=10.0
            for mut mods in mod_mutations.clone() {
                mods.speed = GameSpeed::from_u16(speed);

                let diff_key = DifficultyEntry::new(map.beatmap_hash, &mods);
                if existing.contains_key(&diff_key) { continue }

                // only load the calc if its actually needed
                if calc.is_none() {
                    if calc_failed {
                        // debug!("calc failed");
                        data.insert(diff_key, -1.0);
                        continue;
                    } else {
                        match calc_diff(map, playmode.clone()).await {
                            Ok(c) => calc = Some(c),
                            Err(e) => {
                                error!("couldnt get calc: {e}");
                                calc = None;
                            }
                        }
                        if calc.is_none() { 
                            data.insert(diff_key, -1.0);
                            calc_failed = true;
                            continue;
                        }
                    }
                }
                
                let diff = calc.as_mut().unwrap().calc(&mods).await.unwrap_or_default().diff.normal_or(0.0);
                
                #[cfg(feature="debug_perf_rating")]
                info!("[calc] {diff_key:?} -> {diff}");
                data.insert(diff_key, diff);
            }
        }
    }

    debug!("diffcalc complete for mode {playmode}, added {} entries", data.len());

    let mut r = BEATMAP_DIFFICULTIES.get(&playmode).unwrap().write().unwrap();
    for (k, v) in data {
        r.insert(k, v);
    }

    debug!("diffcalc fully complete for mode {playmode}, total len is {}", r.len());
    drop(r);
    
    if let Err(e) = save_all_diffs() {
        // TODO: notification?
        error!("error saving diffs: {e}");
    }
}
*/


// fn load_all_diffs() -> TatakuResult<HashMap<String, HashMap<DifficultyEntry, f32>>> {
//     if Io::exists(DIFF_FILE) {
//         let data = Io::read_file(DIFF_FILE)?;
//         let mut reader = SerializationReader::new(data);
//         Ok(reader.read("diffs")?)
//     } else {
//         Ok(Default::default())
//     }
// }


#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct DifficultyEntry {
    pub playmode: Md5Hash,
    pub map_hash: Md5Hash,
    pub mods: Md5Hash,
}

impl DifficultyEntry {
    pub fn new(playmode: impl Into<Md5Hash>, map_hash: Md5Hash, mods: &ModManager) -> Self {
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
        let playmode = sr.read::<u128>("map_hash")?.into();
        let map_hash = sr.read::<u128>("map_hash")?.into();
        let mods = sr.read::<u128>("mods")?.into();

        Ok(Self {
            playmode,
            map_hash,
            mods
        })
    }

    fn write(&self, sw:&mut SerializationWriter) {
        let playmode = self.map_hash.as_ref();
        let map_hash = self.map_hash.as_ref();
        let mods = self.mods.as_ref();
        sw.write(playmode);
        sw.write(map_hash);
        sw.write(mods);
        // sw.write(self.mods.speed);
        // sw.write(self.mods.mods.clone());
    }
}
