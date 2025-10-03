use crate::*;
use common::reflect::*;
use tataku_client_proc_macros::Settings;
use engine::settings::*;

const SETTINGS_FILE:&str = "settings.json";

#[derive(Reflect, Settings)]
#[allow(clippy::manual_non_exhaustive)]
#[derive(Serialize, DeserializeSettings)]
#[derive(Clone, Debug2, Default, PartialEq)]
#[serde(default)]
pub struct Settings {
    #[serde(skip)] #[debug(skip)] #[reflect(skip)]
    #[category(text="Settings")] _settings: (),
    
    #[serde(skip)]
    pub save_path: String,

    #[serde(skip)] #[debug(skip)] #[reflect(skip)]
    #[divider(text="Audio Settings")] _audio: (),
    

    #[cfg(feature="ui")]
    #[serde(skip)] #[debug(skip)] 
    #[reflect(rename="buildable")]
    pub buildable_provider: Arc<BuildableSettingsProvider>,

    // audio
    // #[setting(text="Master Volume")]
    pub master_vol: f32,
    // #[setting(text="Music Volume")]
    pub music_vol: f32,
    // #[setting(text="Effect Volume")]
    pub effect_vol: f32,
    #[setting(text="Global Offset", range(-100.0, 100.0))]
    pub global_offset: f32,
    
    // connection
    #[serde(skip)] #[debug(skip)] #[reflect(skip)]
    #[divider(text="Connection Settings")] _connections: (),

    #[subsetting()]
    pub connection_settings: connection::ConnectionSettings,
    
    // game settings
    #[subsetting()]
    pub gamemode_settings: GamemodeSettingsCollection,
    
    #[subsetting(text="Background Game Settings")]
    pub background_game_settings: background_game::BackgroundGameSettings,

    #[subsetting(text="Common Game Settings")]
    pub common_game_settings: common_gameplay::CommonGameplaySettings,

    pub last_played_mode: String,
    pub score_method: engine::data::ScoreRetreivalMethod,
    pub sort_by: engine::data::SortBy,
    
    #[setting(text="Beatmap Hitsounds")]
    pub beatmap_hitsounds: bool,

    #[setting(text="Enable Difficulty Calculation")]
    pub enable_diffcalc: bool,

    #[subsetting(text="Display Settings")]
    pub display_settings: display::DisplaySettings,
    
    // cursor
    pub cursor_settings: cursor::CursorSettings,

    // skin settings
    #[serde(skip)] #[debug(skip)] #[reflect(skip)]
    #[category(text="Skin Settings")] _skin_settings: (),

    #[dropdown(text="Skin", path="enums.skins")]
    pub current_skin: String,

    // TODO:
    #[serde(skip)] #[debug(skip)] #[reflect(skip)]
    #[button(text="Refresh Skins", action="actions::game::GameAction::RefreshSkins")] _refresh_skins_button: (),

    #[dropdown(text="Theme", path="enums.themes")]
    pub theme: SelectedTheme,

    #[setting(text="UI Scale", range(0.1, 4.0))] // not ready yet
    pub ui_scale: f32,
    #[setting(text="Background Dim", range(0.0, 1.0))]
    pub background_dim: f32,

    // misc keybinds
    #[doc(hidden)]
    #[serde(skip)] #[debug(skip)] #[reflect(skip)]
    #[category(text="Common Keybinds")] _5: (),

    #[setting(text="User Panel Key")]
    pub key_user_panel: input::Key,

    // double tap protection
    #[serde(skip)] #[debug(skip)] #[reflect(skip)]
    #[category(text="DoubleTap Protection")] _double_tap_prot: (),

    #[setting(text="Enable DoubleTap Protection")]
    pub enable_double_tap_protection: bool,
    #[setting(text="DoubleTap Protection Leniency", range(10.0, 200.0))]
    pub double_tap_protection_duration: f32,


    // integrations
    #[subsetting(text="Integrations")]
    pub integrations: integration::IntegrationSettings,

    // other misc
    // pub last_git_hash: String,
    pub external_games_folders: Vec<String>,
    
    #[subsetting(text="Log Settings")]
    pub logging_settings: logging::LoggingSettings,
}
impl Settings {
    pub fn load() -> Self {
        Self::load_from(SETTINGS_FILE)
    }
    pub fn load_from(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();

        let mut s = match std::fs::read_to_string(path)
            .map(|s| serde_json::from_str::<Settings>(&s)
            .map_err(|e| e.to_string()))
            .map_err(|e| e.to_string())
        {
            Ok(Ok(settings)) => settings,
            Err(e) | Ok(Err(e)) => {
                warn!("Error reading settings.json\nLoading defaults, {e}");
                
                if let Some(saved_as) = Self::backup_settings(path) {
                    info!("Old settings saved to {saved_as}");
                }
                Self::default_settings()
            }
        };
        s.save_path = path.to_string_lossy().to_string();

        // check password hashes
        s.check_hashes();
        
        // save after loading.
        // writes file if it doesnt exist, and writes new values from updates
        s.save();
        s
    }

    pub fn save(&mut self) {
        debug!("Saving settings");
        self.gamemode_settings.update();

        let str = serde_json::to_string_pretty(self).unwrap();
        match std::fs::write(&self.save_path, str) {
            Ok(_) => trace!("settings saved successfully"),
            Err(e) => error!("Error saving settings: {e}"),
        }
    }

    pub fn connection(&self) -> &connection::ConnectionSettingsProfile {
        &self.connection_settings.current
    }

    #[cfg(feature="ui")]
    pub fn init(
        &mut self, 
        values: &mut dyn Reflect,
        prefix: String,
    ) {
        let mut builder = SettingsBuilder::new(values, "Settings");
        #[cfg(feature = "graphics")]
        self.create_provider(prefix, &mut builder);
        self.buildable_provider = Arc::new(builder.done());
    }

    pub fn gamemode_settings<G: serde::de::DeserializeOwned>(
        &self, 
        gamemode: impl AsRef<str>
    ) -> Option<G> {
        serde_json::from_value(self.gamemode_settings.get(gamemode.as_ref())?.clone())
            .ok()
    }

    pub fn update_gamemode_settings<G: serde::Serialize>(
        &mut self, 
        gamemode: impl AsRef<str>, 
        settings: G
    ) {
        *self.gamemode_settings
            .entry(gamemode.as_ref().to_owned())
            .or_default()
        = serde_json::to_value(settings)
            .expect("couldnt serialize game settings?");

        self.gamemode_settings.rebuild();
    }


    pub fn get_effect_vol(&self) -> f32 { self.effect_vol * self.master_vol }
    pub fn get_music_vol(&self) -> f32 { self.music_vol * self.master_vol }

    pub fn check_hashes(&mut self) {
        let osu_pw = &mut self.integrations.osu.password;
        if !osu_pw.is_empty() { 
            *osu_pw = tataku::Cryptography::check_md5(osu_pw.clone());
        }

        let tataku_pw = &mut self.connection_settings.current.tataku_password;
        if !tataku_pw.is_empty() { 
            *tataku_pw = tataku::Cryptography::check_sha512(tataku_pw.clone());
        }
    }

    // make a backup of the setting before they're overwritten (when the file fails to load)
    fn backup_settings(settings_path: &Path) -> Option<String> {
        if !tataku::fs::exists(settings_path) { return None }
        let settings_path = settings_path.to_string_lossy().to_string();

        let mut counter = 0;
        let mut file = format!("{settings_path}.bak_{counter}");
        while tataku::fs::exists(&file) {
            counter += 1;
            file = format!("{settings_path}.bak_{counter}");
        }
        std::fs::copy(&settings_path, &file)
            .expect("An error occurred while backing up the old settings.json");
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
    }


    fn default_settings() -> Self {
        Self {
            // audio
            music_vol: 0.5,
            effect_vol: 0.5,
            master_vol: 0.3,
            global_offset: 0.0,

            // login
            // username: "Guest".to_owned(),
            // server_url: "wss://server.tataku.ca".to_owned(),
            // score_url: "https://scores.tataku.ca".to_owned(),

            // game settings
            last_played_mode: "osu".to_owned(),
            score_method: engine::data::ScoreRetreivalMethod::Local,
            sort_by: engine::data::SortBy::Title,
            beatmap_hitsounds: true,
            enable_diffcalc: true,
            // keybinds
            key_user_panel: input::Key::F8,

            // doubletap protection
            double_tap_protection_duration: 80.0,
            
            ui_scale: 1.0,
            background_dim: 0.8,

            current_skin: "None".to_owned(),

            ..Default::default()
        }
    
    }
}

#[derive(Reflect)]
#[reflect(display = "display")]
#[derive(Serialize, Deserialize)]
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub enum SelectedTheme {
    #[default]
    Tataku,
    Osu,
    /// path to theme file, name of theme
    Custom(String, String),
}
impl std::fmt::Display for SelectedTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tataku => write!(f, "Tataku"),
            Self::Osu => write!(f, "Osu"),
            Self::Custom(_, name) => write!(f, "{name}"),
        }
    }
}
