use std::{path::Path, fs::DirEntry};

use serde::Deserialize;

pub fn build_gamemodes() {
    // get all gamemodes in the folder
    let cd = std::env::current_dir().unwrap();
    let gamemode_path = cd.as_path().join("src/gameplay/modes/");

    // eprintln!("dir: {:?}", gamemode_path);

    // sort dirs to ensure same order every time
    // resolves building for no reason (thanks fs)
    let mut files:Vec<DirEntry> = std::fs::read_dir(&gamemode_path)
        .unwrap()
        .filter_map(|file|file.ok())
        .collect();
    files.sort_by(|p1, p2| {
        p1.file_name().cmp(&p2.file_name())
    });

    let mut mods = vec![];
    let mut mode_list = Vec::new();
    let mut info_lines = Vec::new();

    for f in files {
        if !f.path().is_dir() {continue}

        let mode_folder = f.file_name().to_string_lossy().to_string();
        // eprintln!("adding gamemode {:?}", mode_folder);

        let mut config = GameModeInfo::default();
        let config_path = f.path().join(Path::new("./config.json"));
        if config_path.exists() {
            let f = std::fs::read(config_path).unwrap();
            let conf:Option<GameModeInfo> = serde_json::from_slice(f.as_slice()).ok();
            if let Some(conf) = conf {
                config = conf
            }
        }

        if let Some(true) = config.ignore { continue }


        // used for identification
        let internal_name = config.internal_name.unwrap_or(mode_folder.clone());

        // // used when the user will see the mode string
        // let display_name = config.display_name.unwrap_or(mode_folder.clone());

        mods.push(format!("mod {};", mode_folder));
        mode_list.push(format!("    \"{internal_name}\","));

        info_lines.push(
            format!("        map.insert(\"{internal_name}\".to_owned(), Box::new({mode_folder}::GameInfo::new()));")
        );
    }

    let mods = mods.join("\n");
    let mode_list = format!("\n{}\n", mode_list.join("\n"));
    let info_str = format!("\n{}", info_lines.join("\n"));


    let output_file = format!(r#"use crate::prelude::*;
{mods}

pub const AVAILABLE_PLAYMODES: &[&'static str] = &[{mode_list}];

lazy_static::lazy_static! {{
    pub static ref GAME_INFOS: Arc<HashMap<String, Box<dyn GameModeInfo + Send + Sync>>> = {{
        let mut map:HashMap<String, Box<dyn GameModeInfo + Send + Sync>> = HashMap::new();{info_str}
        Arc::new(map)
    }};
}}

pub fn get_gamemode_info(playmode: &String) -> Option<&Box<dyn GameModeInfo + Send + Sync>> {{
    GAME_INFOS.get(playmode)
}}

"#);


    let path = gamemode_path.join(Path::new("mod.rs"));

    // check if we should actually write the file
    if let Ok(file) = std::fs::read(&path) {
        if let Ok(str_file) = String::from_utf8(file) {
            if str_file == output_file {
                return
            }
        }
    }

    std::fs::write(path, output_file).unwrap();
}


#[allow(unused)]
#[derive(Clone, Debug, Deserialize, Default)]
struct GameModeInfo {
    // internal stuff

    /// name to use as identifier (ie osu, catch, taiko)
    internal_name: Option<String>,
    /// name to display to end user (ie Osu, Catch the Beat, Taiko)
    display_name: Option<String>,
    /// skip this gamemode? (helpful if mode is not ready to be shipped lol)
    ignore: Option<bool>,

    // meta about gamemode (to be implemented)

    /// about this gamemode
    about: Option<String>,

    /// who made this gamemode
    author: Option<String>,
    /// how to contact this author
    author_contact: Option<String>,
    /// where to report bugs
    bug_report_url: Option<String>,
}