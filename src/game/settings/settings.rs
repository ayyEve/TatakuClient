use crate::prelude::*;

const SETTINGS_FILE:&str = "settings.json";

lazy_static::lazy_static! {
    pub static ref SETTINGS: Arc<OnceCell<RwLock<Settings>>> = Arc::new(OnceCell::const_new());
    // static ref WINDOW_SIZE: OnceCell<Vector2> = OnceCell::const_new();
    // pub static ref LAST_CALLER:Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}

#[macro_export]
macro_rules! get_settings {
    () => {
        Settings::get().await
    }
}

#[macro_export]
macro_rules! get_settings_mut {
    () => {
        MutSettingsHelper::new(Settings::get_mut().await)
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    // audio
    pub master_vol: f32,
    pub music_vol: f32,
    pub effect_vol: f32,
    pub global_offset: f32,
    
    // login
    pub username: String,
    pub password: String,
    pub server_url: String,
    pub score_url: String,
    
    // osu login (for direct)
    pub osu_username: String,
    pub osu_password: String,
    pub osu_api_key: String,
    
    // game settings
    pub logging_settings: LoggingSettings,
    pub standard_settings: StandardSettings,
    pub taiko_settings: TaikoSettings,
    pub catch_settings: CatchSettings,
    pub mania_settings: ManiaSettings,
    pub background_game_settings: BackgroundGameSettings,
    pub common_game_settings: CommonGameplaySettings,
    pub last_played_mode: PlayMode,
    pub current_skin: String,
    pub external_games_folders: Vec<String>,

    // window settings
    pub fps_target: u64,
    pub update_target: u64,
    pub window_size: [f64; 2],
    pub ui_scale: f64,
    pub background_dim: f32,
    /// should the game pause when focus is lost?
    pub pause_on_focus_lost: bool,

    // cursor
    pub cursor_color: String,
    pub cursor_scale: f64,
    pub cursor_border: f32,
    pub cursor_border_color: String,
    

    // misc keybinds
    pub key_user_panel: Key,

    // other misc
    pub last_git_hash: String,
    pub current_version: String,

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

        // save after loading.
        // writes file if it doesnt exist, and writes new values from updates
        s.save().await;
        s
    }
    pub async fn save(&self) {
        trace!("Saving settings");
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
            standard_settings: StandardSettings::default(),
            taiko_settings: TaikoSettings::default(),
            catch_settings: CatchSettings::default(),
            mania_settings: ManiaSettings::default(),
            background_game_settings: BackgroundGameSettings::default(),
            common_game_settings: CommonGameplaySettings::default(),
            pause_on_focus_lost: true,
            last_played_mode: "osu".to_owned(),
            current_skin: "None".to_owned(),
            external_games_folders: Vec::new(),

            // window settings
            fps_target: 144,
            update_target: 10000,
            window_size: [1280.0, 720.0],
            ui_scale: 1.0,
            background_dim: 0.8,

            // cursor
            cursor_scale: 2.0,
            cursor_border: 1.5,
            cursor_color: "#ffff32".to_owned(),
            cursor_border_color: "#000".to_owned(),
            
            // keybinds
            key_user_panel: Key::F8,
            
            // other
            last_git_hash: String::new(),
            current_version: String::new(),
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
