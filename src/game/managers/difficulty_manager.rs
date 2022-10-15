use crate::prelude::*;

lazy_static::lazy_static! {
    static ref BEATMAP_DIFFICULTIES: Arc<ShardedLock<HashMap<DifficultyEntry, f32>>> = Arc::new(ShardedLock::new(HashMap::new()));
}


pub async fn init_diffs() {
    let all_diffs = DifficultyDatabase::get_all_diffs().await;
    *BEATMAP_DIFFICULTIES.write().unwrap() = all_diffs;
}


pub async fn get_diff(map: &BeatmapMeta, playmode: &String, mods: &ModManager) -> Option<f32> {
    let diff_key = DifficultyEntry::new(map.beatmap_hash.clone(), playmode.clone(), mods.clone());
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
        .filter(|m|m.check_mode_override(playmode.clone()) == playmode);


    let mod_mutations = vec![ModManager::default()];
    let existing = BEATMAP_DIFFICULTIES.read().unwrap().clone();
    let mut data = HashMap::new();

    info!("diffcalc starting for mode {playmode}");
    for map in maps {
        if let Ok(mut calc) = calc_diff(map, playmode.clone()).await {
            for speed in (50..200).step_by(5) {
                for mut mods in mod_mutations.clone() {
                    mods.set_speed(speed as f32 / 100.0);

                    let diff_key = DifficultyEntry::new(map.beatmap_hash.clone(), playmode.clone(), mods.clone());
                    if existing.contains_key(&diff_key) { continue }

                    let diff = calc.calc(&mods).await.unwrap_or(0.0).normal_or(0.0);
                    data.insert(diff_key, diff);
                }
            }
        }
    }

    info!("diffcalc complete for mode {playmode}, adding {} entries", data.len());
    DifficultyDatabase::insert_many_diffs(&data).await;

    let mut r = BEATMAP_DIFFICULTIES.write().unwrap();
    for (k, v) in data {
        r.insert(k, v);
    }

    info!("diffcalc fully complete for mode {playmode}, total len is {}", r.len());
}



#[derive(Clone, Eq, PartialEq, Hash)]
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

    pub fn to_string(&self) -> String {
        format!(" ")
    }
}
