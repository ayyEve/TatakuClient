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
    let mut build_gamemode_lines = vec![String::new()];
    let mut acc_calc_lines = vec![String::new()];
    let mut diff_calc_lines = vec![String::new()];
    let mut display_lines = vec![String::new()];
    let mut mode_list = Vec::new();
    let mut hit_judgment_list:Vec<String> = Vec::new();
    let mut perf_calc_lines = vec![String::new()];

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

        if let Some(true) = config.ignore {continue}


        // used for identification
        let internal_name = config.internal_name.unwrap_or(mode_folder.clone());

        // used when the user will see the mode string
        let display_name = config.display_name.unwrap_or(mode_folder.clone());


        // TODO: look for config file (?)
        mods.push(format!("mod {};", mode_folder));
        build_gamemode_lines.push(format!("        \"{internal_name}\" => Box::new({mode_folder}::Game::new(&beatmap, false).await?),"));
        acc_calc_lines.push(      format!("        \"{internal_name}\" => {mode_folder}::calc_acc(score),"));
        diff_calc_lines.push(     format!("        \"{internal_name}\" => Ok(Box::new({mode_folder}::DiffCalc::new(map).await?)),"));
        display_lines.push(       format!("        \"{internal_name}\" => \"{display_name}\","));
        mode_list.push(           format!("        \"{internal_name}\","));
        hit_judgment_list.push(   format!("        \"{internal_name}\" => Box::new({mode_folder}::DefaultHitJudgment),"));
        perf_calc_lines.push(   format!("        \"{internal_name}\" => {mode_folder}::Game::get_perf_calc(),"));
    }

    let mods = mods.join("\n");
    let gamemode_lines = build_gamemode_lines.join("\n");
    let acc_calc_lines = acc_calc_lines.join("\n");
    let diff_calc_lines = diff_calc_lines.join("\n");
    let display_lines = display_lines.join("\n");
    let mode_list = format!("\n{}\n", mode_list.join("\n"));
    let hit_judgment_lines = format!("\n{}\n", hit_judgment_list.join("\n"));
    let perf_calc_lines = format!("\n{}\n", perf_calc_lines.join("\n"));

    let output_file = format!(r#"use crate::prelude::*;
{mods}

pub const AVAILABLE_PLAYMODES: &[&'static str] = &[{mode_list}];

pub async fn manager_from_playmode(playmode: PlayMode, beatmap: &BeatmapMeta) -> TatakuResult<IngameManager> {{
    let beatmap = Beatmap::from_metadata(beatmap)?;
    let gamemode:Box<dyn GameMode> = match &*beatmap.playmode(playmode) {{{gamemode_lines}
        _ => return Err(TatakuError::GameMode(GameModeError::UnknownGameMode))
    }};

    Ok(IngameManager::new(beatmap, gamemode).await)
}}

pub async fn gamemode_from_playmode(playmode: PlayMode, beatmap: &BeatmapMeta) -> TatakuResult<Box<dyn GameMode>> {{
    let beatmap = Beatmap::from_metadata(beatmap)?;
    let gamemode:Box<dyn GameMode> = match &*beatmap.playmode(playmode) {{{gamemode_lines}
        _ => return Err(TatakuError::GameMode(GameModeError::UnknownGameMode))
    }};
    
    Ok(gamemode)
}}

pub async fn perfcalc_for_playmode(playmode: PlayMode) -> PerformanceCalc {{
    match &*playmode {{ {perf_calc_lines}
        _ => return NoMode::get_perf_calc()
    }}
}}

pub fn calc_acc(score: &Score) -> f64 {{
    match &*score.playmode {{{acc_calc_lines}
        _ => score.accuracy,
    }}
    // if the number is nan,infinity, etc, replace it with 1.0 (100%)
    .normal_or(1.0)
}}

pub async fn calc_diff(map: &BeatmapMeta, mode_override: PlayMode) -> TatakuResult<Box<dyn DiffCalc>> {{
    match &*map.check_mode_override(mode_override) {{{diff_calc_lines}
        _ => Err(TatakuError::Beatmap(BeatmapError::UnsupportedMode))
    }}
}}

pub fn gamemode_display_name(mode: &PlayMode) -> &'static str {{
    match &**mode {{{display_lines}
        _ => "Unknown"
    }}
}}

pub fn get_judgments(playmode:&PlayMode) -> Box<dyn crate::prelude::HitJudgments> {{
    match &**playmode {{{hit_judgment_lines}
        _ => Box::new(crate::prelude::DefaultHitJudgments::None)
    }}
}}

// pub fn get_editor(playmode: &Playmode, beatmap: &Beatmap) -> TatakuResult<Box<dyn Menu>> {{}} // TODO: this

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