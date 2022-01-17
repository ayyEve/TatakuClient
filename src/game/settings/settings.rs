use crate::prelude::*;


const SETTINGS_FILE:&str = "settings.json";

lazy_static::lazy_static! {
    static ref SETTINGS: Arc<Mutex<Settings>> = Arc::new(Mutex::new(Settings::load()));

    pub static ref WINDOW_SIZE: OnceCell<Vector2> = OnceCell::new_with(Some(Settings::get_mut("WINDOW_SIZE").window_size.into()));

    static ref LAST_CALLER:Arc<Mutex<&'static str>> = Arc::new(Mutex::new("None"));
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub key_offset_up: Key,
    pub key_offset_down: Key,

    pub last_git_hash: String,

    pub server_url: String,
}
impl Settings {
    fn load() -> Settings {
        let s = match std::fs::read_to_string(SETTINGS_FILE) {
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
    pub fn get() -> Settings {SETTINGS.lock().clone()}

    /// more performant, but can double lock if you arent careful
    pub fn get_mut<'a>(caller:&'static str) -> MutexGuard<'a, Settings> {
        if SETTINGS.is_locked() {
            // panic bc the devs should know when this error occurs, as it completely locks up the app
            let last_caller = LAST_CALLER.lock();
            panic!("Settings Double Locked! Called by {}, locked by {}", caller, last_caller);
        }

        *LAST_CALLER.lock() = caller;
        SETTINGS.lock()
    }

    pub fn window_size() -> Vector2 {*WINDOW_SIZE.get().unwrap()}

    pub fn get_effect_vol(&self) -> f32 {self.effect_vol * self.master_vol}
    pub fn get_music_vol(&self) -> f32 {self.music_vol * self.master_vol}
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
            standard_settings: StandardSettings{..Default::default()},
            taiko_settings: TaikoSettings {..Default::default()},
            catch_settings: CatchSettings {..Default::default()},
            mania_settings: ManiaSettings {..Default::default()},
            background_game_settings: BackgroundGameSettings {..Default::default()},
            pause_on_focus_lost: true,

            // window settings
            fps_target: 144,
            update_target: 1000,
            window_size: [1000.0, 600.0],
            background_dim: 0.8,

            // cursor
            cursor_scale: 2.0,
            cursor_border: 1.5,
            cursor_color: "#ffff32".to_owned(),
            cursor_border_color: "#000".to_owned(),
            

            // keys
            key_offset_up: Key::Equals,
            key_offset_down: Key::Minus,

            // other
            last_git_hash: String::new(),

            server_url: "wss://taikors.ayyeve.xyz".to_owned()
        }
    }
}




#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct BackgroundGameSettings {
    /// wether to have gameplay in the main menu bg or not
    pub enabled: bool,
    /// gameplay alpha multiplier
    pub opacity: f32,
    /// hitsound volume multiplier
    pub hitsound_volume: f32,
    /// what mode should be playing?
    pub mode: PlayMode,
}
impl Default for BackgroundGameSettings {
    fn default() -> Self {
        Self { 
            enabled: true,
            opacity: 0.5,
            hitsound_volume: 0.3,
            mode: PlayMode::Standard
        }
    }
}