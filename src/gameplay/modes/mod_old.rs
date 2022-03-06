use crate::prelude::*;

mod osu;
mod taiko;
mod catch;
mod mania;

mod gamemode_gen;


pub fn manager_from_playmode(playmode: PlayMode, beatmap: &BeatmapMeta) -> TatakuResult<IngameManager> {
    let beatmap = Beatmap::from_metadata(beatmap)?;
    let gamemode:Box<dyn GameMode> = match &*beatmap.playmode(playmode) {
        "osu"   => Box::new(osu::Game::new(&beatmap)?),
        "taiko" => Box::new(taiko::Game::new(&beatmap)?),
        "catch" => Box::new(catch::CatchGame::new(&beatmap)?),
        "mania" => Box::new(mania::ManiaGame::new(&beatmap)?),

        // TODO
        "adofai" | //=> Box::new(taiko::TaikoGame::new(&beatmap)?),
        "pTyping" => return Err(TatakuError::GameMode(GameModeError::NotImplemented)),

        _ => return Err(TatakuError::GameMode(GameModeError::UnknownGameMode))
    };

    Ok(IngameManager::new(beatmap, gamemode))
}

pub fn calc_acc(score: &Score) -> f64 {
    match &*score.playmode {
        "osu" => osu::calc_acc(score),
        "taiko" => taiko::calc_acc(score),
        "catch" => catch::calc_acc(score),
        "mania" => mania::calc_acc(score),

        _ => score.accuracy,
    }
    // if the number is nan,infinity, etc, replace it with 1.0 (100%)
    .normal_or(1.0)
}

pub fn calc_diff(map: &BeatmapMeta, mode_override: PlayMode, mods: &ModManager) -> f32 {
    match &*map.check_mode_override(mode_override) {
        "taiko" => taiko::DiffCalc::new(map).unwrap().calc(mods).unwrap_or_default(),
        
        _ => 0.0
    }
}

pub fn get_display_string(mode: PlayMode) -> &'static str {
    match &*mode {
        "taiko" => "Taiko",
        
        _ => ""
    }
}

