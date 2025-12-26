use crate::*;

#[derive(Default2)]
pub struct GeneralUiTheme {
    #[default(Color::BLACK.with_alpha_f32(0.8))]
    pub background_color: Color,
    
    #[default(Color::WHITE)]
    pub default_color: Color,

    #[default(Color::CYAN)]
    pub hover_color: Color,

    #[default(Color::YELLOW)]
    pub active_color: Color,
}
impl GeneralUiTheme {
    pub fn get_color(&self, active: bool, hover: bool) -> Color {
        if active {
            self.active_color
        } else if hover {
            self.hover_color
        } else {
            self.default_color
        }
    }
}
