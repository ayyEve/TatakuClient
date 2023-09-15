use crate::prelude::*;
use tataku_client_proc_macros::Settings;

const SETTINGS_FILE:&str = "settings.json";

#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[derive(Settings)]
#[serde(default)]
pub struct Settings {
    // audio
    // #[Setting(text="Master Volume", category="Audio Settings")]
    pub master_vol: f32,
    // #[Setting(text="Music Volume")]
    pub music_vol: f32,
    // #[Setting(text="Effect Volume")]
    pub effect_vol: f32,
    #[Setting(text="Global Offset", min=-100.0, max=100.0, category="Audio Settings")]
    pub global_offset: f32,
    
    // login
    #[Setting(text="Tataku Username", category="Tataku Server Settings")]
    pub username: String,
    #[Setting(text="Tataku Password", password=true)]
    pub password: String,
    #[Setting(text="Tataku Server Url")]
    pub server_url: String,
    #[Setting(text="Tataku Score Url")]
    pub score_url: String,
    
    // osu login (for direct)
    #[Setting(text="Osu Username", category="Osu Integration")]
    pub osu_username: String,
    #[Setting(text="Osu Password", password=true)]
    pub osu_password: String,
    #[Setting(text="Osu Api Key", password=true)]
    pub osu_api_key: String,
    
    // game settings
    #[serde(alias="standard_settings")]
    #[Subsetting(category="Osu Settings")]
    pub osu_settings: OsuSettings,
    #[Subsetting(category="Taiko Settings")]
    pub taiko_settings: TaikoSettings,
    // #[Subsetting(category="Catch Settings")]
    pub catch_settings: CatchSettings,
    #[Subsetting(category="Mania Settings")]
    pub mania_settings: ManiaSettings,
    #[Subsetting(category="Background Game Settings")]
    pub background_game_settings: BackgroundGameSettings,
    #[Subsetting(category="Common Game Settings")]
    pub common_game_settings: CommonGameplaySettings,

    pub last_played_mode: String,
    pub last_score_retreival_method: ScoreRetreivalMethod,
    pub last_sort_by: SortBy,
    
    #[Setting(text="Beatmap Hitsounds")]
    pub beatmap_hitsounds: bool,

    #[Setting(text="Enable Difficulty Calculation")]
    pub enable_diffcalc: bool,

    // window settings
    pub window_pos: [i32; 2],
    pub window_size: [f32; 2],
    #[Setting(text="FPS Limit", min=15, max=240, category="Window Settings")]
    pub fps_target: u64,
    #[Setting(text="Vsync")]
    pub vsync: bool,
    #[Setting(text="Update Limit", min=500, max=10_000)]
    pub update_target: u64,
    
    // #[Setting(text="UI Scale")] // not ready yet
    pub ui_scale: f64,
    #[Setting(text="Background Dim", min=0, max=1)]
    pub background_dim: f32,
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
    #[Setting(text="Refresh Monitors", action="|_|GameWindow::refresh_monitors()")]
    refresh_monitors_button: (),

    // cursor
    #[Setting(text="Cursor Color", category="Cursor Settings")]
    pub cursor_color: Color,
    #[Setting(text="Cursor Scale", min=0.1, max=10.0)]
    pub cursor_scale: f32,
    #[Setting(text="Cursor Border", min=0.1, max=5.0)]
    pub cursor_border: f32,
    #[Setting(text="Cursor Border Color")]
    pub cursor_border_color: Color,

    #[Setting(text="Cursor Ripples")]
    pub cursor_ripples: bool,
    #[Setting(text="Cursor Ripple Color")]
    pub cursor_ripple_color: Color,
    #[Setting(text="Cursor Ripple Scale")]
    pub cursor_ripple_final_scale: f32,

    // skin settings
    #[Setting(text="Skin", dropdown="SkinDropdownable", dropdown_value="Skin", category="Skin Settings")]
    pub current_skin: String,

    #[serde(skip)]
    #[Setting(text="Refresh Skins", action="|_|SkinManager::refresh_skins()")]
    refresh_skins_button: (),

    #[Setting(text="Theme", dropdown="SelectedTheme")]
    pub theme: SelectedTheme,


    // misc keybinds
    #[Setting(text="User Panel Key", category="Common Keybinds")]
    pub key_user_panel: Key,

    // double tap protection
    #[Setting(text="Enable DoubleTap Protection", category="DoubleTap Protection")]
    pub enable_double_tap_protection: bool,
    #[Setting(text="DoubleTap Protection Leniency", min=10.0, max=200.0)]
    pub double_tap_protection_duration: f32,


    // integrations
    #[Subsetting(category="Integrations")]
    pub integrations: IntegrationSettings,


    // other misc
    pub last_git_hash: String,
    pub external_games_folders: Vec<String>,
    
    #[Subsetting(category="Log Settings")]
    pub logging_settings: LoggingSettings,

    #[serde(skip)]
    pub skip_autosaveing: bool,
}
impl Settings {
    pub fn get() -> Arc<Self> {
        GlobalValueManager::get::<Settings>().unwrap()
    }
    pub fn get_mut() -> GlobalValueMut<Self> {
        GlobalValueManager::get_mut::<Settings>().unwrap()
    }


    pub async fn load() -> Self {
        let mut s = match std::fs::read_to_string(SETTINGS_FILE).map(|s| serde_json::from_str(&s).map_err(|e|e.to_string())).map_err(|e|e.to_string()) {
            Ok(Ok(settings)) => settings,
            Err(e) | Ok(Err(e)) => {
                // NotificationManager::add_error_notification("Error reading settings.json\nLoading defaults", e).await;
                warn!("Error reading settings.json\nLoading defaults, {e}");
                if let Some(saved_as) = Self::backup_settings().await {
                    info!("Old settings saved t {saved_as}");
                }
                Self::default()
            }
        };

        // check password hashes
        s.check_hashes();
        
        GlobalValueManager::update(Arc::new(s.clone()));
        GlobalValueManager::update(Arc::new(WindowSize(s.window_size.into())));

        // save after loading.
        // writes file if it doesnt exist, and writes new values from updates
        s.save().await;
        s
    }
    pub async fn save(&self) {
        debug!("Saving settings");
        let str = serde_json::to_string_pretty(self).unwrap();
        match std::fs::write(SETTINGS_FILE, str) {
            Ok(_) => trace!("settings saved successfully"),
            Err(e) => NotificationManager::add_error_notification("Error saving settings", e).await,
        }
    }

    pub fn get_effect_vol(&self) -> f32 {self.effect_vol * self.master_vol}
    pub fn get_music_vol(&self) -> f32 {self.music_vol * self.master_vol}

    pub fn check_hashes(&mut self) {
        if self.osu_password.len() > 0 {self.osu_password = check_md5(self.osu_password.clone())}
        if self.password.len() > 0 {self.password = check_sha512(self.password.clone())}
    }

    // make a backup of the setting before they're overwritten (when the file fails to load)
    async fn backup_settings() -> Option<String> {
        if Io::exists(SETTINGS_FILE) {
            let mut counter = 0;
            let mut file = format!("{SETTINGS_FILE}.bak_{counter}");
            while Io::exists(&file) {
                counter += 1;
                file = format!("{SETTINGS_FILE}.bak_{counter}")
            }
            std::fs::copy(SETTINGS_FILE, &file).expect("An error occurred while backing up the old settings.json");
            // if let Err(e) = std::fs::copy(SETTINGS_FILE, &file) {
            //     NotificationManager::add_error_notification("Error backing up settings.json", e).await
            // } else {
            //     NotificationManager::add_text_notification(
            //         &format!("Backup saved as {file}"),
            //         5000.0,
            //         Color::YELLOW
            //     ).await;
            // }

            Some(file)
        } else {
            None
        }
    }

}
impl Default for Settings {
    fn default() -> Self {
        Self {
            // audio
            music_vol: 0.5,
            effect_vol: 0.5,
            master_vol: 0.3,
            global_offset: 0.0,

            // login
            username: "Guest".to_owned(),
            password: String::new(),
            server_url: "wss://server.tataku.ca".to_owned(),
            score_url: "https://scores.tataku.ca".to_owned(),

            // osu
            osu_username: String::new(),
            osu_password: String::new(),
            osu_api_key: String::new(),

            // game settings
            logging_settings: LoggingSettings::new(),
            osu_settings: OsuSettings::default(),
            taiko_settings: TaikoSettings::default(),
            catch_settings: CatchSettings::default(),
            mania_settings: ManiaSettings::default(),
            background_game_settings: BackgroundGameSettings::default(),
            common_game_settings: CommonGameplaySettings::default(),
            pause_on_focus_lost: true,
            last_played_mode: "osu".to_owned(),
            last_score_retreival_method: ScoreRetreivalMethod::Local,
            last_sort_by: SortBy::Title,
            beatmap_hitsounds: true,
            enable_diffcalc: true,

            // window settings
            fps_target: 144,
            update_target: 10_000,
            vsync: false,
            window_pos: [0, 0],
            window_size: [1280.0, 720.0],
            performance_mode: PerformanceMode::HighPerformance,
            
            ui_scale: 1.0,
            background_dim: 0.8,
            raw_mouse_input: false,
            scroll_sensitivity: 1.0,
            
            fullscreen_monitor: FullscreenMonitor::None,
            fullscreen_windowed: false,
            fullscreen_center: true,
            refresh_monitors_button: (),

            // cursor
            cursor_scale: 1.0,
            cursor_border: 1.5,
            cursor_color: Color::from_hex("#ffff32".to_owned()),
            cursor_border_color: Color::from_hex("#000".to_owned()),
            cursor_ripples: true,
            cursor_ripple_color: Color::from_hex("#000".to_owned()),
            cursor_ripple_final_scale: 1.5,
            
            // keybinds
            key_user_panel: Key::F8,

            // doubletap protection
            enable_double_tap_protection: false,
            double_tap_protection_duration: 80.0,
            
            // integrations
            integrations: Default::default(),

            // other
            last_git_hash: String::new(),

            current_skin: "None".to_owned(),
            refresh_skins_button: (),

            external_games_folders: Vec::new(),

            skip_autosaveing: false,
            theme: SelectedTheme::Tataku
        }
    }
}
