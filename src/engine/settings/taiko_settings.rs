use crate::prelude::*;
use tataku_client_proc_macros::Settings;

// TODO: TaikoPlayfield, TaikoNote::playfield_changed(&mut self, playfield:Arc<TaikoPlayfield>)


#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
#[derive(Settings)]
#[Setting(prefix="taiko_settings")]
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
    pub note_radius: f64,
    #[Setting(text="Big Note Scale", min=1, max=5)]
    pub big_note_multiplier: f64,

    /// hit area, but calculated before use
    #[serde(skip)]
    pub hit_position: Vector2,
    pub hit_position_relative_to_window_size: bool,
    pub hit_position_relative_height_div: f64,
    pub hit_position_offset: [f64; 2],

    /// hit area raidus multiplier, 1.0 = note radius
    #[Setting(text="Hit Area Radius Scale", min=1, max=5)]
    pub hit_area_radius_mult: f64,
    /// playfield = note_radius * max(hit_area_radius_mult, big_note_mult) + this
    #[Setting(text="Playfield Vertical Padding", min=0, max=20)]
    pub playfield_height_padding: f64,

    #[Setting(text="Don Color")]
    pub don_color_hex: String,
    #[Setting(text="Kat Color")]
    pub kat_color_hex: String,

    #[serde(skip)]
    pub don_color: Color,
    #[serde(skip)]
    pub kat_color: Color,

    #[Setting(text="Use Skin Judgments")]
    pub use_skin_judgments: bool,
    
    /// how far above the hit position should hit indicators be?
    #[Setting(text="Hit Judgment Y-Offset", min=0, max=100)]
    pub judgement_indicator_offset: f64,
}
impl TaikoSettings {
    pub async fn init_settings(&mut self) {
        // load hit_position
        let base = if self.hit_position_relative_to_window_size {
            let window_size = **WindowSize::get();
            window_size - Vector2::new(window_size.x, window_size.y / self.hit_position_relative_height_div) 
        } else {
            Vector2::ZERO
        };
        self.hit_position = base + Vector2::new(self.hit_position_offset[0], self.hit_position_offset[1]);

        // load colors
        self.don_color = Color::from_hex(&self.don_color_hex);
        self.kat_color = Color::from_hex(&self.kat_color_hex);
    }

    pub fn get_playfield(&self, width: f64, kiai: bool) -> Rectangle {
        let height = self.note_radius * self.big_note_multiplier * 2.0 + self.playfield_height_padding;
        Rectangle::new(
            [0.3, 0.3, 0.3, 1.0].into(),
            1002.0,
            Vector2::new(0.0, self.hit_position.y - height / 2.0),
            Vector2::new(width, height),
            if kiai {
                Some(Border::new(Color::YELLOW, 2.0))
            } else {None}
        )
    }
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
            hit_position: Vector2::ZERO,
            hit_position_relative_to_window_size: true,
            hit_position_relative_height_div: 1.375, // 3/8s the way down the screen
            hit_position_offset: [
                200.0,
                0.0
            ],
            don_color_hex: "#E74721".to_owned(),
            kat_color_hex: "#3797CA".to_owned(),
        
            don_color: Color::new(1.0, 0.0, 0.0, 1.0),
            kat_color: Color::new(0.0, 0.0, 1.0, 1.0),
            
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
    fn new_default(left_kat:u8, left_don:u8, right_don: u8, right_kat: u8) -> Self {
        Self {
            left_kat: ControllerInputConfig::new(Some(left_kat), None),
            left_don: ControllerInputConfig::new(Some(left_don), None),
            right_don: ControllerInputConfig::new(Some(right_don), None),
            right_kat: ControllerInputConfig::new(Some(right_kat), None)
        }
    }
    pub fn defaults(controller_name: Arc<String>) -> Self {
        match &**controller_name {
            "Taiko Controller"|"HORI CO.,LTD. Taiko Controller" => Self::new_default(6, 10, 11, 7),
            "Xbox Controller"|"Microsoft X-Box One S pad"|"Microsoft X-Box One pad"|"Microsoft XBox One X pad" => Self::new_default(13, 12, 0, 1),
            "Wireless Controller"|"Sony Interactive Entertainment Wireless Controller" => Self::new_default(17, 15, 0, 2),

            _ => Self {
                left_kat: ControllerInputConfig::new(None, None),
                left_don: ControllerInputConfig::new(None, None),
                right_don:ControllerInputConfig::new(None, None),
                right_kat:ControllerInputConfig::new(None, None),
            }
        }
    }
}
