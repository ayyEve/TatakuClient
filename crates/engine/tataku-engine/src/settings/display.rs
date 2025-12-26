use crate::*;
use common::reflect::*;
use tataku_client_proc_macros::Settings;

#[derive(Reflect, Settings)]
#[allow(clippy::manual_non_exhaustive)]
#[derive(Serialize, DeserializeSettings)]
#[derive(Clone, Debug2, Default2, PartialEq)]
#[serde(default)]
pub struct DisplaySettings {
    pub window_pos: [i32; 2],
    
    #[default([1280.0, 720.0])]
    pub window_size: [f32; 2],

    #[default(144)]
    #[setting(text="FPS Limit", range(15.0, 1_000.0))]
    pub fps_target: u64,

    #[serde(deserialize_with = "vsync")]
    #[dropdown(text="Vsync", path="enums.vsync")]
    pub vsync: tataku::Vsync,
    
    #[default(10_000)]
    #[setting(text="Update Limit", range(500.0, 10_000.0))]
    pub update_target: u64,
    
    /// should the game pause when focus is lost?
    #[default(true)]
    #[setting(text="Pause on Focus Loss")]
    pub pause_on_focus_lost: bool,

    #[setting(text="Raw Mouse Input (requires restart)")]
    pub raw_mouse_input: bool,

    #[default(1.0)]
    #[setting(text="Scroll Sensitivity", range(0.1, 5.0))]
    pub scroll_sensitivity: f32,

    #[dropdown(text="Fullscreen", path="enums.monitors")]
    pub fullscreen_monitor: window::FullscreenMonitor,

    /// render at window_size?
    pub fullscreen_windowed: bool, 
    
    /// when rendering at window_size, center?
    #[default(true)]
    pub fullscreen_center: bool,

    
    #[dropdown(text="Performance Mode (requires restart)", path="enums.performance_mode")]
    pub performance_mode: PerformanceMode,
    
    #[serde(skip)] #[reflect(skip)] #[debug(skip)] 
    #[button(text="Refresh Monitors", action="actions::window::WindowAction::RefreshMonitors")]
    _refresh_monitors: (),

    #[setting(text="Hide Decorations")]
    pub hide_decorations: bool,

    #[default(true)]
    #[setting(text="Blur Enabled")]
    pub enable_blur: bool,
}

#[derive(Reflect)]
#[reflect(display = "display")]
#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum PerformanceMode {
    PowerSaver,
    #[default] HighPerformance,
}
impl PerformanceMode {
    pub fn list() -> Vec<Self> {
        vec![
            Self::PowerSaver,
            Self::HighPerformance,
        ]
    }
}
impl std::fmt::Display for PerformanceMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
