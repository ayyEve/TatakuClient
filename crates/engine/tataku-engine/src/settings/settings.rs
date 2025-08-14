use crate::prelude::*;

#[cfg(feature="graphics")]
use tataku_client_proc_macros::Settings;

const SETTINGS_FILE:&str = "settings.json";

#[derive(Serialize)]
#[derive(Clone, PartialEq, Debug2)]
#[cfg_attr(feature="graphics", derive(Settings))]
#[derive(SettingsDeserialize, Reflect)]
#[serde(default)]
#[allow(clippy::manual_non_exhaustive)]
pub struct Settings {
    #[serde(skip)]
    pub save_path: String,

    #[serde(skip)]
    #[debug(skip)]
    #[reflect(rename="buildable")]
    pub buildable_provider: Arc<BuildableSettingsProvider>,

    // audio
    // #[Setting(text="Master Volume", category="Audio Settings")]
    pub master_vol: f32,
    // #[Setting(text="Music Volume")]
    pub music_vol: f32,
    // #[Setting(text="Effect Volume")]
    pub effect_vol: f32,
    #[cfg_attr(feature="graphics", setting(text="Global Offset", min=-100.0, max=100.0, category="Audio Settings"))]
    pub global_offset: f32,
    
    // login
    #[cfg_attr(feature="graphics", setting(text="Tataku Username", category="Connection Settings"))]
    pub username: String,
    #[cfg_attr(feature="graphics", setting(text="Tataku Password", password=true))]
    pub password: String,
    #[cfg_attr(feature="graphics", setting(text="Tataku Server Url"))]
    pub server_url: String,
    #[cfg_attr(feature="graphics", setting(text="Tataku Score Url"))]
    pub score_url: String,
    
    // osu login (for direct)
    #[cfg_attr(feature="graphics", setting(text="Osu Username", category="Osu Integration"))]
    pub osu_username: String,
    #[cfg_attr(feature="graphics", setting(text="Osu Password", password=true))]
    pub osu_password: String,
    #[cfg_attr(feature="graphics", setting(text="Osu Api Key", password=true))]
    pub osu_api_key: String,
    
    // game settings
    #[cfg_attr(feature="graphics", subsetting())]
    pub gamemode_settings: GamemodeSettingsCollection,

    #[cfg_attr(feature="graphics", subsetting(category="Background Game Settings"))]
    pub background_game_settings: BackgroundGameSettings,
    #[cfg_attr(feature="graphics", subsetting(category="Common Game Settings"))]
    pub common_game_settings: CommonGameplaySettings,

    pub last_played_mode: String,
    pub score_method: ScoreRetreivalMethod,
    pub sort_by: SortBy,
    
    #[cfg_attr(feature="graphics", setting(text="Beatmap Hitsounds"))]
    pub beatmap_hitsounds: bool,

    #[cfg_attr(feature="graphics", setting(text="Enable Difficulty Calculation"))]
    pub enable_diffcalc: bool,

    #[cfg_attr(feature="graphics", subsetting(category="Display Settings"))]
    pub display_settings: DisplaySettings,
    
    // cursor
    pub cursor_settings: CursorSettings,

    // skin settings
    #[cfg_attr(feature="graphics", setting(text="Skin", dropdown="SkinDropdownable", category="Skin Settings"))]
    pub current_skin: String,

    // TODO:
    #[serde(skip)]
    #[reflect(skip)]
    #[cfg_attr(feature="graphics", setting(text="Refresh Skins", action="GameAction::RefreshSkins"))]
    refresh_skins_button: (),

    #[cfg_attr(feature="graphics", setting(text="Theme", dropdown="SelectedTheme"))]
    pub theme: SelectedTheme,

    #[cfg_attr(feature="graphics", setting(text="UI Scale", min=0.1, max=4.0))] // not ready yet
    pub ui_scale: f32,
    #[cfg_attr(feature="graphics", setting(text="Background Dim", min=0, max=1))]
    pub background_dim: f32,

    // misc keybinds
    #[cfg_attr(feature="graphics", setting(text="User Panel Key", category="Common Keybinds"))]
    pub key_user_panel: Key,

    // double tap protection
    #[cfg_attr(feature="graphics", setting(text="Enable DoubleTap Protection", category="DoubleTap Protection"))]
    pub enable_double_tap_protection: bool,
    #[cfg_attr(feature="graphics", setting(text="DoubleTap Protection Leniency", min=10.0, max=200.0))]
    pub double_tap_protection_duration: f32,


    // integrations
    #[cfg_attr(feature="graphics", subsetting(category="Integrations"))]
    pub integrations: IntegrationSettings,


    // other misc
    pub last_git_hash: String,
    pub external_games_folders: Vec<String>,
    
    #[cfg_attr(feature="graphics", subsetting(category="Log Settings"))]
    pub logging_settings: LoggingSettings,
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
                Self::default()
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
        if !self.osu_password.is_empty() { 
            self.osu_password = Cryptography::check_md5(self.osu_password.clone());
        }
        if !self.password.is_empty() { 
            self.password = Cryptography::check_sha512(self.password.clone());
        }
    }

    // make a backup of the setting before they're overwritten (when the file fails to load)
    fn backup_settings(settings_path: &Path) -> Option<String> {
        if !Io::exists(settings_path) { return None }
        let settings_path = settings_path.to_string_lossy().to_string();

        let mut counter = 0;
        let mut file = format!("{settings_path}.bak_{counter}");
        while Io::exists(&file) {
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

}
impl Default for Settings {
    fn default() -> Self {
        Self {
            buildable_provider: Arc::default(),
            save_path: String::new(),

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
            logging_settings: LoggingSettings::default(),
            gamemode_settings: GamemodeSettingsCollection::default(),
            background_game_settings: BackgroundGameSettings::default(),
            common_game_settings: CommonGameplaySettings::default(),
            last_played_mode: "osu".to_owned(),
            score_method: ScoreRetreivalMethod::Local,
            sort_by: SortBy::Title,
            beatmap_hitsounds: true,
            enable_diffcalc: true,

            cursor_settings: CursorSettings::default(),
            
            // keybinds
            key_user_panel: Key::F8,

            // doubletap protection
            enable_double_tap_protection: false,
            double_tap_protection_duration: 80.0,
            
            // integrations
            integrations: IntegrationSettings::default(),

            display_settings: DisplaySettings::default(),
            ui_scale: 1.0,
            background_dim: 0.8,

            // other
            last_git_hash: String::new(),

            current_skin: "None".to_owned(),
            refresh_skins_button: (),

            external_games_folders: Vec::new(),
            theme: SelectedTheme::Tataku
        }
    }
}


//TODO: move this
lazy_static::lazy_static! {
    // TODO: change this to skin meta
    pub static ref AVAILABLE_SKINS:Arc<RwLock<Vec<String>>> = {
        let mut list = vec!["None".to_owned()];
        if let Ok(folder) = std::fs::read_dir(SKINS_FOLDER) {
            for f in folder.filter_map(|f| f.ok()) {
                list.push(f.file_name().to_string_lossy().to_string());
            }
        }
        Arc::new(RwLock::new(list))
    };
}
// pub struct SkinDropdownable;
// #[cfg(feature="graphics")]
// impl Dropdownable2 for SkinDropdownable {
//     type T = String;
//     fn variants() -> Vec<String> {
//         AVAILABLE_SKINS.read().clone() //.iter().map(|s|Self::Skin(s.clone())).collect()
//     }
// }


lazy_static::lazy_static! {
    static ref THEMES: Vec<(String, String)> = {
        Vec::new()
    };
}

#[derive(Clone, Eq, PartialEq, Debug)]
#[derive(Reflect)]
#[derive(Serialize, Deserialize)]
#[reflect(display = "display")]
pub enum SelectedTheme {
    Tataku,
    Osu,
    /// path to theme file, name of theme
    Custom(String, String),
}
// #[cfg(feature="graphics")]
// impl tataku_client_common::Dropdownable2 for SelectedTheme {
//     type T = Self;
//     fn variants() -> Vec<Self::T> {
//         [Self::Tataku, Self::Osu]
//             .into_iter()
//             .chain(THEMES.clone().into_iter().map(|t| Self::Custom(t.0, t.1)))
//             .collect()
//     }
// }
impl Display for SelectedTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tataku => write!(f, "Tataku"),
            Self::Osu => write!(f, "Osu"),
            Self::Custom(_, name) => write!(f, "{name}"),
        }
    }
}
