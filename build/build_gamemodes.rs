use std::fs::DirEntry;

const GAMEMODE_FOLDER: &str = "src/tataku/gameplay/modes";

pub fn build_gamemodes() {
    // get all gamemodes in the folder
    let cd = std::env::current_dir().unwrap();
    let gamemode_path = cd.as_path().join(GAMEMODE_FOLDER);

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
        if !f.path().is_dir() { continue }

        let mode_folder = f.file_name().to_string_lossy().to_string();
        // eprintln!("adding gamemode {:?}", mode_folder);

        if mode_folder.starts_with("_") { continue }

        mods.push(format!("mod {};", mode_folder));
        mode_list.push(format!("    \"{mode_folder}\","));

        info_lines.push(
            format!("        map.insert(\"{mode_folder}\".to_owned(), Box::new({mode_folder}::GameInfo::new()));")
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


    let path = gamemode_path.join("mod.rs");

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
