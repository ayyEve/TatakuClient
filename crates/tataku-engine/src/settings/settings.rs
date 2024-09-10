use crate::prelude::*;
use tataku_client_proc_macros::Settings;

const SETTINGS_FILE:&str = "settings.json";

#[derive(Clone, Serialize, PartialEq, Debug)]
#[cfg_attr(feature="graphics", derive(Settings))]
#[derive(SettingsDeserialize, Reflect)]
#[serde(default)]
pub struct Settings {
    #[serde(skip)]
    pub save_path: String,


    // audio
    // #[Setting(text="Master Volume", category="Audio Settings")]
    pub master_vol: f32,
    // #[Setting(text="Music Volume")]
    pub music_vol: f32,
    // #[Setting(text="Effect Volume")]
    pub effect_vol: f32,
    #[cfg_attr(feature="graphics", Setting(text="Global Offset", min=-100.0, max=100.0, category="Audio Settings"))]
    pub global_offset: f32,
    
    // login
    #[cfg_attr(feature="graphics", Setting(text="Tataku Username", category="Tataku Server Settings"))]
    pub username: String,
    #[cfg_attr(feature="graphics", Setting(text="Tataku Password", password=true))]
    pub password: String,
    #[cfg_attr(feature="graphics", Setting(text="Tataku Server Url"))]
    pub server_url: String,
    #[cfg_attr(feature="graphics", Setting(text="Tataku Score Url"))]
    pub score_url: String,
    
    // osu login (for direct)
    #[cfg_attr(feature="graphics", Setting(text="Osu Username", category="Osu Integration"))]
    pub osu_username: String,
    #[cfg_attr(feature="graphics", Setting(text="Osu Password", password=true))]
    pub osu_password: String,
    #[cfg_attr(feature="graphics", Setting(text="Osu Api Key", password=true))]
    pub osu_api_key: String,
    
    // game settings
    #[serde(alias="standard_settings")]
    #[cfg_attr(feature="graphics", Subsetting(category="Osu Settings"))]
    pub osu_settings: OsuSettings,
    #[cfg_attr(feature="graphics", Subsetting(category="Taiko Settings"))]
    pub taiko_settings: TaikoSettings,
    // #[Subsetting(category="Catch Settings")]
    pub catch_settings: CatchSettings,
    #[cfg_attr(feature="graphics", Subsetting(category="Mania Settings"))]
    pub mania_settings: ManiaSettings,
    #[cfg_attr(feature="graphics", Subsetting(category="Background Game Settings"))]
    pub background_game_settings: BackgroundGameSettings,
    #[cfg_attr(feature="graphics", Subsetting(category="Common Game Settings"))]
    pub common_game_settings: CommonGameplaySettings,

    pub last_played_mode: String,
    pub score_method: ScoreRetreivalMethod,
    pub sort_by: SortBy,
    
    #[cfg_attr(feature="graphics", Setting(text="Beatmap Hitsounds"))]
    pub beatmap_hitsounds: bool,

    #[cfg_attr(feature="graphics", Setting(text="Enable Difficulty Calculation"))]
    pub enable_diffcalc: bool,

    #[cfg_attr(feature="graphics", Subsetting(category="Display Settings"))]
    pub display_settings: DisplaySettings,
    

    // cursor
    #[cfg_attr(feature="graphics", Setting(text="Cursor Color", category="Cursor Settings"))]
    pub cursor_color: SettingsColor,
    #[cfg_attr(feature="graphics", Setting(text="Cursor Scale", min=0.1, max=10.0))]
    pub cursor_scale: f32,
    #[cfg_attr(feature="graphics", Setting(text="Cursor Border", min=0.1, max=5.0))]
    pub cursor_border: f32,
    #[cfg_attr(feature="graphics", Setting(text="Cursor Border Color"))]
    pub cursor_border_color: SettingsColor,

    #[cfg_attr(feature="graphics", Setting(text="Cursor Ripples"))]
    pub cursor_ripples: bool,
    #[cfg_attr(feature="graphics", Setting(text="Cursor Ripple Color"))]
    pub cursor_ripple_color: SettingsColor,
    #[cfg_attr(feature="graphics", Setting(text="Cursor Ripple Scale"))]
    pub cursor_ripple_final_scale: f32,

    // skin settings
    #[cfg_attr(feature="graphics", Setting(text="Skin", dropdown="SkinDropdownable", category="Skin Settings"))]
    pub current_skin: String,

    // TODO
    #[serde(skip)]
    #[reflect(skip)]
    // #[cfg_attr(feature="graphics", Setting(text="Refresh Skins", action="SkinManager::refresh_skins()"))]
    refresh_skins_button: (),

    #[cfg_attr(feature="graphics", Setting(text="Theme", dropdown="SelectedTheme"))]
    pub theme: SelectedTheme,

    // #[Setting(text="UI Scale")] // not ready yet
    pub ui_scale: f64,
    #[cfg_attr(feature="graphics", Setting(text="Background Dim", min=0, max=1))]
    pub background_dim: f32,

    // misc keybinds
    #[cfg_attr(feature="graphics", Setting(text="User Panel Key", category="Common Keybinds"))]
    pub key_user_panel: Key,

    // double tap protection
    #[cfg_attr(feature="graphics", Setting(text="Enable DoubleTap Protection", category="DoubleTap Protection"))]
    pub enable_double_tap_protection: bool,
    #[cfg_attr(feature="graphics", Setting(text="DoubleTap Protection Leniency", min=10.0, max=200.0))]
    pub double_tap_protection_duration: f32,


    // integrations
    #[cfg_attr(feature="graphics", Subsetting(category="Integrations"))]
    pub integrations: IntegrationSettings,


    // other misc
    pub last_git_hash: String,
    pub external_games_folders: Vec<String>,
    
    #[cfg_attr(feature="graphics", Subsetting(category="Log Settings"))]
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


    pub async fn load(actions: &mut ActionQueue) -> Self {
        Self::load_from(SETTINGS_FILE, actions).await
    }
    pub async fn load_from(path: impl AsRef<Path>, actions: &mut ActionQueue) -> Self {
        let path = path.as_ref();

        let mut s = match std::fs::read_to_string(path).map(|s| serde_json::from_str::<Settings>(&s).map_err(|e| e.to_string())).map_err(|e| e.to_string()) {
            Ok(Ok(settings)) => settings,
            Err(e) | Ok(Err(e)) => {
                // NotificationManager::add_error_notification("Error reading settings.json\nLoading defaults", e).await;
                warn!("Error reading settings.json\nLoading defaults, {e}");
                if let Some(saved_as) = Self::backup_settings(path).await {
                    info!("Old settings saved to {saved_as}");
                }
                Self::default()
            }
        };
        s.save_path = path.to_string_lossy().to_string();

        // check password hashes
        s.check_hashes();
        
        GlobalValueManager::update(Arc::new(s.clone()));
        GlobalValueManager::update(Arc::new(WindowSize(s.display_settings.window_size.into())));

        // save after loading.
        // writes file if it doesnt exist, and writes new values from updates
        s.save(actions);
        s
    }

    pub  fn save(
        &self,
        actions: &mut ActionQueue,
    ) {
        debug!("Saving settings");
        let str = serde_json::to_string_pretty(self).unwrap();
        match std::fs::write(&self.save_path, str) {
            Ok(_) => trace!("settings saved successfully"),
            Err(e) => actions.push(GameAction::AddNotification(Notification::new_error("Error saving settings", e))),
        }
    }



    pub fn get_effect_vol(&self) -> f32 { self.effect_vol * self.master_vol }
    pub fn get_music_vol(&self) -> f32 { self.music_vol * self.master_vol }

    pub fn check_hashes(&mut self) {
        if self.osu_password.len() > 0 { self.osu_password = check_md5(self.osu_password.clone()) }
        if self.password.len() > 0 { self.password = check_sha512(self.password.clone()) }
    }

    // make a backup of the setting before they're overwritten (when the file fails to load)
    async fn backup_settings(settings_path: &Path) -> Option<String> {
        if !Io::exists(settings_path) { return None }
        let settings_path = settings_path.to_string_lossy().to_string();

        let mut counter = 0;
        let mut file = format!("{settings_path}.bak_{counter}");
        while Io::exists(&file) {
            counter += 1;
            file = format!("{settings_path}.bak_{counter}")
        }
        std::fs::copy(&settings_path, &file).expect("An error occurred while backing up the old settings.json");
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
            osu_settings: OsuSettings::default(),
            taiko_settings: TaikoSettings::default(),
            catch_settings: CatchSettings::default(),
            mania_settings: ManiaSettings::default(),
            background_game_settings: BackgroundGameSettings::default(),
            common_game_settings: CommonGameplaySettings::default(),
            last_played_mode: "osu".to_owned(),
            score_method: ScoreRetreivalMethod::Local,
            sort_by: SortBy::Title,
            beatmap_hitsounds: true,
            enable_diffcalc: true,


            // cursor
            cursor_scale: 1.0,
            cursor_border: 1.5,
            cursor_color: Color::from_hex("#ffff32".to_owned()).into(),
            cursor_border_color: Color::from_hex("#000".to_owned()).into(),
            cursor_ripples: true,
            cursor_ripple_color: Color::from_hex("#000".to_owned()).into(),
            cursor_ripple_final_scale: 1.5,
            
            // keybinds
            key_user_panel: Key::F8,

            // doubletap protection
            enable_double_tap_protection: false,
            double_tap_protection_duration: 80.0,
            
            // integrations
            integrations: Default::default(),

            display_settings: Default::default(),
            ui_scale: 1.0,
            background_dim: 0.8,

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


#[derive(Reflect)]
#[reflect(from_string = "auto")]
#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub enum ScoreRetreivalMethod {
    #[default]
    Local,
    LocalMods,
    Global,
    GlobalMods,

    OgGame,
    OgGameMods,
    // Friends,
    // FriendsMods
}
impl ScoreRetreivalMethod {
    pub fn list() -> Vec<Self> {
        vec![
            Self::Local,
            Self::LocalMods,
            
            Self::Global,
            Self::GlobalMods,

            Self::OgGame,
            Self::OgGameMods,
        ]
    }

    pub fn filter_by_mods(&self) -> bool {
        use ScoreRetreivalMethod::*;
        match self {
            Local 
            | OgGame 
            // | Friends
            | Global => false,

            LocalMods
            // | FriendsMods
            | OgGameMods
            | GlobalMods => true,
        }
    }
}
impl Display for ScoreRetreivalMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl TryFrom<&TatakuValue> for ScoreRetreivalMethod {
    type Error = String;
    fn try_from(value: &TatakuValue) -> Result<Self, Self::Error> {
        match value {
            TatakuValue::String(s) => {
                match &**s {
                    "Local" | "local" => Ok(Self::Local),
                    "LocalMods" | "local_mods" => Ok(Self::LocalMods),

                    "Global" | "global" => Ok(Self::Global),
                    "GlobalMods" | "global_mods" => Ok(Self::GlobalMods),

                    "OgGame" | "og_game" => Ok(Self::OgGame),
                    "OgGameMods" | "og_game_mods" => Ok(Self::OgGameMods),

                    other => Err(format!("invalid ScoreRetreivalMethod str: '{other}'"))
                }
            }
            TatakuValue::U64(n) => {
                match *n {
                    0 => Ok(Self::Local),
                    1 => Ok(Self::LocalMods),
                    2 => Ok(Self::Global),
                    3 => Ok(Self::GlobalMods),
                    4 => Ok(Self::OgGame),
                    5 => Ok(Self::OgGameMods),
                    other => Err(format!("Invalid ScoreRetreivalMethod number: {other}")),
                }
            }

            other => Err(format!("Invalid ScoreRetreivalMethod value: {other:?}"))
        }
    }
}
impl Into<TatakuValue> for ScoreRetreivalMethod {
    fn into(self) -> TatakuValue {
        TatakuValue::String(format!("{self:?}"))
    }
}

/// helper for colors inside settings
#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Reflect)]
#[reflect(from_string = "from_str")]
#[serde(from="Color", into="Color")]
pub struct SettingsColor {
    pub string: String,
    pub color: Color,
    pub valid: bool,
}
impl SettingsColor {
    pub fn update(&mut self, s: String) {
        if let Some(color) = Color::try_from_hex(&s) {
            self.color = color;
            self.valid = true;
        } else {
            self.valid = false;
        }

        self.string = s;
    }
}
impl PartialEq for SettingsColor {
    fn eq(&self, other: &Self) -> bool {
        self.color == other.color
    }
}
impl std::str::FromStr for SettingsColor {
    type Err = ReflectError<'static>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = Color::try_from_hex(s);
        Ok(Self {
            string: s.to_owned(),
            valid: c.is_some(),
            color: c.unwrap_or_default(),
        })
    }
}

impl From<Color> for SettingsColor {
    fn from(color: Color) -> Self {
        Self {
            string: color.to_hex(),
            color,
            valid: true,
        }
    }
}
impl From<SettingsColor> for Color {
    fn from(value: SettingsColor) -> Self {
        value.color
    }
}
impl Deref for SettingsColor {
    type Target = Color; 
    fn deref(&self) -> &Self::Target {
        &self.color
    }
}



//TODO: move this
lazy_static::lazy_static! {
    // TODO: change this to skin meta
    pub static ref AVAILABLE_SKINS:Arc<RwLock<Vec<String>>> = {
        let mut list = vec!["None".to_owned()];
        if let Ok(folder) = std::fs::read_dir(SKINS_FOLDER) {
            for f in folder.filter_map(|f| f.ok()) {
                list.push(f.file_name().to_string_lossy().to_string())
            }
        }
        Arc::new(RwLock::new(list))
    };
}
pub struct SkinDropdownable;
#[cfg(feature="graphics")]
impl Dropdownable2 for SkinDropdownable {
    type T = String;
    fn variants() -> Vec<String> {
        AVAILABLE_SKINS.read().clone() //.iter().map(|s|Self::Skin(s.clone())).collect()
    }
}