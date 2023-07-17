use crate::prelude::*;


pub const LEADERBOARD_PADDING:f32 = 100.0;
pub const LEADERBOARD_POS:Vector2 = Vector2::new(10.0, LEADERBOARD_PADDING);
pub const LEADERBOARD_ITEM_SIZE:Vector2 = Vector2::new(200.0, 50.0);


#[derive(ScrollableGettersSetters)]
pub struct LeaderboardItem {
    pos: Vector2,
    size: Vector2,
    hover: bool,
    selected: bool,
    tag: String,

    score: IngameScore,
    font: Font,

    score_mods: String,

    ui_scale: Vector2,

    pub color_override: Option<Color>,
    pub text_color_override: Option<Color>,
    pub image: Option<Image>,
    pub theme: ThemeHelper,
}
impl LeaderboardItem {
    pub fn new(score:IngameScore) -> LeaderboardItem {
        let pos = Vector2::ZERO;
        let size = LEADERBOARD_ITEM_SIZE;

        let tag = score.hash(); //username.clone();
        let font = Font::Main;
        let score_mods = ModManager::short_mods_string(score.mods(), false, &score.playmode);

        LeaderboardItem {
            pos,
            size,
            score,
            tag,
            hover: false,
            selected: false,
            font,
            score_mods,
            ui_scale: Vector2::ONE,

            color_override: None,
            text_color_override: None,
            image: None,
            theme: ThemeHelper::new(),
        }
    }
    pub async fn load_image(mut self) -> Self {
        self.image = SkinManager::get_texture("menu-button-background", true).await;
        self
    }
}
impl ScrollableItem for LeaderboardItem {
    fn ui_scale_changed(&mut self, scale: Vector2) {
        self.ui_scale = scale;
        self.size = LEADERBOARD_ITEM_SIZE * scale;
    }
    fn update(&mut self) {
        self.theme.update();
    }

    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
        const PADDING:Vector2 = Vector2::new(5.0, 5.0);

        let now = chrono::Utc::now().timestamp() as u64;
        let time_diff = now as i64 - self.score.time as i64;
        let time_diff_str = if time_diff < 60 * 5 {
            format!(" | {time_diff}s")
        } else {
            String::new()
        };
        
        let color = if let Some(color) = self.color_override {
            color
        } else if self.selected {
            self.theme.get_color(ThemeColor::LeaderboardSelect).unwrap_or(Color::BLUE)
        } else if self.hover {
            self.theme.get_color(ThemeColor::LeaderboardHover).unwrap_or(Color::RED)
        } else {
            self.theme.get_color(ThemeColor::LeaderboardBg).unwrap_or(Color::WHITE)
        };

        let text_color = if let Some(color) = self.text_color_override {
            color
        } else if self.selected {
            self.theme.get_color(ThemeColor::LeaderboardTextSelected).unwrap_or(Color::WHITE)
        } else if self.hover {
            self.theme.get_color(ThemeColor::LeaderboardTextHovered).unwrap_or(Color::WHITE)
        } else {
            self.theme.get_color(ThemeColor::LeaderboardText).unwrap_or(Color::WHITE)
        };

        if let Some(mut img) = self.image.clone() {
            img.pos = self.pos;
            img.origin = Vector2::ZERO;
            img.color = color;
            img.set_size(LEADERBOARD_ITEM_SIZE * self.ui_scale);

            list.push(img)
        } else {
            // bounding rect
            list.push(Rectangle::new(
                self.pos + pos_offset,
                LEADERBOARD_ITEM_SIZE * self.ui_scale,
                Color::new(0.2, 0.2, 0.2, 1.0),
                Some(Border::new(color, 1.5 * self.ui_scale.y))
            ).shape(Shape::Round(5.0)));
        }


        // score text
        list.push(Text::new(
            self.pos + pos_offset + PADDING * self.ui_scale,
            15.0 * self.ui_scale.y,
            format!("{}: {}", self.score.username, crate::format_number(self.score.score.score)),
            text_color,
            self.font.clone()
        ));

        // combo text
        list.push(Text::new(
            self.pos + pos_offset + (PADDING + Vector2::new(0.0, PADDING.y + 15.0)) * self.ui_scale,
            12.0 * self.ui_scale.y,
            format!("{}x, {:.2}%, {}{time_diff_str}", crate::format_number(self.score.max_combo), calc_acc(&self.score) * 100.0, self.score_mods),
            text_color,
            self.font.clone()
        ));
    }
}
