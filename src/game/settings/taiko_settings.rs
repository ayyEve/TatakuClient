use crate::prelude::*;


#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TaikoSettings {
    // input
    pub left_kat: Key,
    pub left_don: Key,
    pub right_don: Key,
    pub right_kat: Key,
    pub ignore_mouse_buttons: bool,
    pub controller_config: HashMap<String, TaikoControllerConfig>,

    // sv
    pub static_sv: bool,
    pub sv_multiplier: f32,

    // size stuff
    pub note_radius: f64,
    pub big_note_multiplier: f64,

    /// hit area, but calculated before use
    #[serde(skip)]
    pub hit_position: Vector2,
    pub hit_position_relative_to_window_size: bool,
    pub hit_position_relative_height_div: f64,
    pub hit_position_offset: [f64; 2],

    /// hit area raidus multiplier, 1.0 = note radius
    pub hit_area_radius_mult: f64,
    /// playfield = note_radius * max(hit_area_radius_mult, big_note_mult) + this
    pub playfield_height_padding: f64,

    pub don_color_hex: String,
    pub kat_color_hex: String,

    #[serde(skip)]
    pub don_color: Color,
    #[serde(skip)]
    pub kat_color: Color,

    
    /// how far above the hit position should hit indicators be?
    pub judgement_indicator_offset: f64,
}
impl TaikoSettings {
    pub fn init_settings(&mut self) {
        // load hit_position
        let base = if self.hit_position_relative_to_window_size {
            let window_size = Settings::window_size();
            window_size - Vector2::new(window_size.x, window_size.y / self.hit_position_relative_height_div) 
        } else {
            Vector2::zero()
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
            static_sv: false,
            sv_multiplier: 1.1,
            
            // size stuff
            note_radius: 42.0,
            big_note_multiplier: 1.666666,
            hit_area_radius_mult: 1.2,
            playfield_height_padding: 8.0,
            // hit area stuff
            hit_position: Vector2::zero(),
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
            
            judgement_indicator_offset: 0.0
        }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
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
