use crate::prelude::*;

pub struct LeaderboardElement {
    scores: Vec<IngameScore>,
    image: Option<Image>,
}
impl LeaderboardElement {
    pub async fn new() -> Self {
        Self {
            scores: Vec::new(),
            image: None,
        }
    }
}
#[async_trait]
impl InnerUIElement for LeaderboardElement {
    fn display_name(&self) -> &'static str { "Leaderboard" }

    fn get_bounds(&self) -> Bounds {
        #[cfg(feature="graphics")]
        return Bounds::new(
            Vector2::ZERO,
            Vector2::new(
                LEADERBOARD_ITEM_SIZE.x,
                LEADERBOARD_ITEM_SIZE.y * 10.0
            )
        );

        #[cfg(not(feature="graphics"))]
        Bounds::default()
    }


    fn update(&mut self, manager: &mut GameplayManager) {
        //TODO: make this better?
        self.scores = manager.all_scores().into_iter().map(|i|i.clone()).collect();
    }

    #[cfg(feature="graphics")]
    fn draw(&mut self, pos_offset:Vector2, scale:Vector2, list: &mut RenderableCollection) {
        // draw scores
        // let args = RenderArgs {
        //     ext_dt: 0.0,
        //     window_size: [0.0, 0.0],
        //     draw_size: [0, 0],
        // };

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
            l.draw(Vector2::ZERO, list);
            base_pos += Vector2::with_y(l.size().y + 5.0) * scale;
        }

    }

    #[cfg(feature="graphics")]
    async fn reload_skin(&mut self, source: &TextureSource, skin_manager: &mut SkinManager) {
        self.image = skin_manager.get_texture("menu-button-background", source, SkinUsage::Gamemode, false).await;
    }
}
