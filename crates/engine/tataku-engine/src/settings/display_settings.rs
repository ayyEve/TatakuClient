use crate::prelude::*;

#[derive(Clone, Serialize, PartialEq, Debug2)]
#[cfg_attr(feature="graphics", derive(Settings))]
#[derive(Reflect, SettingsDeserialize)]
#[allow(clippy::manual_non_exhaustive)]
#[serde(default)]
pub struct DisplaySettings {
    pub window_pos: [i32; 2],
    pub window_size: [f32; 2],

    #[setting(text="FPS Limit", range(15.0, 1_000.0))]
    pub fps_target: u64,
    #[dropdown(text="Vsync", path="enums.vsync")]
    #[serde(deserialize_with = "vsync_reader")]
    pub vsync: Vsync,
    #[setting(text="Update Limit", range(500.0, 10_000.0))]
    pub update_target: u64,
    
    /// should the game pause when focus is lost?
    #[setting(text="Pause on Focus Loss")]
    pub pause_on_focus_lost: bool,
    #[setting(text="Raw Mouse Input (requires restart)")]
    pub raw_mouse_input: bool,
    #[setting(text="Scroll Sensitivity", range(0.1, 5.0))]
    pub scroll_sensitivity: f32,

    #[dropdown(text="Fullscreen", path="enums.monitors")]
    pub fullscreen_monitor: FullscreenMonitor,
    pub fullscreen_windowed: bool, // render at window_size?
    pub fullscreen_center: bool, // when rendering at window_size, center?

    
    #[dropdown(text="Performance Mode (requires restart)", path="enums.performance_mode")]
    pub performance_mode: PerformanceMode,
    
    #[serde(skip)] #[reflect(skip)] #[debug(skip)] 
    #[button(text="Refresh Monitors", action="WindowAction::RefreshMonitors")] _refresh_monitors: (),

    #[setting(text="Hide Decorations")]
    pub hide_decorations: bool,

    #[setting(text="Blur Enabled")]
    pub enable_blur: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            // window settings
            pause_on_focus_lost: true,
            fps_target: 144,
            update_target: 10_000,
            vsync: Vsync::default(),
            window_pos: [0, 0],
            window_size: [1280.0, 720.0],
            performance_mode: PerformanceMode::HighPerformance,
            
            raw_mouse_input: false,
            scroll_sensitivity: 1.0,
            
            fullscreen_monitor: FullscreenMonitor::None,
            fullscreen_windowed: false,
            fullscreen_center: true,
            _refresh_monitors: (),

            hide_decorations: false,
            enable_blur: true,
        }
    }
}


#[derive(Copy, Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[derive(Reflect)]
#[reflect(display = "display")]
pub enum PerformanceMode {
    PowerSaver,
    HighPerformance,
}
impl PerformanceMode {
    pub fn list() -> Vec<Self> {
        vec![
            Self::PowerSaver,
            Self::HighPerformance,
        ]
    }
}
impl Display for PerformanceMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
