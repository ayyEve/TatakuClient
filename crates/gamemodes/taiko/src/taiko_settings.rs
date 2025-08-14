use crate::prelude::*;
use tataku_client_proc_macros::Settings;

#[derive(Clone, Serialize, PartialEq, Debug)]
#[cfg_attr(feature="graphics", derive(Settings))]
#[derive(Reflect, SettingsDeserialize)]
#[serde(default)]
pub struct TaikoSettings {
    // input
    #[cfg_attr(feature="graphics", setting(text="Left Kat"))]
    pub left_kat: Key,
    #[cfg_attr(feature="graphics", setting(text="Left Don"))]
    pub left_don: Key,
    #[cfg_attr(feature="graphics", setting(text="Right Don"))]
    pub right_don: Key,
    #[cfg_attr(feature="graphics", setting(text="Right Kat"))]
    pub right_kat: Key,
    #[cfg_attr(feature="graphics", setting(text="Ignore Mouse Buttons"))]
    pub ignore_mouse_buttons: bool,
    #[reflect(skip)] // TaikoControllerConfig isnt reflectable
    pub controller_config: HashMap<ArcStr, TaikoControllerConfig>,

    // sv
    #[cfg_attr(feature="graphics", setting(text="SV Multiplier", min=1, max=2))]
    pub sv_multiplier: f32,

    // size stuff
    #[cfg_attr(feature="graphics", setting(text="Note Radius", min=1, max=100))]
    pub note_radius: f32,
    #[cfg_attr(feature="graphics", setting(text="Big Note Scale", min=1, max=5))]
    pub big_note_multiplier: f32,

    pub hit_position_relative_to_window_size: bool,
    pub hit_position_relative_height_div: f32,
    #[cfg_attr(feature="graphics", setting(text="Playfield Horizontal Offset", min=0, max=500))]
    pub playfield_x_offset: f32,
    #[cfg_attr(feature="graphics", setting(text="Playfield Vertical Offset", min=0, max=200))]
    pub playfield_y_offset: f32,

    /// hit area raidus multiplier, 1.0 = note radius
    #[cfg_attr(feature="graphics", setting(text="Hit Area Radius Scale", min=1, max=5))]
    pub hit_area_radius_mult: f32,
    /// playfield = note_radius * max(hit_area_radius_mult, big_note_mult) + this
    #[cfg_attr(feature="graphics", setting(text="Playfield Vertical Padding", min=0, max=20))]
    /// playfield = note_radius * max(hit_area_radius_mult, big_note_mult) + this
    pub playfield_height_padding: f32,

    #[cfg_attr(feature="graphics", setting(text="Don Color"))]
    pub don_color: SettingsColor,
    #[cfg_attr(feature="graphics", setting(text="Kat Color"))]
    pub kat_color: SettingsColor,

    #[cfg_attr(feature="graphics", setting(text="Use Skin Judgments"))]
    pub use_skin_judgments: bool,
    
    /// how far above the hit position should hit indicators be?
    #[cfg_attr(feature="graphics", setting(text="Hit Judgment Y-Offset", min=0, max=100))]
    pub judgement_indicator_offset: f32,
}
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
            hit_position_relative_to_window_size: true,
            hit_position_relative_height_div: 1.375, // 3/8s the way down the screen
            playfield_x_offset: 200.0,
            playfield_y_offset: 0.0,
        
            don_color: Color::from_hex("#E74721").into(),
            kat_color: Color::from_hex("#3797CA").into(),
            
            judgement_indicator_offset: 0.0,
            use_skin_judgments: true
        }
    }
}

impl GamemodeSettings for TaikoSettings {
    fn to_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
    fn duplicate_settings(&self) -> Box<dyn GamemodeSettings> {
        Box::new(self.clone())
    }
}

// #[cfg(feature = "gameplay")]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TaikoControllerConfig {
    pub left_kat: ControllerBinding,
    pub left_don: ControllerBinding,
    pub right_don: ControllerBinding,
    pub right_kat: ControllerBinding,
}
// #[cfg(feature = "gameplay")]
impl TaikoControllerConfig {
    fn new_default<I:Into<ControllerBinding>>(
        left_kat: I, 
        left_don: I, 
        right_don: I, 
        right_kat: I
    ) -> Self {
        Self {
            left_kat: left_kat.into(),
            left_don:  left_don.into(),
            right_don: right_don.into(),
            right_kat: right_kat.into()
        }
    }
    pub fn defaults(controller_name: ArcStr) -> Self {
        match &*controller_name {
            "Taiko Controller"
            | "HORI CO.,LTD. Taiko Controller"
            | "HID-compliant game controller" => Self::new_default(
                ControllerButton::LeftBumper, 
                ControllerButton::LeftThumb, 
                ControllerButton::RightThumb, 
                ControllerButton::RightBumper
            ),

            "Xbox Controller"
            | "Xbox One Game Controller" => Self::new_default(
                ControllerButton::DPadLeft, 
                ControllerButton::DPadDown, 
                ControllerButton::South, 
                ControllerButton::East
            ),

            // "Wireless Controller"
            // | "Sony Interactive Entertainment Wireless Controller" 
            //     => Self::new_default(17, 15, 0, 2),

            _ => Self::new_default(
                ControllerButton::LeftBumper, 
                ControllerButton::LeftThumb, 
                ControllerButton::RightThumb, 
                ControllerButton::RightBumper
            )
        }
    }
}
