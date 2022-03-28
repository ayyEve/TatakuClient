use crate::prelude::*;
mod mania;
mod osu;
mod taiko;
mod utyping;

pub const AVAILABLE_PLAYMODES: &[&'static str] = &[
        "mania",
        "osu",
        "taiko",
        "utyping",
];

pub fn manager_from_playmode(playmode: PlayMode, beatmap: &BeatmapMeta) -> TatakuResult<IngameManager> {
    let beatmap = Beatmap::from_metadata(beatmap)?;
    let gamemode:Box<dyn GameMode> = match &*beatmap.playmode(playmode) {
        "mania" => Box::new(mania::Game::new(&beatmap, false)?),
        "osu" => Box::new(osu::Game::new(&beatmap, false)?),
        "taiko" => Box::new(taiko::Game::new(&beatmap, false)?),
        "utyping" => Box::new(utyping::Game::new(&beatmap, false)?),
        _ => return Err(TatakuError::GameMode(GameModeError::UnknownGameMode))
    };

    Ok(IngameManager::new(beatmap, gamemode))
}

pub fn calc_acc(score: &Score) -> f64 {
    match &*score.playmode {
        "mania" => mania::calc_acc(score),
        "osu" => osu::calc_acc(score),
        "taiko" => taiko::calc_acc(score),
        "utyping" => utyping::calc_acc(score),
        _ => score.accuracy,
    }
    // if the number is nan,infinity, etc, replace it with 1.0 (100%)
    .normal_or(1.0)
}

pub fn calc_diff(map: &BeatmapMeta, mode_override: PlayMode, mods: &ModManager) -> TatakuResult<f32> {
    Ok(match &*map.check_mode_override(mode_override) {
        "mania" => mania::DiffCalc::new(map)?.calc(mods),
        "osu" => osu::DiffCalc::new(map)?.calc(mods),
        "taiko" => taiko::DiffCalc::new(map)?.calc(mods),
        "utyping" => utyping::DiffCalc::new(map)?.calc(mods),
        _ => Ok(0.0)
    }?
    // if the number is nan,infinity, etc, replace it with 0.0
    .normal_or(0.0))
}

pub fn gamemode_display_name(mode: &PlayMode) -> &'static str {
    match &**mode {
        "mania" => "Mania",
        "osu" => "Osu",
        "taiko" => "Taiko",
        "utyping" => "uTyping",
        _ => "Unknown"
    }
}

pub fn get_score_hit_string(mode: &PlayMode, score_hit: &ScoreHit) -> String {
    match &**mode {
        "mania" => mania::Game::score_hit_string(score_hit),
        "osu" => osu::Game::score_hit_string(score_hit),
        "taiko" => taiko::Game::score_hit_string(score_hit),
        "utyping" => utyping::Game::score_hit_string(score_hit),

        _ => String::new()
    }
}

// pub fn get_editor(playmode: &Playmode, beatmap: &Beatmap) -> TatakuResult<Box<dyn Menu>> {} // todo

