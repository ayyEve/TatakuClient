use crate::prelude::*;

const DIFF_FILE:&str = "diffs.db";

lazy_static::lazy_static! {
    pub static ref BEATMAP_DIFFICULTIES: Arc<ShardedLock<HashMap<DifficultyEntry, f32>>> = Arc::new(ShardedLock::new(HashMap::new()));
}


pub async fn init_diffs() {
    let all_diffs = match load_all_diffs() {
        Ok(d) => d,
        Err(e) => {
            error!("error loading diffs: {e}");
            Default::default()
        }
    };

    #[cfg(feature="debug_perf_rating")]
    for (k, v) in &all_diffs {
        info!("{k:?} -> {v}")
    }

    *BEATMAP_DIFFICULTIES.write().unwrap() = all_diffs;
}


pub fn get_diff(map: &BeatmapMeta, playmode: &String, mods: &ModManager) -> Option<f32> {
    let mut mods = mods.clone();
    mods.mods.clear();

    let diff_key = DifficultyEntry::new(map.beatmap_hash.clone(), playmode.clone(), mods);
    BEATMAP_DIFFICULTIES.read().unwrap().get(&diff_key).cloned()
}


pub async fn do_diffcalc(playmode: PlayMode) {
    info!("diffcalc initiated for mode {playmode}");

    let maps = BEATMAP_MANAGER
        .read()
        .await
        .beatmaps
        .clone();
    let maps = maps
        .iter()
        .filter(|m| m.check_mode_override(playmode.clone()) == playmode);


    let mod_mutations = vec![ModManager::default()];
    let existing = BEATMAP_DIFFICULTIES.read().unwrap().clone();
    let mut data = HashMap::new();

    info!("diffcalc starting for mode {playmode}");
    for map in maps {
        if let Ok(mut calc) = calc_diff(map, playmode.clone()).await {
            for speed in (50..=1000).step_by(5) { // 0.5..=10.0
                for mut mods in mod_mutations.clone() {
                    mods.speed = speed;

                    let diff_key = DifficultyEntry::new(map.beatmap_hash.clone(), playmode.clone(), mods.clone());
                    if existing.contains_key(&diff_key) { continue }
                    
                    let diff = calc.calc(&mods).await.unwrap_or(0.0).normal_or(0.0);
                    
                    #[cfg(feature="debug_perf_rating")]
                    info!("[calc] {diff_key:?} -> {diff}");
                    data.insert(diff_key, diff);
                }
            }
        }
    }

    info!("diffcalc complete for mode {playmode}, added {} entries", data.len());

    let mut r = BEATMAP_DIFFICULTIES.write().unwrap();
    for (k, v) in data {
        r.insert(k, v);
    }

    info!("diffcalc fully complete for mode {playmode}, total len is {}", r.len());
    drop(r);
    
    if let Err(e) = save_all_diffs() {
        // TODO: notification?
        error!("error saving diffs: {e}");
    }
}



fn load_all_diffs() -> TatakuResult<HashMap<DifficultyEntry, f32>> {
    if exists(DIFF_FILE) {
        let data = std::fs::read(DIFF_FILE)?;
        let mut reader = SerializationReader::new(data);
        Ok(reader.read()?)
    } else {
        Ok(Default::default())
    }
}

fn save_all_diffs() -> TatakuResult<()> {
    let entries = &*BEATMAP_DIFFICULTIES.read().unwrap();
    let bytes = SimpleWriter::new().write(entries.clone()).done();
    Ok(std::fs::write(DIFF_FILE, bytes)?)
}


#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct DifficultyEntry {
    pub map_hash: String,
    pub playmode: String,
    pub mods: ModManager
}

impl DifficultyEntry {
    pub fn new(map_hash: String, playmode: String, mods: ModManager) -> Self {
        Self {
            map_hash,
            playmode,
            mods
        }
    }
}

impl Serializable for DifficultyEntry {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> where Self: Sized {
        let map_hash = sr.read()?;
        let playmode = sr.read()?;
        let speed = sr.read()?;

        let mut mods = ModManager::default();
        mods.speed = speed;

        Ok(Self {
            map_hash,
            playmode,
            mods
        })
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write(self.map_hash.clone());
        sw.write(self.playmode.clone());
        sw.write(self.mods.speed);
    }
}
