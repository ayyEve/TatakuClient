mod misc;
mod dialog;
mod custom_menus;
mod menu_widgets;
mod visualizations;
mod gameplay_widgets;

pub mod prelude {
    pub(crate) use tataku_engine::prelude::*;
    
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
