#![cfg_attr(not(feature="graphics"), allow(unused))]
#[macro_use] extern crate log;
pub mod engine;
pub mod tataku;
pub mod prelude;
pub mod interface;

// folders
pub const DOWNLOADS_DIR:&str = "downloads";
pub const SONGS_DIR:&str = "songs";
pub const REPLAYS_DIR:&str = "replays";
pub const SKINS_FOLDER:&str = "skins";
pub const REPLAY_EXPORTS_DIR:&str = "../replays";
