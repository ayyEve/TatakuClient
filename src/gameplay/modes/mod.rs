use crate::prelude::*;
mod mania;
mod osu;
mod catch;
mod taiko;

pub fn manager_from_playmode(playmode: PlayMode, beatmap: &BeatmapMeta) -> TatakuResult<IngameManager> {
    let beatmap = Beatmap::from_metadata(beatmap)?;
    let gamemode:Box<dyn GameMode> = match &*beatmap.playmode(playmode) {
    "mania" => Box::new(mania::Game::new(&beatmap)?),
    "osu" => Box::new(osu::Game::new(&beatmap)?),
    "catch" => Box::new(catch::Game::new(&beatmap)?),
    "taiko" => Box::new(taiko::Game::new(&beatmap)?),
        _ => return Err(TatakuError::GameMode(GameModeError::UnknownGameMode))
    };

    Ok(IngameManager::new(beatmap, gamemode))
}

pub fn calc_acc(score: &Score) -> f64 {
    match &*score.playmode {
    "mania" => mania::calc_acc(score),
    "osu" => osu::calc_acc(score),
    "catch" => catch::calc_acc(score),
    "taiko" => taiko::calc_acc(score),
        _ => score.accuracy,
    }
    // if the number is nan,infinity, etc, replace it with 1.0 (100%)
    .normal_or(1.0)
}

pub fn calc_diff(map: &BeatmapMeta, mode_override: PlayMode, mods: &ModManager) -> TatakuResult<f32> {
    match &*map.check_mode_override(mode_override) {
    "mania" => mania::DiffCalc::new(map)?.calc(mods),
    "osu" => osu::DiffCalc::new(map)?.calc(mods),
    "catch" => catch::DiffCalc::new(map)?.calc(mods),
    "taiko" => taiko::DiffCalc::new(map)?.calc(mods),
        _ => Ok(0.0)
    }
}

pub fn gamemode_display_name(mode: PlayMode) -> &'static str {
    match &*mode {
    "mania" => "Mania",
    "osu" => "Osu",
    "catch" => "Catch",
    "taiko" => "Taiko",
        _ => ""
    }
}
    