use crate::prelude::*;

pub struct LeaderboardElement {
    scores: Vec<IngameScore>,
}
impl LeaderboardElement {
    pub fn new() -> Self {
        Self {
            scores: Vec::new()
        }
    }
}

impl InnerUIElement for LeaderboardElement {
    fn display_name(&self) -> &'static str { "Leaderboard" }

    fn get_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(
            Vector2::zero(),
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

    fn draw(&mut self, pos_offset:Vector2, scale:Vector2, list: &mut Vec<Box<dyn Renderable>>) {
        
        // draw scores
        let args = RenderArgs {
            ext_dt: 0.0,
            window_size: [0.0,0.0],
            draw_size: [0,0],
        };

        let mut base_pos = pos_offset;
        for score in self.scores.iter() {
            let mut l = LeaderboardItem::new(score.clone());
            l.ui_scale_changed(scale);

            if score.is_current {l.set_hover(true)}
            else if score.is_previous {l.set_selected(true)}

            l.set_pos(base_pos);
            l.draw(args, Vector2::zero(), 0.0, list);
            base_pos += Vector2::y_only(l.size().y + 5.0) * scale;
        }

    }
}
