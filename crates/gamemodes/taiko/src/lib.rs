mod game;
mod info;
mod notes;
mod helpers;
mod settings;
mod diff_calc;
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

    pub use super::info::*;
    pub use super::game::*;
    pub use super::notes::*;
    pub use super::helpers::*;
    pub use super::settings::*;
    pub use super::diff_calc::*;
    #[cfg(feature="graphics")] pub use super::don_chan::*;
}
