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

    score: Score,
    font: Arc<Mutex<opengl_graphics::GlyphCache<'static>>>,

    score_mods: ModManager
}
impl LeaderboardItem {
    pub fn new(score:Score) -> LeaderboardItem {
        let tag = score.username.clone();
        let font = get_font();

        let mods = if let Some(mods) = &score.mods_string {
            serde_json::from_str(mods).unwrap_or(ModManager::new())
        } else {
            ModManager::new()
        };

        LeaderboardItem {
            pos: Vector2::zero(),
            size: LEADERBOARD_ITEM_SIZE,
            score,
            tag,
            hover: false,
            selected: false,
            font,
            score_mods: mods
        }
    }
}
impl ScrollableItem for LeaderboardItem {
    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2, parent_depth:f64, list:&mut Vec<Box<dyn Renderable>>) {
        const PADDING:Vector2 = Vector2::new(5.0, 5.0);

        // bounding rect
        list.push(Box::new(Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            parent_depth + 5.0,
            self.pos + pos_offset,
            LEADERBOARD_ITEM_SIZE,
            self.get_border_none(1.0)
        ).shape(Shape::Round(5.0, 10))));

        // score text
        list.push(Box::new(Text::new(
            Color::WHITE,
            parent_depth + 4.0,
            self.pos + pos_offset + PADDING,
            15,
            format!("{}: {}", self.score.username, crate::format_number(self.score.score)),
            self.font.clone()
        )));

        // combo text
        list.push(Box::new(Text::new(
            Color::WHITE,
            parent_depth + 4.0,
            self.pos + pos_offset + PADDING + Vector2::new(0.0, PADDING.y + 15.0),
            12,
            format!("{}x, {:.2}%, {}", crate::format_number(self.score.max_combo), calc_acc(&self.score) * 100.0, self.score_mods.mods_list_string()),
            self.font.clone()
        )));
    }
}
