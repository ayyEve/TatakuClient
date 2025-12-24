mod math;
mod game;
mod info;
mod notes;
mod coords;
mod settings;
mod relax_play;
mod auto_replay;
mod diff_calc;
#[cfg(feature="graphics")] mod cursor;

pub use info::GAME_INFO;

/// import helper
mod prelude {
    pub use tataku_engine::input;
    pub use tataku_engine as engine;
    pub use tataku_engine_common::common::*;

    pub use tataku_graphics::TatakuRenderable;

    pub use super::math::*;
    pub use super::game::*;
    pub use super::info::*;
    pub use super::notes::*;
    pub use super::coords::*;
    pub use super::settings::*;
    pub use super::relax_play::*;
    pub use super::auto_replay::*;
    #[cfg(feature="graphics")] pub use super::cursor::*;
    pub use super::diff_calc::OsuDifficultyCalculator;
}
