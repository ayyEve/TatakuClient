#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(clippy::module_inception)]
#![allow(clippy::new_without_default)]


mod io;
mod game;
mod data;
mod audio;
mod input;
mod online;
mod window;
mod locale;
mod graphics;
mod settings;
mod interface;
mod databases;
mod tataku_event;
pub mod prelude;


/// how tall is the duration bar
pub const DURATION_HEIGHT:f32 = 35.0;

// folders
pub const DOWNLOADS_DIR:&str = "downloads";
pub const SONGS_DIR:&str = "songs";
pub const REPLAYS_DIR:&str = "replays";
pub const SKINS_FOLDER:&str = "skins";
pub const REPLAY_EXPORTS_DIR:&str = "../replays";

/// format a number into a locale string ie 1000000 -> 1,000,000
pub fn format_number(num: impl num_format::ToFormattedStr) -> String {
    use num_format::{Buffer, Locale};
    let mut buf = Buffer::default();
    buf.write_formatted(&num, &Locale::en);

    buf.as_str().to_owned()
}

/// format a float into a locale string ie 1000.1 -> 1,000.100
pub fn format_float(num: impl ToString, precis: usize) -> String {
    let num = num.to_string();
    let mut split = num.split(".");
    let Some(num) = split.next().and_then(|a|a.parse::<i64>().ok()).map(format_number) else { return String::new() };

    let Some(dec) = split.next() else {
        return format!("{num}.{}", "0".repeat(precis));
    };

    let dec = if dec.len() > precis {
        dec.split_at(precis).0.to_owned()
    } else {
        format!("{dec:0precis$}")
    };

    format!("{num}.{dec}")
}


use crate::prelude::*;
pub fn visibility_bg(pos:Vector2, size:Vector2) -> impl TatakuRenderable {
    Rectangle::new(
        pos,
        size,
        Color::WHITE.alpha(0.6),
        None
    )
}
