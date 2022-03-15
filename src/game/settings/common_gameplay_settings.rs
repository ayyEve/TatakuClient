use crate::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct CommonGameplaySettings {
    pub key_offset_up: Key,
    pub key_offset_down: Key,

    // duration settings
    /// color of duration to go (bg)
    pub duration_color_hex: String, 
    #[serde(skip)]
    pub duration_color: Color,
    /// color of duration completed
    pub duration_color_full_hex: String,
    #[serde(skip)]
    pub duration_color_full: Color,
    /// color of duration border
    pub duration_border_color_hex: String,
    #[serde(skip)]
    pub duration_border_color: Color,


    // health bar
    /// colors for healthbar at %s
    /// ie [0%-full], [0%-50%, 50%-full], or [0%-33%, 33%-66%, 66%-full], etc
    pub healthbar_colors_hex: Vec<String>, 
    #[serde(skip)]
    pub healthbar_colors: Vec<Color>,
    /// color of healthbar background
    pub healthbar_bg_color_hex: String,
    #[serde(skip)]
    pub healthbar_bg_color: Color,
    /// color of healthbar border
    pub healthbar_border_color_hex: String,
    #[serde(skip)]
    pub healthbar_border_color: Color,
}
impl CommonGameplaySettings {
    /// init colors etc
    pub fn init(mut self) -> Self {
        // duration colors
        self.duration_color = Color::from_hex(&self.duration_color_hex);
        self.duration_color_full = Color::from_hex(&self.duration_color_full_hex);
        self.duration_border_color = Color::from_hex(&self.duration_border_color_hex);

        // healthbar
        self.healthbar_colors = self.healthbar_colors_hex.iter().map(|c|Color::from_hex(c)).collect();
        self.healthbar_bg_color = Color::from_hex(&self.healthbar_bg_color_hex);
        self.healthbar_border_color = Color::from_hex(&self.healthbar_border_color_hex);

        self
    }
}

impl Default for CommonGameplaySettings {
    fn default() -> Self {
        Self { 
            key_offset_up: Key::Equals,
            key_offset_down: Key::Minus,

            // duration
            duration_color_hex: "#66666680".to_owned(),
            duration_color_full_hex: "#666F".to_owned(),
            duration_border_color_hex: "#000".to_owned(),
            
            duration_color: Color::WHITE,
            duration_color_full: Color::WHITE,
            duration_border_color: Color::WHITE,

            // healthbar
            healthbar_colors_hex: vec!["#0F0".to_owned()],
            healthbar_bg_color_hex: "#66666680".to_owned(),
            healthbar_border_color_hex: "#000".to_owned(),

            healthbar_colors: vec![Color::WHITE],
            healthbar_bg_color: Color::WHITE,
            healthbar_border_color: Color::WHITE,
        }
    }
}