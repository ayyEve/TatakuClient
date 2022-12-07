use crate::prelude::*;
mod mania;
mod osu;
mod taiko;

pub const AVAILABLE_PLAYMODES: &[&'static str] = &[
    "mania",
    "osu",
    "taiko",
];

lazy_static::lazy_static! {
    pub static ref GAME_INFOS: Arc<HashMap<String, Box<dyn GameModeInfo + Send + Sync>>> = {
        let mut map:HashMap<String, Box<dyn GameModeInfo + Send + Sync>> = HashMap::new();
        map.insert("mania".to_owned(), Box::new(mania::GameInfo::new()));
        map.insert("osu".to_owned(), Box::new(osu::GameInfo::new()));
        map.insert("taiko".to_owned(), Box::new(taiko::GameInfo::new()));
        Arc::new(map)
    };
}

pub fn get_gamemode_info(playmode: &String) -> Option<&Box<dyn GameModeInfo + Send + Sync>> {
    GAME_INFOS.get(playmode)
}

