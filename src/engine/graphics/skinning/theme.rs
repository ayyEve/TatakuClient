use crate::{prelude::*, create_value_helper};

#[derive(Debug, Serialize, Deserialize)]
pub struct Theme {
    name: String,
    colors: HashMap<ThemeColor, Color>
}
impl Theme {
    pub fn get_color(&self, color: ThemeColor) -> Option<Color> {
        self.colors.get(&color).cloned()
    }
}

impl Default for Theme {
    fn default() -> Self {
        tataku_theme()
    }
}

create_value_helper!(CurrentTheme, Theme, ThemeHelper);


#[allow(unused)]
#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Copy, Clone)]
pub enum ThemeColor {

    // main menu
    MainMenuPrimary,
    MainMenuSecondary,

    // beatmap select
    BeatmapSelectSetBg,
    BeatmapSelectSetHover,
    BeatmapSelectSetSelect,
    BeatmapSelectMapBg,
    BeatmapSelectMapHover,
    BeatmapSelectMapSelect,
    BeatmapSelectText,
    BeatmapSelectTextSelected,

}




pub fn tataku_theme() -> Theme {
    let name = "Tataku".to_owned();
    let colors = [
        // main menu
        (ThemeColor::MainMenuPrimary, Color::WHITE),
        (ThemeColor::MainMenuSecondary, Color::new(1.0, 1.0, 1.0, 0.1)),
        
        // beatmap select
        (ThemeColor::BeatmapSelectSetBg, Color::new(0.2, 0.2, 0.2, 1.0)),
        (ThemeColor::BeatmapSelectSetHover, Color::BLUE),
        (ThemeColor::BeatmapSelectSetSelect, Color::RED),
        (ThemeColor::BeatmapSelectMapBg, Color::new(0.2, 0.2, 0.2, 1.0)),
        (ThemeColor::BeatmapSelectMapHover, Color::BLUE),
        (ThemeColor::BeatmapSelectMapSelect, Color::RED),
        (ThemeColor::BeatmapSelectText, Color::WHITE),

    ].into_iter().collect::<HashMap<ThemeColor, Color>>();

    Theme {
        name, 
        colors
    }
}

pub fn osu_theme() -> Theme {
    let name = "Osu".to_owned();
    let pink = col([235, 73, 153, 240]);
    let white = col([255, 255, 255, 220]);
    let blue = col([0, 150, 236, 240]);

    let lighten = 0.3;
    let lighten = Color::new(lighten, lighten, lighten, 1.0);
    
    let colors = [
        // main menu
        (ThemeColor::MainMenuPrimary, Color::WHITE),
        (ThemeColor::MainMenuSecondary, Color::new(1.0, 1.0, 1.0, 0.1)),

        // beatmap select
        (ThemeColor::BeatmapSelectSetBg, pink),
        (ThemeColor::BeatmapSelectSetHover, pink / lighten),
        (ThemeColor::BeatmapSelectSetSelect, white),
        (ThemeColor::BeatmapSelectMapBg, blue),
        (ThemeColor::BeatmapSelectMapHover, blue / lighten),
        (ThemeColor::BeatmapSelectMapSelect, white),
        (ThemeColor::BeatmapSelectText, Color::WHITE),
        (ThemeColor::BeatmapSelectTextSelected, Color::BLACK),

    ].into_iter().collect::<HashMap<ThemeColor, Color>>();

    Theme {
        name, 
        colors
    }
}


fn col(b:[u8; 4]) -> Color {
    Color::new(
        b[0] as f32 / 255.0, 
        b[1] as f32 / 255.0, 
        b[2] as f32 / 255.0, 
        b[3] as f32 / 255.0
    )
}



lazy_static::lazy_static! {
    static ref THEMES: Vec<(String, String)> = {
        Vec::new()
    };
}

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum SelectedTheme {
    Tataku,
    Osu,
    /// path to theme file, name of theme
    Custom(String, String),
}


impl Dropdownable for SelectedTheme {
    fn variants() -> Vec<Self> {
        [Self::Tataku, Self::Osu].into_iter().chain(THEMES.clone().into_iter().map(|t|Self::Custom(t.0, t.1))).collect()
    }

    fn display_text(&self) -> String {
        match self {
            Self::Tataku => "Tataku".to_owned(),
            Self::Osu => "Osu".to_owned(),
            Self::Custom(_, name) => name.clone()
        }
    }

    fn from_string(s:String) -> Self {
        match &*s {
            "Tataku" => Self::Tataku,
            "Osu" => Self::Osu,
            _ => {
                let t = THEMES.iter().find(|t|t.1 == s).unwrap().clone();
                Self::Custom(t.0, t.1)
            }
        }
    }
}