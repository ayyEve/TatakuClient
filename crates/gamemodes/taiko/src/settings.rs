use crate::prelude::*;
use common::reflect::*;
use tataku::Color;
use engine::{
    input::{
        Key,
        GamepadButton,
        ControllerInputBinding,
    },
    gameplay::GamemodeSettings,
    settings::SettingsColor,
};
use tataku_client_proc_macros::Settings;

#[derive(Reflect, Settings)]
#[derive(Clone, PartialEq, Debug)]
#[derive(Serialize, DeserializeSettings)]
#[serde(default)]
pub struct Settings {
    // input
    #[setting(text="Left Kat")]
    pub left_kat: Key,
    #[setting(text="Left Don")]
    pub left_don: Key,
    #[setting(text="Right Don")]
    pub right_don: Key,
    #[setting(text="Right Kat")]
    pub right_kat: Key,
    #[setting(text="Ignore Mouse Buttons")]
    pub ignore_mouse_buttons: bool,
    #[reflect(skip)] // TaikoControllerConfig isnt reflectable
    pub controller_config: HashMap<ArcStr, TaikoControllerConfig>,

    // sv
    #[setting(text="SV Multiplier", range(1.0, 2.0))]
    pub sv_multiplier: f32,

    // size stuff
    #[setting(text="Note Radius", range(1.0, 100.0))]
    pub note_radius: f32,
    #[setting(text="Big Note Scale", range(1.0, 5.0))]
    pub big_note_multiplier: f32,

    #[setting(text="Playfield Horizontal Fractional Position", range(0.0, 1.0))]
    pub playfield_x_pos: f32,
    #[setting(text="Playfield Vertical Fractional Position", range(0.0, 1.0))]
    pub playfield_y_pos: f32,

    /// hit area raidus multiplier, 1.0 = note radius
    #[setting(text="Hit Area Radius Scale", range(1.0, 5.0))]
    pub hit_area_radius_mult: f32,

    /// playfield = note_radius * max(hit_area_radius_mult, big_note_mult) + this
    #[setting(text="Playfield Vertical Padding", range(0.0, 20.0))]
    pub playfield_height_padding: f32,

    #[setting(text="Don Color")]
    pub don_color: SettingsColor,
    #[setting(text="Kat Color")]
    pub kat_color: SettingsColor,

    #[setting(text="Use Skin Judgments")]
    pub use_skin_judgments: bool,

    /// how far above the hit position should hit indicators be?
    #[setting(text="Hit Judgment Y-Offset", range(0.0, 100.0))]
    pub judgement_indicator_offset: f32,

    #[setting(text="Left Kat (Gamepad)")]
    pub gamepad_left_kat: Option<GamepadButton>,
    #[setting(text="Left don (Gamepad)")]
    pub gamepad_left_don: Option<GamepadButton>,
    #[setting(text="Right don (Gamepad)")]
    pub gamepad_right_don: Option<GamepadButton>,
    #[setting(text="Right Kat (Gamepad)")]
    pub gamepad_right_kat: Option<GamepadButton>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            // input
            left_kat: Key::D,
            left_don: Key::F,
            right_don: Key::J,
            right_kat: Key::K,
            ignore_mouse_buttons: false,
            controller_config: HashMap::new(),

            // sv
            sv_multiplier: 1.1,

            // size stuff
            note_radius: 42.0,
            big_note_multiplier: 1.666666,
            hit_area_radius_mult: 1.2,
            playfield_height_padding: 8.0,
            // hit area stuff
            playfield_x_pos: 0.2,
            playfield_y_pos: 3.0 / 8.0,

            don_color: Color::from_hex("#E74721").into(),
            kat_color: Color::from_hex("#3797CA").into(),

            judgement_indicator_offset: 0.0,
            use_skin_judgments: true,

            gamepad_left_don: None,
            gamepad_left_kat: None,
            gamepad_right_don: None,
            gamepad_right_kat: None,
        }
    }
}

impl GamemodeSettings for Settings {
    fn to_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
    fn duplicate_settings(&self) -> Box<dyn GamemodeSettings> {
        Box::new(self.clone())
    }
}

// #[cfg(feature = "gameplay")]
#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug, PartialEq)]
pub struct TaikoControllerConfig {
    pub left_kat: ControllerInputBinding,
    pub left_don: ControllerInputBinding,
    pub right_don: ControllerInputBinding,
    pub right_kat: ControllerInputBinding,
}
// #[cfg(feature = "gameplay")]
impl TaikoControllerConfig {
    fn new_default<I:Into<ControllerInputBinding>>(
        left_kat: I,
        left_don: I,
        right_don: I,
        right_kat: I
    ) -> Self {
        Self {
            left_kat: left_kat.into(),
            left_don: left_don.into(),
            right_don: right_don.into(),
            right_kat: right_kat.into()
        }
    }
    pub fn defaults(controller_name: &str) -> Self {
        match controller_name {
            "Taiko Controller"
            | "HORI CO.,LTD. Taiko Controller"
            | "HID-compliant game controller" => Self::new_default(
                GamepadButton::LeftBumper,
                GamepadButton::LeftThumb,
                GamepadButton::RightThumb,
                GamepadButton::RightBumper
            ),

            "Xbox Controller"
            | "Xbox One Game Controller" => Self::new_default(
                GamepadButton::DPadLeft,
                GamepadButton::DPadDown,
                GamepadButton::South,
                GamepadButton::East
            ),

            // "Wireless Controller"
            // | "Sony Interactive Entertainment Wireless Controller"
            //     => Self::new_default(17, 15, 0, 2),

            _ => Self::new_default(
                GamepadButton::DPadLeft,
                GamepadButton::DPadDown,
                GamepadButton::South,
                GamepadButton::East
            )
        }
    }
}


// #[test]
// fn test() {
//     let mut settings = engine::Settings::default();
//     settings.save_path = "/tmp/test.json".into();

//     let infos = engine::gameplay::GamemodeInfos::new(vec![crate::GAME_INFO]);
//     settings.gamemode_settings.build(infos.clone());


//     let gamemode = crate::GAME_INFO.id;

//     let mut t_settings = settings
//         .gamemode_settings::<TaikoSettings>(gamemode)
//         .unwrap_or_default();

//     assert_eq!(t_settings.left_don, TaikoSettings::default().left_don, "default test");

//     t_settings.left_don = Key::Calculator;
//     settings.update_gamemode_settings(gamemode, t_settings);

//     {
//         let t_settings = settings
//             .gamemode_settings::<TaikoSettings>(gamemode)
//             .unwrap();

//         assert_eq!(t_settings.left_don, Key::Calculator, "update test");
//     }

//     settings.save();

//     {
//         let mut settings = engine::Settings::load_from(&settings.save_path);
//         settings.gamemode_settings.build(infos);
//         let gamemode = crate::GAME_INFO.id;

//         let t_settings = settings
//             .gamemode_settings::<TaikoSettings>(gamemode)
//             .unwrap();

//         assert_eq!(t_settings.left_don, Key::Calculator, "save test");
//     }
// }
