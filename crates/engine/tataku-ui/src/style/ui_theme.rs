use crate::prelude::*;


pub struct GeneralUiTheme {
    pub background_color: Color,
    pub default_color: Color,
    pub hover_color: Color,
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
impl Default for GeneralUiTheme {
    fn default() -> Self {
        Self {
            background_color: Color::BLACK.alpha(0.8),
            default_color: Color::WHITE,
            hover_color: Color::CYAN,
            active_color: Color::YELLOW,
        }
    }
}

