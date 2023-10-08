use crate::prelude::*;
use tataku_client_proc_macros::Settings;

#[derive(Clone, Serialize, PartialEq)]
#[derive(Settings, SettingsDeserialize)]
#[serde(default)]
pub struct TaikoSettings {
    // input
    #[Setting(text="Left Kat")]
    pub left_kat: Key,
    #[Setting(text="Left Don")]
    pub left_don: Key,
    #[Setting(text="Right Don")]
    pub right_don: Key,
    #[Setting(text="Right Kat")]
    pub right_kat: Key,
    #[Setting(text="Ignore Mouse Buttons")]
    pub ignore_mouse_buttons: bool,
    pub controller_config: HashMap<String, TaikoControllerConfig>,

    // sv
    #[Setting(text="SV Multiplier", min=1, max=2)]
    pub sv_multiplier: f32,

    // size stuff
    #[Setting(text="Note Radius", min=1, max=100)]
    pub note_radius: f32,
    #[Setting(text="Big Note Scale", min=1, max=5)]
    pub big_note_multiplier: f32,

    // /// hit area, but calculated before use
    // #[serde(skip)]
    // pub hit_position: Vector2,
    pub hit_position_relative_to_window_size: bool,
    pub hit_position_relative_height_div: f32,
    #[Setting(text="Playfield Horizontal Offset", min=0, max=500)]
    pub playfield_x_offset: f32,
    #[Setting(text="Playfield Vertical Offset", min=0, max=200)]
    pub playfield_y_offset: f32,

    /// hit area raidus multiplier, 1.0 = note radius
    #[Setting(text="Hit Area Radius Scale", min=1, max=5)]
    pub hit_area_radius_mult: f32,
    /// playfield = note_radius * max(hit_area_radius_mult, big_note_mult) + this
    #[Setting(text="Playfield Vertical Padding", min=0, max=20)]
    pub playfield_height_padding: f32,
    /// playfield = note_radius * max(hit_area_radius_mult, big_note_mult) + this

    #[Setting(text="Don Color")]
    pub don_color: Color,
    #[Setting(text="Kat Color")]
    pub kat_color: Color,

    #[Setting(text="Use Skin Judgments")]
    pub use_skin_judgments: bool,
    
    /// how far above the hit position should hit indicators be?
    #[Setting(text="Hit Judgment Y-Offset", min=0, max=100)]
    pub judgement_indicator_offset: f32,
}
// impl TaikoSettings {
//     pub fn get_playfield(&self, width: f32, kiai: bool) -> Rectangle {
//         let height = self.note_radius * self.big_note_multiplier * 2.0 + self.playfield_height_padding;
//         Rectangle::new(
//             Vector2::new(0.0, self.hit_position.y - height / 2.0),
//             Vector2::new(width, height),
//             Color::new(0.1, 0.1, 0.1, 1.0),
//             if kiai {
//                 Some(Border::new(Color::YELLOW, 2.0))
//             } else {None}
//         )
//     }
// }
impl Default for TaikoSettings {
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
            // hit_position: Vector2::ZERO,
            hit_position_relative_to_window_size: true,
            hit_position_relative_height_div: 1.375, // 3/8s the way down the screen
            playfield_x_offset: 200.0,
            playfield_y_offset: 0.0,
        
            don_color: Color::from_hex("#E74721"),
            kat_color: Color::from_hex("#3797CA"),
            
            judgement_indicator_offset: 0.0,
            use_skin_judgments: true
        }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TaikoControllerConfig {
    pub left_kat: ControllerInputConfig,
    pub left_don: ControllerInputConfig,
    pub right_don: ControllerInputConfig,
    pub right_kat: ControllerInputConfig,
}
impl TaikoControllerConfig {
    fn new_default<I:Into<ControllerInputConfig>>(left_kat: I, left_don: I, right_don: I, right_kat: I) -> Self {
        Self {
            left_kat: left_kat.into(),
            left_don:  left_don.into(),
            right_don: right_don.into(),
            right_kat: right_kat.into()
        }
    }
    pub fn defaults(controller_name: Arc<String>) -> Self {
        match &**controller_name {
            "Taiko Controller"|"HORI CO.,LTD. Taiko Controller"|"HID-compliant game controller" => Self::new_default(ControllerButton::LeftTrigger2, ControllerButton::LeftThumb, ControllerButton::RightThumb, ControllerButton::RightTrigger2),
            "Xbox Controller"|"Xbox One Game Controller" => Self::new_default(ControllerButton::DPadLeft, ControllerButton::DPadDown, ControllerButton::South, ControllerButton::East),
            // "Wireless Controller"|"Sony Interactive Entertainment Wireless Controller" => Self::new_default(17, 15, 0, 2),

            _ => Self::new_default(ControllerButton::LeftTrigger2, ControllerButton::LeftThumb, ControllerButton::RightThumb, ControllerButton::RightTrigger2)
            // _ => Self {
            //     left_kat: ControllerInputConfig::new(None, None),
            //     left_don: ControllerInputConfig::new(None, None),
            //     right_don:ControllerInputConfig::new(None, None),
            //     right_kat:ControllerInputConfig::new(None, None),
            // }
        }
    }
}
