#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(clippy::module_inception)]
#![allow(clippy::new_without_default)]

mod io;
mod game;
mod data;
mod audio;
mod online;
mod window;
mod locale;
#[cfg(feature="graphics")]
mod graphics;
mod settings;
mod tataku_event;
mod tataku_integration_event;
pub mod prelude;


/// how tall is the duration bar
pub const DURATION_HEIGHT:f32 = 35.0;

// folders
pub const DOWNLOADS_DIR:&str = "downloads";
pub const SONGS_DIR:&str = "songs";
pub const REPLAYS_DIR:&str = "replays";
pub const SKINS_FOLDER:&str = "skins";
pub const REPLAY_EXPORTS_DIR:&str = "../replays";
