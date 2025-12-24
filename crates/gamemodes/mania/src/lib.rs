mod game;
mod info;
mod notes;
mod settings;
mod velocity;
mod playfield;
mod auto_replay;
mod diff_calc;

pub use info::GAME_INFO;

mod prelude {
    pub use engine::input;
    pub use tataku_engine as engine;
    pub use tataku_engine_common::common::*;
    #[cfg(feature="graphics")] 
    pub use tataku_engine::graphics;

    pub use tataku_graphics::TatakuRenderable;

    pub use super::game::*;
    pub use super::info::*;
    pub use super::notes::*;
    pub use super::velocity::*;
    pub use super::playfield::*;
    pub use super::auto_replay::*;
    pub use super::settings::*;
    pub use super::diff_calc::*;
}
