mod hit;
mod game;
mod info;
mod notes;
mod settings;
mod diff_calc;
mod playfield;
mod note_queue;
mod auto_replay;
mod battery_health;
mod full_alt_counter;
#[cfg(feature="graphics")] mod don_chan;

pub use info::GAME_INFO;


/// external for when we build as a dynamic gamemode
mod external {
    use tataku_engine::gameplay::info::external;
    const EXTERN:external::GamemodeInfo = crate::GAME_INFO.as_extern();

    #[unsafe(no_mangle)]
    pub extern "C" fn version() -> u8 { 
        1
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn game_info() -> &'static external::GamemodeInfo {
        &EXTERN
    }
}


mod prelude {
    pub use tataku_engine as engine;
    pub use tataku_engine_common::common::*;

    pub use tataku_graphics::TatakuRenderable;

    pub use super::hit::*;
    pub use super::game::*;
    pub use super::info::*;
    pub use super::notes::*;
    pub use super::settings::*;
    pub use super::diff_calc::*;
    pub use super::playfield::*;
    pub use super::note_queue::*;
    pub use super::auto_replay::*;
    pub use super::battery_health::*;
    pub use super::full_alt_counter::*;
    #[cfg(feature="graphics")] pub use super::don_chan::*;
}
