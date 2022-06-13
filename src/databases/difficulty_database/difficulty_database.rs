use crate::prelude::*;

lazy_static::lazy_static! {
    pub static ref DIFFICULTY_DATABASE: Arc<DifficultyDatabase> = DifficultyDatabase::new();
}

#[derive(Clone)]
pub struct DifficultyDatabase {
    // connection: Arc<Mutex<Connection>>,
    data: Arc<RwLock<HashMap<DiffInfo, f32>>>
}


impl DifficultyDatabase {
    pub fn new() -> Arc<Self> {
        let file = Path::new("./diffs.json");

        let mut data = HashMap::new();
        if file.exists() {
            if let Ok(file) = std::fs::read(file) {
                if let Ok(parsed) = serde_json::from_slice::<HashMap<String, f32>>(&file) {
                    for (s, diff) in parsed.iter() {
                        if let Some(d) = DiffInfo::from_string(s) {
                            data.insert(d,  *diff);
                        }
                    }
                } else {
                    error!("error parsing diffs")
                }
            } else {
                error!("error opening diffs")
            }
        }

        let data = Arc::new(RwLock::new(data));
        Arc::new(Self {data})
    }



    pub async fn insert_many_diffs(playmode:&PlayMode, mods:&ModManager, diffs:impl Iterator<Item=(String, f32)>) {
        warn!("insert many");
        // let version = 1;

        let mods = mods.as_json().replace("'", "\\'");
        let playmode = playmode.clone();
        
        let mut data = DIFFICULTY_DATABASE.data.write().await;

        for (hash, diff) in diffs {
            let diff_info = DiffInfo::new(hash, mods.clone(), playmode.clone());
            
            if !data.contains_key(&diff_info) {
                data.insert(diff_info, diff);
            }
        }


        let file = Path::new("./diffs.json");
        let mut data2 = HashMap::new();
        for (i, d) in data.iter() {
            data2.insert(i.string(), d);
        }

        std::fs::write(file, serde_json::to_string(&data2).unwrap()).expect("err saving diffs");
    }


    pub async fn get_all_diffs(playmode: &PlayMode, mods: &ModManager) -> HashMap<String, f32> {
        info!("retreive many");

        let mods = mods.as_json().replace("'", "\\'");

        let mut map = HashMap::new();
        
        let data = DIFFICULTY_DATABASE.data.read().await;
        data
            .iter()
            .for_each(
                |(info, diff)| {
                    if &info.mode == playmode && info.mods == mods {
                        map.insert(info.map.clone(), *diff);
                    }
                }
            );
        
        info!("map done");
        map
    }

}



#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Clone)]
struct DiffInfo {
    map: String,
    mods: String,
    mode: String,
}
impl DiffInfo {
    fn new(map: String, mods: String, mode: String) -> Self {
        Self { map, mods, mode }
    }
    fn string(&self) -> String {
        format!("{}|{}|{}", self.map, self.mode, self.mods)
    }
    fn from_string(str: impl AsRef<str>) -> Option<Self> {
        let mut split = str.as_ref().split("|");
        if let (Some(map), Some(mode), Some(mods)) = (split.next(), split.next(), split.next()) {
            Some(Self {
                map: map.to_owned(),
                mode: mode.to_owned(),
                mods: mods.to_owned(),
            })
        } else {
            None
        }
    }
}



#[test]
fn test_diff_db() {
    let runtime = tokio::runtime::Builder::new_current_thread().build().expect("no runtime?");

    runtime.block_on(async {
        let mode = "osu".to_owned();
        let mods = ModManager::new();

        let diffs = vec![
            ("1".to_owned(), 0.1),
            ("2".to_owned(), 0.2),
            ("3".to_owned(), 0.3),
            ("4".to_owned(), 0.4),
        ];

        DifficultyDatabase::insert_many_diffs(&mode, &mods, diffs.iter().map(|i|i.clone())).await;

        println!("insert test done");

        let diffs = DifficultyDatabase::get_all_diffs(&mode, &mods).await;

        
        println!("retreive test done: {}", diffs.len());
    });
}