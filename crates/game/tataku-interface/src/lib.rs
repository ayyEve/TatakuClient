#[cfg(feature="graphics")] pub mod misc;
#[cfg(feature="graphics")] pub mod dialog;
#[cfg(feature="graphics")] pub mod custom_menus;
#[cfg(feature="graphics")] pub mod menu_widgets;
#[cfg(feature="graphics")] pub mod visualizations;
#[cfg(feature="graphics")] pub mod gameplay_widgets;

#[cfg(feature="graphics")] 
pub mod prelude {
    pub(crate) use tataku_engine::{
        ChainableInitializer,
        tracing::*,
        Default2,
        Debug2,
        From,

        Serialize,
        Deserialize,
    };

    pub(crate) use tataku_engine as engine;
    pub(crate) use engine::{ input, actions };

    pub(crate) use tataku_ui as ui;
    pub(crate) use tataku_graphics as graphics;
    pub(crate) use crate::menu_widgets as widgets;
    pub(crate) use tataku_engine_common::common::*;

    
    pub use crate::misc::*;
    pub use crate::dialog::*;
    pub use crate::custom_menus::*;
    pub use crate::visualizations::*;

    /// list of default gameplay widgets
    pub const DEFAULT_GAMEPLAY_WIDGETS: &[engine::gameplay::widgets::GameplayWidgetBuilder] = &[
        crate::gameplay_widgets::DURATION_BAR,
        crate::gameplay_widgets::ELAPSED,
        crate::gameplay_widgets::HEALTH_BAR,
        crate::gameplay_widgets::JUDGMENT_BAR,
        crate::gameplay_widgets::JUDGMENT_COUNTER,
        crate::gameplay_widgets::KEY_COUNTER,
        crate::gameplay_widgets::LEADERBOARD,
        crate::gameplay_widgets::SCORE,
        crate::gameplay_widgets::COMBO,
        crate::gameplay_widgets::ACCURACY,
        crate::gameplay_widgets::PERFORMANCE,
        crate::gameplay_widgets::SPECTATORS
    ];
}
