use crate::prelude::*;


// entry point for this command
// TODO!: implement mods
pub async fn diff_calc_cli(args: &mut impl Iterator<Item = String>) {
    info!("performing diff calc...");

    // parse cli args
    let args = DiffCalcArgs::from_args(args);

    // load maps from db
    let maps = Database::get_all_beatmaps().await;

    if let Some(replay_path) = args.replay_file {
        // load the replay file
        let bytes = Io::read_file_async(&replay_path).await.expect("error reading replay file provided");
        let mut reader = SerializationReader::new(bytes);
        let replay:Replay = reader.read().expect("error parsing replay file provided");
        let score = replay.score_data.expect("This replay has no score data");
        println!("got score playmode {}", score.playmode);

        // load the map
        let mut map = None;
        if let Some(map_hash_or_path) = args.map.as_ref().cloned().or_else(||Some(score.beatmap_hash.to_string())) {

            // try to find by hash or path
            if let Ok(hash) = map_hash_or_path.clone().try_into() {
                for i in maps.iter() {
                    if &i.beatmap_hash == &hash {
                        map = Some(i.clone())
                    }
                }
            }
            if map.is_none() {
                match Beatmap::load_multiple(&map_hash_or_path) {
                    Ok(maps) => {
                        map = maps.get(0).map(|m|m.get_beatmap_meta());
                        if let Some(map) = &map {
                            // add it to the beatmap manager
                            BEATMAP_MANAGER.write().await.add_beatmap(map);
                        }
                    },
                    Err(e) => panic!("error loading beatmap '{map_hash_or_path}': {e}"),
                }
            }
        }
        let map = map.expect("no beatmap?");

        // get the diff

        // load existing diffs
        init_diffs(None).await;

        // make sure everythings updated
        for mode in AVAILABLE_PLAYMODES {
            do_diffcalc(mode.to_owned().to_owned()).await;
        }

        let mut mods = ModManager::default();
        mods.speed = score.speed;

        let diff = get_diff(&map, &score.playmode, &mods).unwrap_or_default();

        // calc the performance
        let info = get_gamemode_info(&score.playmode).unwrap();
        let perf_fn = info.get_perf_calc();
        let perf = (perf_fn)(diff, score.accuracy as f32);
        println!("got perf: {perf}");
        return;
    }


    // get the export type

    // do this here so we dont waste time calcing everything just to error later
    let export_type = DiffCalcExportType::get_type(
        args.export_type.clone().unwrap_or(
            args.export_file.clone().unwrap_or("csv".to_owned()).split(".").last().unwrap_or("csv").to_string()
        )
    ).expect("unknown file export type");

    // setup data to export
    let mut data = DiffCalcData::default();


    // calc the data
    if let Some(map_hash_or_path) = &args.map {
        let mut map = None;
        
        // try to find by hash or path
        if let Ok(hash) = map_hash_or_path.try_into() {
            for i in maps.iter() {
                if &i.beatmap_hash == &hash {
                    map = Some(i.clone())
                }
            }
        }
        if map.is_none() {
            match Beatmap::load_multiple(map_hash_or_path) {
                Ok(maps) => map = maps.get(0).map(|m|m.get_beatmap_meta()),
                Err(e) => panic!("error loading beatmap '{}': {}", map_hash_or_path, e),
            }
        }

        let map = map.expect("could not find or load map specified");
        let playmode = if let Some(mode) = &args.gamemode {
            map.check_mode_override(mode.clone())
        } else {
            map.mode.clone()
        };

        let info = get_gamemode_info(&playmode).unwrap();
        let mut calc = info.create_diffcalc(&map).await.expect("error creating diffcalc");
        let mod_mutations = vec![ModManager::default()];
        
        for speed in (50..200).step_by(5) {
            for mut mods in mod_mutations.clone() {
                mods.set_speed(speed as f32 / 100.0);

                let diff = calc.calc(&mods).await.unwrap_or_default().diff.normal_or(0.0);
                data.add(&map, mods.speed.as_u16() as u32, diff, playmode.clone());
            }
        }

    } else {

        // // perform diff calc on every 
        // for i in maps.iter() {
        //     let playmode = if let Some(mode) = &args.gamemode {
        //         let new_mode = i.check_mode_override(mode.clone());
        //         // if a mode is specified, and its not the same as the map (and can't be converted), dont run diffcalc for this map
        //         if &new_mode != mode { continue }
        //         new_mode
        //     } else {
        //         i.mode.clone()
        //     };
        //     let diff = calc_diff(&i, playmode).await.unwrap().calc(&ModManager::new()).await;
        //     data.add(&i, diff.unwrap_or(-1.0));
        // }

        // load existing diffs
        init_diffs(None).await;

        // make sure everythings updated
        for mode in AVAILABLE_PLAYMODES {
            do_diffcalc(mode.to_owned().to_owned()).await;
        }

        // do the thing
        // for (mode, diffs) in BEATMAP_DIFFICULTIES.iter() {
        //     let mut manager = BEATMAP_MANAGER.write().await;
        //     for map in Database::get_all_beatmaps().await {
        //         manager.add_beatmap(&map);
        //     }

        //     for (a, b) in &*diffs.read().unwrap() {
        //         let hash = radix_fmt::radix(a.map_hash, 16).to_string();
        //         if let Some(map) = manager.get_by_hash(&hash) {
        //             data.add(&map, a.mods.speed as u32, *b, mode.clone());
        //         }
        //     }
        // }
        

        
    }

    let file_data = data.export(export_type);
    std::fs::write(args.export_file.clone().unwrap_or(format!("diff_calc.{}", export_type.ext())), file_data).expect("error writing test.csv");

    println!("calc done");
}

#[derive(Default)]
pub struct DiffCalcArgs {
    pub gamemode: Option<String>,
    pub map: Option<String>,
    pub mods: Option<String>,
    pub export_file: Option<String>,
    pub export_type: Option<String>,

    pub replay_file: Option<String>,
}
impl DiffCalcArgs {
    pub fn from_args(args: &mut impl Iterator<Item = String>) -> Self {
        let mut data = Self::default();

        while let Some(other_arg) = args.next() {
            match &*other_arg {
                "--mode" => data.gamemode = args.next(),
                "--map"  => data.map = args.next(),
                "--mods" => data.mods = args.next(),
                "--export_file" => data.export_file = args.next(),
                "--export_type" => data.export_type = args.next(),

                "--replay_file" => data.replay_file = args.next(),

                _ => {}
            }
        }

        data
    }
}

#[derive(Copy, Clone)]
enum DiffCalcExportType {
    Csv,
    Json,
    Db
}
impl DiffCalcExportType {
    pub fn get_type(s: String) -> Option<DiffCalcExportType> {
        match &*s.to_lowercase() {
            "csv" => Some(DiffCalcExportType::Csv),
            "json" => Some(DiffCalcExportType::Json),
            "db" => Some(DiffCalcExportType::Db),
            _ => None
        }
    }
    pub fn ext(&self) -> &str {
        match self {
            DiffCalcExportType::Csv  => "csv",
            DiffCalcExportType::Json => "json",
            DiffCalcExportType::Db => "db",
        }
    }
}


#[derive(Default, Clone)]
struct DiffCalcData(Vec<(Arc<BeatmapMeta>, u32, f32, String)>);
impl DiffCalcData {
    pub fn add(&mut self, map: &Arc<BeatmapMeta>, speed: u32, diff: f32, playmode: String) {
        self.0.push((map.clone(), speed, diff, playmode))
    }

    pub fn export(&self, export_type: DiffCalcExportType) -> Vec<u8> {
        match export_type {
            DiffCalcExportType::Csv => {
                let mut lines = Vec::new();
                lines.push(format!("Hash,Title,Artist,Version,Speed,Diff,Playmode"));

                for (i, speed, diff, mode) in &self.0 {
                    lines.push(format!(
                        "\"{}\",\"{}\",\"{}\",\"{}\",{},{},\"{}\"",
                        i.beatmap_hash,
                        i.title,
                        i.artist,
                        i.version,
                        speed, 
                        diff,
                        mode
                    ));
                }

                lines.join("\n").into_bytes()
            }

            DiffCalcExportType::Json => {
                #[derive(Serialize)]
                struct Data {
                    hash: String,
                    title: String,
                    artist: String,
                    version: String,
                    speed: u32,
                    diff: f32,
                    playmode: String
                }

                let data: Vec<Data> = self.0.iter().map(|(b, s, d, m)| {
                    Data {
                        hash: b.beatmap_hash.to_string(),
                        title: b.title.clone(),
                        artist: b.artist.clone(),
                        version: b.version.clone(),
                        speed: *s,
                        diff: *d,
                        playmode: m.clone()
                    }
                }).collect();

                serde_json::to_string_pretty(&data).expect("Error serializing data. please report this.").into_bytes()
            }

            DiffCalcExportType::Db => {
                let mut db = SerializationWriter::new();
                db.write(&self.0.len());

                for (map, speed, diff, playmode) in &self.0 {
                    db.write(&map.beatmap_hash);
                    db.write(playmode);
                    db.write(speed);
                    db.write(diff);
                }

                db.data()
            }
        }

    }
}
impl Deref for DiffCalcData {
    type Target = Vec<(Arc<BeatmapMeta>, u32, f32, String)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for DiffCalcData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
