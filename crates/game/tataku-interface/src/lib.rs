#[cfg(feature="graphics")] mod misc;
#[cfg(feature="graphics")] mod dialog;
#[cfg(feature="graphics")] mod custom_menus;
#[cfg(feature="graphics")] mod menu_widgets;
#[cfg(feature="graphics")] mod visualizations;
#[cfg(feature="graphics")] mod gameplay_widgets;

#[cfg(feature="graphics")] 
pub mod prelude {
    pub(crate) use tataku_ui::prelude::*;
    pub(crate) use tataku_engine::prelude::*;
    pub(crate) use tataku_graphics::prelude::*;
    
    pub use crate::misc::*;
    pub use crate::dialog::*;
    pub use crate::custom_menus::*;
    pub use crate::menu_widgets::*;
    pub use crate::visualizations::*;
    pub use crate::gameplay_widgets::*;

    /// list of default gameplay widgets
    pub const DEFAULT_GAMEPLAY_WIDGETS: &[GameplayWidgetBuilder] = &[
        DURATION_BAR,
        ELAPSED,
        HEALTH_BAR,
        JUDGMENT_BAR,
        JUDGMENT_COUNTER,
        KEY_COUNTER,
        LEADERBOARD,
        SCORE,
        COMBO,
        ACCURACY,
        PERFORMANCE,
        SPECTATORS
    ];
}
