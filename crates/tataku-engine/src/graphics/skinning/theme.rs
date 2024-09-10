use crate::prelude::*;

#[derive(Reflect)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Theme {
    name: String,

    colors: HashMap<ThemeColor, Color>,

    // TODO: impl Reflect on Vector2
    #[reflect(skip)]
    positions: HashMap<ThemePosition, Vector2>,
    #[reflect(skip)]
    scales: HashMap<ThemeScale, Vector2>,
}
impl Theme {
    pub fn get_color(&self, color: ThemeColor) -> Option<Color> {
        self.colors.get(&color).cloned()
    }
    pub fn get_pos(&self, pos: ThemePosition) -> Option<Vector2> {
        self.positions.get(&pos).cloned()
    }
    pub fn get_scale(&self, scale: ThemeScale) -> Option<Vector2> {
        self.scales.get(&scale).cloned()
    }
}
impl Default for Theme {
    fn default() -> Self {
        tataku_theme()
    }
}

// create_value_helper!(CurrentTheme, Theme, ThemeHelper);


#[allow(unused)]
#[derive(Reflect)]
#[reflect(from_string = "auto")]
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
    BeatmapSelectTextHovered,
    BeatmapSelectTextSelected,
    
    // leaderboard
    LeaderboardBg,
    LeaderboardHover,
    LeaderboardSelect,
    LeaderboardText,
    LeaderboardTextHovered,
    LeaderboardTextSelected,

    LeaderboardCurrentScore,
    LeaderboardPreviousScores,
    LeaderboardPreviousBest,
}
impl ToString for ThemeColor {
    fn to_string(&self) -> String {
        format!("{self:?}")
    }
}

#[allow(unused)]
#[derive(Reflect)]
#[reflect(from_string = "auto")]
#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Copy, Clone)]
pub enum ThemePosition {
    // beatmap select
    BeatmapSelectSetSelectedOffset,
    BeatmapSelectSetHoveredOffset,
    BeatmapSelectSetOffset,

    BeatmapSelectMapSelectedOffset,
    BeatmapSelectMapHoveredOffset,
    BeatmapSelectMapOffset,
}
impl ToString for ThemePosition {
    fn to_string(&self) -> String {
        format!("{self:?}")
    }
}

#[allow(unused)]
#[derive(Reflect)]
#[reflect(from_string = "auto")]
#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Copy, Clone)]
pub enum ThemeScale {
    // beatmap select
    BeatmapSelectSetSelectedScale,
    BeatmapSelectSetHoveredScale,
    BeatmapSelectSetScale,

    BeatmapSelectMapSelectedScale,
    BeatmapSelectMapHoveredScale,
    BeatmapSelectMapScale,
}
impl ToString for ThemeScale {
    fn to_string(&self) -> String {
        format!("{self:?}")
    }
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

    use ThemePosition::*;
    let positions = [
        // beatmap select
        (BeatmapSelectSetSelectedOffset, Vector2::ZERO),
        (BeatmapSelectSetHoveredOffset, Vector2::ZERO),
    ].into_iter().collect::<HashMap<ThemePosition, Vector2>>();

    use ThemeScale::*;
    let scales = [
        // beatmap select
        (BeatmapSelectSetSelectedScale, Vector2::ONE),
        (BeatmapSelectSetHoveredScale, Vector2::ONE),
    ].into_iter().collect::<HashMap<ThemeScale, Vector2>>();


    Theme {
        name, 
        colors,
        scales,
        positions
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
        (ThemeColor::BeatmapSelectTextHovered, Color::BLACK),
        (ThemeColor::BeatmapSelectTextSelected, Color::BLACK),

        // leaderboard
        (ThemeColor::LeaderboardBg, Color::BLACK.alpha(0.5)),
        (ThemeColor::LeaderboardHover, lighten.alpha(0.5)),
        (ThemeColor::LeaderboardSelect, lighten.alpha(0.5)),
        (ThemeColor::LeaderboardText, Color::WHITE),
        (ThemeColor::LeaderboardTextHovered, Color::WHITE),
        (ThemeColor::LeaderboardTextSelected, Color::WHITE),

        (ThemeColor::LeaderboardPreviousBest, col([255, 69, 0, 150])),
        (ThemeColor::LeaderboardCurrentScore, Color::BLACK.alpha(0.5)),

    ].into_iter().collect::<HashMap<ThemeColor, Color>>();

    
    use ThemePosition::*;
    let positions = [
        // beatmap select
        (BeatmapSelectSetSelectedOffset, Vector2::ZERO),
        (BeatmapSelectSetHoveredOffset, Vector2::ZERO),
        (BeatmapSelectSetOffset, Vector2::with_x(20.0)),

        // (BeatmapSelectMapSelectedOffset, Vector2::with_x(-20.0)),
        // (BeatmapSelectMapHoveredOffset, Vector2::with_x(-20.0)),
        // (BeatmapSelectMapOffset, Vector2::with_x(-20.0)),
    ].into_iter().collect::<HashMap<ThemePosition, Vector2>>();

    
    use ThemeScale::*;
    let scales = [
        // beatmap select
        (BeatmapSelectSetSelectedScale, Vector2::ONE * 1.4),
        (BeatmapSelectSetHoveredScale, Vector2::ONE * 1.4),

        (BeatmapSelectMapSelectedScale, Vector2::ONE * 1.4),
        (BeatmapSelectMapHoveredScale, Vector2::ONE * 1.4),
        (BeatmapSelectMapScale, Vector2::ONE * 1.4),
    ].into_iter().collect::<HashMap<ThemeScale, Vector2>>();

    Theme {
        name, 
        colors,
        scales,
        positions
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

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
#[derive(Reflect)]
pub enum SelectedTheme {
    Tataku,
    Osu,
    /// path to theme file, name of theme
    Custom(String, String),
}


#[cfg(feature="graphics")]
impl tataku_client_common::Dropdownable2 for SelectedTheme {
    type T = Self;
    fn variants() -> Vec<Self::T> {
        [Self::Tataku, Self::Osu].into_iter().chain(THEMES.clone().into_iter().map(|t|Self::Custom(t.0, t.1))).collect()
    }

    // fn display_text(&self) -> String {
    //     match self {
    //         Self::Tataku => "Tataku".to_owned(),
    //         Self::Osu => "Osu".to_owned(),
    //         Self::Custom(_, name) => name.clone()
    //     }
    // }

    // fn from_string(s:String) -> Self {
    //     match &*s {
    //         "Tataku" => Self::Tataku,
    //         "Osu" => Self::Osu,
    //         _ => {
    //             let t = THEMES.iter().find(|t|t.1 == s).unwrap().clone();
    //             Self::Custom(t.0, t.1)
    //         }
    //     }
    // }
}

impl Display for SelectedTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tataku => write!(f, "Tataku"),
            Self::Osu => write!(f, "Osu"),
            Self::Custom(_, name) => write!(f, "{name}"),
        }
    }
}