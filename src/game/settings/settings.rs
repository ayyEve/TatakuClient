use crate::prelude::*;
use tataku_client_proc_macros::Settings;


const SETTINGS_FILE:&str = "settings.json";

lazy_static::lazy_static! {
    pub static ref SETTINGS: Arc<OnceCell<RwLock<Settings>>> = Arc::new(OnceCell::const_new());
}

#[macro_export]
macro_rules! get_settings {
    () => {{
        Settings::get().await
    }}
}

#[macro_export]
macro_rules! get_settings_mut {
    () => {{
        MutSettingsHelper::new(Settings::get_mut().await)
    }}
}

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
    #[Subsetting(category="Osu Settings")]
    pub standard_settings: StandardSettings,
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
    
    #[Setting(text="Gamemode Ripple Override")]
    pub allow_gamemode_cursor_ripple_override: bool,
    #[Setting(text="Beatmap Hitsounds")]
    pub beatmap_hitsounds: bool,

    // window settings
    pub window_size: [f64; 2],
    #[Setting(text="FPS Limit", min=15, max=240, category="Window Settings")]
    pub fps_target: u64,
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

    // cursor
    #[Setting(text="Cursor Color", category="Cursor Settings")]
    pub cursor_color: String,
    #[Setting(text="Cursor Scale", min=0.1, max=10.0)]
    pub cursor_scale: f64,
    #[Setting(text="Cursor Border", min=0.1, max=5.0)]
    pub cursor_border: f32,
    #[Setting(text="Cursor Border Color")]
    pub cursor_border_color: String,

    #[Setting(text="Cursor Ripples")]
    pub cursor_ripples: bool,
    #[Setting(text="Cursor Ripple Color")]
    pub cursor_ripple_color: String,
    #[Setting(text="Cursor Ripple Scale")]
    pub cursor_ripple_final_scale: f64,



    // misc keybinds
    #[Setting(text="User Panel Key", category="Common Keybinds")]
    pub key_user_panel: Key,

    // double tap protection
    #[Setting(text="Enable DoubleTap Protection", category="DoubleTap Protection")]
    pub enable_double_tap_protection: bool,
    #[Setting(text="DoubleTap Protection Leniency", min=10.0, max=200.0)]
    pub double_tap_protection_duration: f32,


    // other misc
    pub last_git_hash: String,
    
    #[Setting(text="Skin", dropdown="SkinDropdownable", dropdown_value="Skin", category="Skin Settings")]
    pub current_skin: String,

    pub logging_settings: LoggingSettings,
    pub external_games_folders: Vec<String>,

    #[serde(skip)]
    pub skip_autosaveing: bool,
}
impl Settings {
    pub async fn load() -> Settings {
        let mut s = match std::fs::read_to_string(SETTINGS_FILE).map(|s| serde_json::from_str(&s).map_err(|e|e.to_string())).map_err(|e|e.to_string()) {
            Ok(Ok(settings)) => settings,
            Err(e) | Ok(Err(e)) => {
                // warn!("error reading settings.json, loading defaults");
                NotificationManager::add_error_notification("Error reading settings.json\nLoading defaults", e).await;
                backup_settings().await;
                Settings::default()
            }
        };

        // check password hashes
        s.check_hashes();

        // // set window size const
        // WINDOW_SIZE.set(s.window_size.into()).unwrap();
        
        SETTINGS.set(RwLock::new(s.clone())).ok().unwrap();
        *super::SETTINGS_CHECK.write().unwrap() = Arc::new(s.clone());

        // save after loading.
        // writes file if it doesnt exist, and writes new values from updates
        s.save().await;
        s
    }
    pub async fn save(&self) {
        info!("Saving settings");
        let str = serde_json::to_string_pretty(self).unwrap();
        match std::fs::write(SETTINGS_FILE, str) {
            Ok(_) => trace!("settings saved successfully"),
            Err(e) => NotificationManager::add_error_notification("Error saving settings", e).await,
        }
    }

    pub async fn get<'a>() -> tokio::sync::RwLockReadGuard<'a, Settings> {
        SETTINGS.get().unwrap().read().await
    }

    /// more performant, but can double lock if you arent careful
    pub async fn get_mut<'a>() -> tokio::sync::RwLockWriteGuard<'a, Settings> {
        SETTINGS.get().unwrap().write().await
    }

    // pub fn window_size() -> Vector2 {
    //     *WINDOW_SIZE.get().unwrap()
    // }

    pub fn get_effect_vol(&self) -> f32 {self.effect_vol * self.master_vol}
    pub fn get_music_vol(&self) -> f32 {self.music_vol * self.master_vol}

    pub fn check_hashes(&mut self) {
        if self.osu_password.len() > 0 {self.osu_password = check_md5(self.osu_password.clone())}
        if self.password.len() > 0 {self.password = check_sha512(self.password.clone())}
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

            // username
            username: "Guest".to_owned(),
            password: String::new(),

            // osu
            osu_username: String::new(),
            osu_password: String::new(),
            osu_api_key: String::new(),

            // mode settings
            standard_settings: StandardSettings::default(),
            taiko_settings: TaikoSettings::default(),
            catch_settings: CatchSettings::default(),
            mania_settings: ManiaSettings::default(),
            background_game_settings: BackgroundGameSettings::default(),
            common_game_settings: CommonGameplaySettings::default(),
            pause_on_focus_lost: true,
            last_played_mode: "osu".to_owned(),
            last_score_retreival_method: ScoreRetreivalMethod::Local,
            last_sort_by: SortBy::Title,
            allow_gamemode_cursor_ripple_override: true,
            beatmap_hitsounds: true,

            // window settings
            fps_target: 144,
            update_target: 10_000,
            window_size: [1280.0, 720.0],
            ui_scale: 1.0,
            background_dim: 0.8,
            raw_mouse_input: false,

            // cursor
            cursor_scale: 1.0,
            cursor_border: 1.5,
            cursor_color: "#ffff32".to_owned(),
            cursor_border_color: "#000".to_owned(),
            cursor_ripples: true,
            cursor_ripple_color: "#000".to_owned(),
            cursor_ripple_final_scale: 1.5,
            

            // keys
            key_user_panel: Key::F8,

            // doubletap protection
            enable_double_tap_protection: false,
            double_tap_protection_duration: 80.0,
            

            // other
            last_git_hash: String::new(),

            server_url: "wss://server.tataku.ca".to_owned(),
            score_url: "https://tataku.ca".to_owned(),
            current_skin: "None".to_owned(),
            logging_settings: LoggingSettings::new(),

            external_games_folders: Vec::new(),

            skip_autosaveing: false
        }
    }
}

// make a backup of the setting before they're overwritten (when the file fails to load)
async fn backup_settings() {
    if exists(SETTINGS_FILE) {
        let mut counter = 0;
        while exists(format!("{SETTINGS_FILE}.bak_{counter}")) {
            counter += 1;
        }
        let file = format!("{SETTINGS_FILE}.bak_{counter}");

        if let Err(e) = std::fs::copy(SETTINGS_FILE, &file) {
            NotificationManager::add_error_notification("Error backing up settings.json", e).await
        } else {
            NotificationManager::add_text_notification(
                &format!("Backup saved as {file}"),
                5000.0,
                Color::YELLOW
            ).await;
        }
    }
}
