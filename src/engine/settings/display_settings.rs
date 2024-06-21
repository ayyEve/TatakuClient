use crate::prelude::*;


#[derive(Clone, Serialize, PartialEq)]
#[derive(Settings, SettingsDeserialize)]
#[serde(default)]
pub struct DisplaySettings {
    pub window_pos: [i32; 2],
    pub window_size: [f32; 2],
    #[Setting(text="FPS Limit", min=15, max=1_000, category="Window Settings")]
    pub fps_target: u64,
    #[Setting(text="Vsync", dropdown="Vsync")]
    #[serde(deserialize_with = "vsync_reader")]
    pub vsync: Vsync,
    #[Setting(text="Update Limit", min=500, max=10_000)]
    pub update_target: u64,
    
    /// should the game pause when focus is lost?
    #[Setting(text="Pause on Focus Loss")]
    pub pause_on_focus_lost: bool,
    #[Setting(text="Raw Mouse Input (requires restart)")]
    pub raw_mouse_input: bool,
    #[Setting(text="Scroll Sensitivity", min=0.1, max=5.0)]
    pub scroll_sensitivity: f32,

    #[Setting(text="Fullscreen", dropdown="FullscreenMonitor")]
    pub fullscreen_monitor: FullscreenMonitor,
    pub fullscreen_windowed: bool, // render at window_size?
    pub fullscreen_center: bool, // when rendering at window_size, center?

    
    #[Setting(text="Performance Mode (requires restart)", dropdown="PerformanceMode")]
    pub performance_mode: PerformanceMode,
    
    #[serde(skip)]
    #[Setting(text="Refresh Monitors", action="GameWindow::refresh_monitors()")]
    refresh_monitors_button: (),
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
            refresh_monitors_button: (),
        }
    }
}