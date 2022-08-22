use crate::prelude::*;

#[derive(Default)]
pub struct DiffCalcArgs {
    pub gamemode: Option<String>,
    pub map: Option<String>,
    pub mods: Option<String>,
    pub export_file: Option<String>,
    pub export_type: Option<String>
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

                _ => {}
            }
        }

        data
    }
}

// entry point for this command
// TODO!: implement mods
pub async fn diff_calc_cli(args: &mut impl Iterator<Item = String>) {
    info!("performing diff calc...");

    // parse cli args
    let args = DiffCalcArgs::from_args(args);
    
    // load maps from db
    let maps = Database::get_all_beatmaps().await;

    if let Some(map_hash_or_path) = &args.map {
        let mut map = None;
        
        // try to find by hash or path
        for i in maps.iter() {
            if &i.beatmap_hash == map_hash_or_path {
                map = Some(i.clone())
            }
        }
        if map.is_none() {
            match Beatmap::load_multiple(map_hash_or_path) {
                Ok(maps) => map = maps.get(0).map(|m|m.get_beatmap_meta()),
                Err(e) => panic!("error loading beatmap '{}': {}", map_hash_or_path, e),
            }
        }

        if let Some(map) = map {
            let playmode = if let Some(mode) = &args.gamemode {
                map.check_mode_override(mode.clone())
            } else {
                map.mode.clone()
            };

            let diff = calc_diff(&map, playmode, &ModManager::new()).await;
            println!("got diff {diff:?}");
        } else {
            panic!("could not find or load map specified")
        }

    } else {
        let mut data = DiffCalcData::default();

        // do this here so we dont waste time calcing everything just to error later
        let export_type = DiffCalcExportType::get_type(
            args.export_type.clone().unwrap_or(
                args.export_file.clone().unwrap_or("csv".to_owned()).split(".").last().unwrap_or("csv").to_string()
            )
        ).expect("unknown file export type");

        // perform diff calc on every 
        for i in maps.iter() {
            let playmode = if let Some(mode) = &args.gamemode {
                let new_mode = i.check_mode_override(mode.clone());

                // if a mode is specified, and its not the same as the map (and can't be converted), dont run diffcalc for this map
                if &new_mode != mode { continue }

                new_mode
            } else {
                i.mode.clone()
            };

            let diff = calc_diff(&i, playmode, &ModManager::new()).await;
            data.add(&i, diff.unwrap_or(-1.0));
        }

        let file_data = data.export(export_type);
        std::fs::write(args.export_file.clone().unwrap_or(format!("diff_calc.{}", export_type.ext())), file_data).expect("error writing test.csv");
    }


}


#[derive(Copy, Clone)]
enum DiffCalcExportType {
    Csv,
    Json,
}
impl DiffCalcExportType {
    pub fn get_type(s: String) -> Option<DiffCalcExportType> {
        match &*s.to_lowercase() {
            "csv" => Some(DiffCalcExportType::Csv),
            "json" => Some(DiffCalcExportType::Json),
            _ => None
        }
    }
    pub fn ext(&self) -> &str {
        match self {
            DiffCalcExportType::Csv  => "csv",
            DiffCalcExportType::Json => "json",
        }
    }
}


#[derive(Default, Clone)]
struct DiffCalcData(Vec<(Arc<BeatmapMeta>, f32)>);
impl DiffCalcData {
    pub fn add(&mut self, map: &Arc<BeatmapMeta>, diff: f32) {
        self.0.push((map.clone(), diff))
    }

    pub fn export(&self, export_type: DiffCalcExportType) -> String {
        match export_type {
            DiffCalcExportType::Csv => {
                let mut lines = Vec::new();
                lines.push(format!("Hash,Title,Artist,Version,Diff"));

                for (i, diff) in &self.0 {
                    lines.push(format!(
                        "\"{}\",\"{}\",\"{}\",\"{}\",{}",
                        i.beatmap_hash,
                        i.title,
                        i.artist,
                        i.version,
                        diff
                    ));
                }

                lines.join("\n")
            }

            DiffCalcExportType::Json => {

                #[derive(Serialize)]
                struct Data {
                    hash: String,
                    title: String,
                    artist: String,
                    version: String,
                    diff: f32
                }

                let data: Vec<Data> = self.0.iter().map(|(b, d)| {
                    Data {
                        hash: b.beatmap_hash.clone(),
                        title: b.title.clone(),
                        artist: b.artist.clone(),
                        version: b.version.clone(),
                        diff: *d
                    }
                }).collect();

                serde_json::to_string_pretty(&data).expect("Error serializing data. please report this.")
            }
        }

    }
}
impl Deref for DiffCalcData {
    type Target = Vec<(Arc<BeatmapMeta>, f32)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for DiffCalcData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
