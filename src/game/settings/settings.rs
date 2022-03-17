use crate::prelude::*;

const SETTINGS_FILE:&str = "settings.json";

use parking_lot::{RwLockReadGuard, RwLockWriteGuard};

lazy_static::lazy_static! {
    pub static ref SETTINGS: Arc<RwLock<Settings>> = Arc::new(RwLock::new(Settings::load()));
    pub static ref WINDOW_SIZE: OnceCell<Vector2> = OnceCell::new_with(Some(get_settings!().window_size.into()));
    pub static ref LAST_CALLER:Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}

#[macro_export]
macro_rules! get_settings {
    () => {{
        let caller = format!("{}:{}:{}", file!(), line!(), column!());
        Settings::get(caller)
    }}
}

#[macro_export]
macro_rules! get_settings_mut {
    () => {{
        let caller = format!("{}:{}:{}", file!(), line!(), column!());
        Settings::get_mut(caller)
    }}
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
    
    // osu login (for direct)
    pub osu_username: String,
    pub osu_password: String,
    
    // game settings
    pub standard_settings: StandardSettings,
    pub taiko_settings: TaikoSettings,
    pub catch_settings: CatchSettings,
    pub mania_settings: ManiaSettings,
    pub background_game_settings: BackgroundGameSettings,
    pub common_game_settings: CommonGameplaySettings,
    pub last_played_mode: PlayMode,

    // window settings
    pub fps_target: u64,
    pub update_target: u64,
    pub window_size: [f64; 2],

    // cursor
    pub cursor_color: String,
    pub cursor_scale: f64,
    pub cursor_border: f32,
    pub cursor_border_color: String,
    

    // bg
    pub background_dim: f32,
    /// should the game pause when focus is lost?
    pub pause_on_focus_lost: bool,


    // misc keybinds
    pub key_user_panel: Key,

    // other misc
    pub last_git_hash: String,
    pub server_url: String,
    pub current_skin: String,

    pub external_games_folders: Vec<String>
}
impl Settings {
    fn load() -> Settings {
        let mut s = match std::fs::read_to_string(SETTINGS_FILE) {
            Ok(b) => match serde_json::from_str(&b) {
                Ok(settings) => settings,
                Err(e) => {
                    // println!("error reading settings.json, loading defaults");
                    NotificationManager::add_error_notification("Error reading settings.json\nLoading defaults", e);
                    Settings::default()
                }
            }
            Err(e) => {
                // println!("error reading settings.json, loading defaults");
                NotificationManager::add_error_notification("Error reading settings.json\nLoading defaults", e);
                Settings::default()
            }
        };

        // check password hashes
        s.check_hashes();

        // save after loading.
        // writes file if it doesnt exist, and writes new values from updates
        s.save();
        s
    }
    pub fn save(&self) {
        println!("Saving settings");
        let str = serde_json::to_string_pretty(self).unwrap();
        match std::fs::write(SETTINGS_FILE, str) {
            Ok(_) => println!("settings saved successfully"),
            Err(e) => NotificationManager::add_error_notification("Error saving settings", e),
        }
    }

    /// relatively slow, if you need a more performant get, use get_mut
    pub fn get<'a>(caller: String) -> RwLockReadGuard<'a, Settings> {
        if SETTINGS.is_locked_exclusive() {
            // panic bc the devs should know when this error occurs, as it completely locks up the app
            let last_caller = LAST_CALLER.lock();
            panic!("Settings Double Locked! Called by {}, locked by {}", caller, last_caller);
        }

        *LAST_CALLER.lock() = caller;
        SETTINGS.read()
    }

    /// more performant, but can double lock if you arent careful
    pub fn get_mut<'a>(caller:String) -> RwLockWriteGuard<'a, Settings> {
        if SETTINGS.is_locked() {
            // panic bc the devs should know when this error occurs, as it completely locks up the app
            let last_caller = LAST_CALLER.lock();
            panic!("Settings Double Locked! Called by {}, locked by {}", caller, last_caller);
        }

        *LAST_CALLER.lock() = caller;
        SETTINGS.write()
    }

    pub fn window_size() -> Vector2 {*WINDOW_SIZE.get().unwrap()}

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

            // mode settings
            standard_settings: StandardSettings::default(),
            taiko_settings: TaikoSettings::default(),
            catch_settings: CatchSettings::default(),
            mania_settings: ManiaSettings::default(),
            background_game_settings: BackgroundGameSettings::default(),
            common_game_settings: CommonGameplaySettings::default(),
            pause_on_focus_lost: true,
            last_played_mode: "osu".to_owned(),

            // window settings
            fps_target: 144,
            update_target: 10000,
            window_size: [1280.0, 720.0],
            background_dim: 0.8,

            // cursor
            cursor_scale: 2.0,
            cursor_border: 1.5,
            cursor_color: "#ffff32".to_owned(),
            cursor_border_color: "#000".to_owned(),
            

            // keys
            key_user_panel: Key::F8,
            

            // other
            last_git_hash: String::new(),

            server_url: "wss://taikors.ayyeve.xyz".to_owned(),
            current_skin: "default".to_owned(),

            external_games_folders: Vec::new()
        }
    }
}
