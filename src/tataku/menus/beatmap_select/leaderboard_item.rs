use crate::prelude::*;


pub const LEADERBOARD_PADDING:f64 = 100.0;
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
    font: Font2,

    score_mods: String,

    ui_scale: Vector2,
}
impl LeaderboardItem {
    pub fn new(score:IngameScore) -> LeaderboardItem {
        let tag = score.hash(); //username.clone();
        let font = get_font();
        let score_mods = ModManager::short_mods_string(score.mods(), false, &score.playmode);

        LeaderboardItem {
            pos: Vector2::zero(),
            size: LEADERBOARD_ITEM_SIZE,
            score,
            tag,
            hover: false,
            selected: false,
            font,
            score_mods,
            ui_scale: Vector2::one()
        }
    }
}
impl ScrollableItem for LeaderboardItem {
    fn ui_scale_changed(&mut self, scale: Vector2) {
        self.ui_scale = scale;
        self.size = LEADERBOARD_ITEM_SIZE * scale;
    }

    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2, parent_depth:f64, list: &mut RenderableCollection) {
        const PADDING:Vector2 = Vector2::new(5.0, 5.0);

        let now = chrono::Utc::now().timestamp() as u64;
        let time_diff = now as i64 - self.score.time as i64;
        let time_diff_str = if time_diff < 60 * 5 {
            format!(" | {time_diff}s")
        } else {
            String::new()
        };

        // bounding rect
        list.push(Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            parent_depth + 5.0,
            self.pos + pos_offset,
            LEADERBOARD_ITEM_SIZE * self.ui_scale,
            self.get_border_none(1.0)
        ).shape(Shape::Round(5.0, 10)));

        // score text
        list.push(Text::new(
            Color::WHITE,
            parent_depth + 4.0,
            self.pos + pos_offset + PADDING * self.ui_scale,
            (15.0 * self.ui_scale.y) as u32,
            format!("{}: {}", self.score.username, crate::format_number(self.score.score.score)),
            self.font.clone()
        ));

        // combo text
        list.push(Text::new(
            Color::WHITE,
            parent_depth + 4.0,
            self.pos + pos_offset + (PADDING + Vector2::new(0.0, PADDING.y + 15.0)) * self.ui_scale,
            (12.0 * self.ui_scale.y) as u32,
            format!("{}x, {:.2}%, {}{time_diff_str}", crate::format_number(self.score.max_combo), calc_acc(&self.score) * 100.0, self.score_mods),
            self.font.clone()
        ));
    }
}
