#![allow(clippy::to_string_trait_impl, reason = "lazy")]
use crate::*;
use common::reflect::*;

// FIXME: literally all of this. it was an idea and it should have stayed that way

#[derive(Reflect)]
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Theme {
    name: String,
    #[reflect(skip)]
    colors: HashMap<ThemeColor, Color>,

    // TODO: impl Reflect on Vector2
    #[reflect(skip)]
    positions: HashMap<ThemePosition, Vector2>,
    #[reflect(skip)]
    scales: HashMap<ThemeScale, Vector2>,
}
impl Theme {
    pub fn get_color(&self, color: ThemeColor) -> Option<Color> {
        self.colors.get(&color).copied()
    }
    pub fn get_pos(&self, pos: ThemePosition) -> Option<Vector2> {
        self.positions.get(&pos).copied()
    }
    pub fn get_scale(&self, scale: ThemeScale) -> Option<Vector2> {
        self.scales.get(&scale).copied()
    }
}
impl Default for Theme {
    fn default() -> Self {
        tataku_theme()
    }
}

#[allow(unused)]
#[derive(Reflect, FromStr)]
#[derive(Serialize, Deserialize)]
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
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
#[derive(Reflect, FromStr)]
#[derive(Serialize, Deserialize)]
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
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
#[derive(Reflect, FromStr)]
#[derive(Serialize, Deserialize)]
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
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
        (ThemeColor::MainMenuSecondary, Color::new_f32(1.0, 1.0, 1.0, 0.1)),
        
        // beatmap select
        (ThemeColor::BeatmapSelectSetBg, Color::new_f32(0.2, 0.2, 0.2, 1.0)),
        (ThemeColor::BeatmapSelectSetHover, Color::BLUE),
        (ThemeColor::BeatmapSelectSetSelect, Color::RED),
        (ThemeColor::BeatmapSelectMapBg, Color::new_f32(0.2, 0.2, 0.2, 1.0)),
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
    let pink = Color::new_rgba8(235, 73, 153, 240);
    let white = Color::new_rgba8(255, 255, 255, 220);
    let blue = Color::new_rgba8(0, 150, 236, 240);

    let lighten = 0.3;
    let lighten = Color::new_f32(lighten, lighten, lighten, 1.0);
    
    let colors = [
        // main menu
        (ThemeColor::MainMenuPrimary, Color::WHITE),
        (ThemeColor::MainMenuSecondary, Color::new_f32(1.0, 1.0, 1.0, 0.1)),

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
        (ThemeColor::LeaderboardBg, Color::BLACK.with_alpha_f32(0.5)),
        (ThemeColor::LeaderboardHover, lighten.with_alpha_f32(0.5)),
        (ThemeColor::LeaderboardSelect, lighten.with_alpha_f32(0.5)),
        (ThemeColor::LeaderboardText, Color::WHITE),
        (ThemeColor::LeaderboardTextHovered, Color::WHITE),
        (ThemeColor::LeaderboardTextSelected, Color::WHITE),

        (ThemeColor::LeaderboardPreviousBest, Color::new_rgba8(255, 69, 0, 150)),
        (ThemeColor::LeaderboardCurrentScore, Color::BLACK.with_alpha_f32(0.5)),

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
