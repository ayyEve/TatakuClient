#![allow(unused)]
use crate::prelude::*;


pub async fn load_osu_skins(path: impl AsRef<Path>) {
    let mut path = path.as_ref();
    if path.is_file() {
        path = path.parent().unwrap();
    }

    let skins_folder = path.join("Skins");
    if skins_folder.exists() {
        if let Ok(skins_folder) = skins_folder.read_dir() {
            for f in skins_folder.filter_map(|f|f.ok()) {
                if f.path().is_dir() {
                    let mut options = fs_extra::dir::CopyOptions::new();
                    options.copy_inside = true;
                    if let Err(e) = fs_extra::dir::copy(f.path(), SKINS_FOLDER, &options) {
                        // NotificationManager::add_error_notification("error copying skin", TatakuError::from_err(e)).await;
                    }
                }
            }
        }
    }
}

pub async fn load_osu_settings(path: impl AsRef<Path>, settings: &mut Settings) -> Result<(), TatakuError> {
    let path = path.as_ref();
    let data = Io::read_lines_resolved(path)?
        .flat_map(|i| {
            let mut s = i.split("=");
            s
                .next()
                .zip(s.next())
                .map(|(key, val)| (key.to_owned(), val.to_owned()))
        })
        .collect::<HashMap<String, String>>();


    // read bool from map
    macro_rules! bool {
        ($key:expr) => {
            data.get(&$key.to_owned()) == Some("1")
        };
        ($key:expr, $val:ident) => {
            if let Some(v) = data.get(&$key.to_owned()) {
                settings.$val = v == "1"
            }
        };
        ($key:expr, $val:ident, $sub:ident) => {
            if let Some(v) = data.get(&$key.to_owned()) {
                settings.$val.$sub = v == "1"
            }
        }
    }
    // read bool from map, but true when val is false
    macro_rules! bool_rev {
        ($key:expr) => {
            data.get(&$key.to_owned()) == Some("0")
        };
        ($key:expr, $val:ident) => {
            if let Some(v) = data.get(&$key.to_owned()) {
                settings.$val = v == "0"
            }
        };
        ($key:expr, $val:ident,$sub:ident) => {
            if let Some(v) = data.get(&$key.to_owned()) {
                settings.$val.$sub = v == "0"
            }
        }
    }
    // read f32
    macro_rules! num {
        ($key:expr, $n:tt) => {
            if let Some(Ok(v)) = data.get(&$key.to_owned()).map(|v|v.parse::<$n>()) {
                Some(v)
            } else {
                None
            }
        };
        ($key:expr, $val:ident, $n:tt) => {
            if let Some(Ok(v)) = data.get(&$key.to_owned()).map(|v|v.parse::<$n>()) {
                settings.$val = v
            }
        }
    }
    // read f32 when the number is 0-100
    macro_rules! num_big {
        ($key:expr, $val:ident, $n:tt) => {
            if let Some(Ok(v)) = data.get(&$key.to_owned()).map(|v|v.parse::<$n>()) {
                settings.$val = v / 100.0
            }
        }
    }

    // string
    macro_rules! string {
        ($key:expr) => {
            data.get(&$key.to_owned())
        };

        ($key:expr, $val:ident) => {
            if let Some(v) = data.get(&$key.to_owned()) {
                settings.$val = v.to_owned()
            }
        }
    }
    // string
    macro_rules! string_lower {
        ($key:expr, $val:ident) => {
            if let Some(v) = data.get(&$key.to_owned()) {
                settings.$val = v.to_lowercase()
            }
        }
    }
    

    num_big!("VolumeUniversal", master_vol, f32);
    num_big!("VolumeEffect", effect_vol, f32);
    num_big!("VolumeMusic", music_vol, f32);
    bool!("CursorRipple", cursor_settings, cursor_ripples);
    bool_rev!("IgnoreBeatmapSamples", beatmap_hitsounds);
    // bool_rev!("IgnoreBeatmapSkins", beatmap_skins);
    string_lower!("LastPlayMode", last_played_mode);
    bool!("MouseDisableButtons", osu_settings, ignore_mouse_buttons);
    num!("Offset", global_offset, f32);
    if let Some((width, height)) = num!("Width", f32).zip(num!("Height", f32)) {
        settings.display_settings.window_size = [width, height] 
    }

    // num!("CustomFrameLimit", display_settings.fps_target, u64);
    string!("Skin", current_skin);
    // bool!("RawInput", display_settings.raw_mouse_input);
    // bool!("ComboColourSliderBall", standard_settings, combo_color_slider);
    string!("Username", osu_username);
    // bool!("DiscordRichPresence", discord);

    // keys
    macro_rules! key {
        ($key:expr, $val:ident) => {
            if let Some(v) = data.get(&$key.to_owned()) {
                if let Some(k) = parse_key(&v) {
                    settings.$val = k
                }
            }
        };
        ($key:expr, $val:ident, $sub:ident) => {
            if let Some(v) = data.get(&$key.to_owned()) {
                if let Some(k) = parse_key(&v) {
                    settings.$val.$sub = k
                }
            }
        }
    }
    
    key!("keyOsuLeft", osu_settings, left_key);
    key!("keyOsuRight", osu_settings, right_key);
    // key!("keyOsuSmoke", standard_settings, smoke_key);

    key!("keyTaikoOuterLeft", taiko_settings, left_kat);
    key!("keyTaikoInnerLeft", taiko_settings, left_don);
    key!("keyTaikoInnerRight", taiko_settings, right_don);
    key!("keyTaikoOuterRight", taiko_settings, right_kat);

    // key!("keyPause", pause_key);
    // key!("keySkip", skip_key);
    // key!("keyToggleScoreboard", scoreboard_key);
    // key!("keyToggleChat", chat_key);
    // key!("keyScreenshot", screenshot_key);
    // key!("keyIncreaseAudioOffset", offset_increase_key);
    // key!("keyDecreaseAudioOffset", offset_decrease_key);
    // key!("keyQuickRetry", retry);

    // key!("keyIncreaseSpeed", mania_settings, sv_increase_key);
    // key!("keyDecreaseSpeed", mania_settings, sv_decrease_key);

    
    // key!("keyVolumeIncrease", volume_up_key);
    // key!("keyVolumeDecrease", volume_down_key);

    for c in 1..=10 { 
        if let Some(k_str) = string!(format!("ManiaLayouts{}K", c)) {
            let mut keys = Vec::new();
            for s in k_str.split(" "){
                if let Some(k) = parse_key(&s.to_owned()) {
                    keys.push(k);
                }
            }

            if keys.len() == c {
                settings.mania_settings.keys[c-1] = keys
            }
        }
        
    }


    let songs_folder = data.get("BeatmapDirectory").cloned().unwrap_or("Songs".to_owned());
    let songs_folder = path.parent().unwrap().join(&songs_folder).to_string_lossy().to_string();
    if !settings.external_games_folders.contains(&songs_folder) {
        settings.external_games_folders.push(songs_folder);
    }

    Ok(())
}


fn parse_key(k: &String) -> Option<Key> {
    use crate::input::Key::*;

    match &**k {
        "LeftShift" => Some(LShift),
        "RightShift" => Some(RShift),
        "OemSemicolon" => Some(Semicolon),
        // "OemTilde" => Some(Key::),

        other => serde_json::from_str(&format!("{{\"k\":\"{other}\"}}")).ok(),
    }
}

#[derive(Deserialize)]
struct K { k: Key }