use crate::prelude::*;

pub struct LeaderboardElement {
    scores: Vec<IngameScore>,
    image: Option<Image>,
}
impl LeaderboardElement {
    pub async fn new() -> Self {
        Self {
            scores: Vec::new(),
            image: SkinManager::get_texture("menu-button-background", true).await,
        }
    }
}

impl InnerUIElement for LeaderboardElement {
    fn display_name(&self) -> &'static str { "Leaderboard" }

    fn get_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(
            Vector2::ZERO,
            Vector2::new(
                LEADERBOARD_ITEM_SIZE.x,
                LEADERBOARD_ITEM_SIZE.y * 10.0
            )
        )
    }


    fn update(&mut self, manager: &mut IngameManager) {
        //TODO: make this better?
        self.scores = manager.all_scores().into_iter().map(|i|i.clone()).collect();
    }

    fn draw(&mut self, pos_offset:Vector2, scale:Vector2, list: &mut RenderableCollection) {
        // draw scores
        let args = RenderArgs {
            ext_dt: 0.0,
            window_size: [0.0, 0.0],
            draw_size: [0, 0],
        };

        let mut is_pb = true;

        let mut base_pos = pos_offset;
        for score in self.scores.iter() {
            let mut l = LeaderboardItem::new(score.clone());
            l.image = self.image.clone();
            l.ui_scale_changed(scale);

            if score.is_current { 
                l.color_override = Some(l.theme.get_color(ThemeColor::LeaderboardCurrentScore).unwrap_or(Color::RED));
            } else if score.is_previous { 
                if is_pb {
                    is_pb = false;
                    l.color_override = Some(l.theme.get_color(ThemeColor::LeaderboardPreviousBest).unwrap_or(Color::BLUE));
                } else {
                    l.color_override = Some(l.theme.get_color(ThemeColor::LeaderboardPreviousScores).unwrap_or(Color::BLUE));
                }
            }

            l.set_pos(base_pos);
            l.draw(args, Vector2::ZERO, 0.0, list);
            base_pos += Vector2::with_y(l.size().y + 5.0) * scale;
        }

    }
}
