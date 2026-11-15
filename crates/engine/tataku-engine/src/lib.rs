#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(clippy::module_inception)]
#![allow(clippy::new_without_default)]

pub mod io;
pub mod game;
pub mod data;
pub mod online;
pub mod window;
pub mod settings;
pub mod integration_event;

// folders
pub const DOWNLOADS_DIR:&str = "downloads";
pub const SONGS_DIR:&str = "songs";
pub const REPLAYS_DIR:&str = "replays";
pub const SKINS_FOLDER:&str = "skins";
pub const REPLAY_EXPORTS_DIR:&str = "../replays";



// std imports
pub(crate) use std::ops::Range;
pub(crate) use std::collections::VecDeque;

// async trait
pub use async_trait::async_trait;

// triple buffer imports
#[cfg(feature = "ui")]
pub use triple_buffer;

// logging
pub use tracing::*;

// tataku exports
pub use crate::exports::*;


// shunting yard re-exports

// /// engine::shunting_yards
// pub mod shunting_yards {
//     pub use shunting_yard::GenericShuntingYard;
//     pub use path_shunting_yard::VariablePathResolver; // engine::VariablePathResolver
//     pub use buildable_shunting_yard::BuildableShuntingYard;
// }
pub use buildable_shunting_yard::BuildableCalc;
pub use path_shunting_yard::VariablePathResolver;

// manual re-exports
pub use crate::{
    settings::Settings, // engine::Settings
    integration_event::TatakuIntegrationEvent, // engine::TatakuIntegrationEvent

    io::{
        Downloadable, // engine::Downloadable
        AsyncLoader, // engine::AsyncLoader
    }, 

    online::{
        online_content, // engine::online_content
    },

    data::{
        database, // engine::database
    },

    game::{
        task, // engine::task
        actions, // engine::actions
        beatmaps, // engine::beatmaps
        gameplay, // engine::gameplay
        notifications, // engine::notifications
        notifications::Notification, // engine::Notification
        beatmaps::{
            BeatmapMeta, // engine::BeatmapMeta
        },
        task::{
            TatakuTask as Task, // engine::Task
        }, 
    },
};

#[cfg(feature="graphics")]
pub use crate::game::beatmap_animation::BeatmapAnimation; // engine::BeatmapAnimation

pub mod exports {
    pub use crate as engine;
    pub use tataku_input as input;
    pub use tataku_common as common;
    pub use tataku_engine_common::errors;
    pub use tataku_engine_common::common::*;
    pub use tataku_engine_common::prelude as tataku;
    
    #[cfg(feature="graphics")] pub use tataku_ui as ui;
    #[cfg(feature="graphics")] pub use tataku_graphics as graphics;
}
