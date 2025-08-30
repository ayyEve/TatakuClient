use crate::prelude::*;
pub const LEADERBOARD_ITEM_SIZE:Vector2 = Vector2::new(200.0, 50.0);

struct LeaderboardElement {
    scores: Vec<IngameScore>,
    image: Option<Image>,

    info: GamemodeInfo,
}
impl LeaderboardElement {
    fn build(
        info: &GamemodeInfo,
        _: &Arc<CommonGameplaySettings>
    ) -> Box<dyn GameplayWidget> {
        Box::new(Self {
            scores: Vec::new(),
            image: None,
            info: *info
        })
    }
}
impl GameplayWidget for LeaderboardElement {
    fn display_name(&self) -> &'static str { "Leaderboard" }

    fn max_size(&self) -> Vector2 {
        Vector2::new(
            LEADERBOARD_ITEM_SIZE.x,
            LEADERBOARD_ITEM_SIZE.y * 10.0
        )
    }


    fn update(&mut self, manager: &mut dyn GameplayManagerTrait) {
        //TODO: make this better?
        self.scores = manager.all_scores().into_iter().cloned().collect();
    }

    fn draw(
        &mut self,
        mut pos_offset: Vector2,
        scale: Vector2,
        _align: Alignment,
        list: &mut RenderableCollection,
    ) {
        // draw scores
        // let args = RenderArgs {
        //     ext_dt: 0.0,
        //     window_size: [0.0, 0.0],
        //     draw_size: [0, 0],
        // };

        let mut is_pb = true;
        // let mut base_pos = pos_offset;

        let theme = Theme::default();

        for score in self.scores.iter() {

            let mut color_override = None;

            // let mut l = LeaderboardItem::new(score.clone(), self.info);
            // l.image = self.image.clone();
            // l.ui_scale_changed(scale);

            if score.is_current {
                color_override = Some(theme.get_color(ThemeColor::LeaderboardCurrentScore).unwrap_or(Color::RED));
            } else if score.is_previous {
                if is_pb {
                    is_pb = false;
                    color_override = Some(theme.get_color(ThemeColor::LeaderboardPreviousBest).unwrap_or(Color::BLUE));
                } else {
                    color_override = Some(theme.get_color(ThemeColor::LeaderboardPreviousScores).unwrap_or(Color::BLUE));
                }
            }

            // l.pos = base_pos;
            // l.draw(Vector2::ZERO, list);

            {
                let score_mods = ModManager::short_mods_string(
                    &score.mods,
                    false,
                    &self.info
                );

                const PADDING:Vector2 = Vector2::new(5.0, 5.0);
                let size = LEADERBOARD_ITEM_SIZE * scale;

                let now = chrono::Utc::now().timestamp() as u64;
                let time_diff = now as i64 - score.time as i64;
                let time_diff_str = if time_diff < 60 * 5 {
                    format!(" | {time_diff}s")
                } else {
                    String::new()
                };

                let color = if let Some(color) = color_override {
                    color
                }
                // else if self.selected {
                //     theme.get_color(ThemeColor::LeaderboardSelect).unwrap_or(Color::BLUE)
                // }
                // else if self.hover {
                //     theme.get_color(ThemeColor::LeaderboardHover).unwrap_or(Color::RED)
                // }
                else {
                    theme.get_color(ThemeColor::LeaderboardBg).unwrap_or(Color::WHITE)
                };

                // let text_color = if let Some(color) = text_color_override {
                //     color
                // }
                // else if self.selected {
                //     theme.get_color(ThemeColor::LeaderboardTextSelected).unwrap_or(Color::WHITE)
                // }
                // else if self.hover {
                //     theme.get_color(ThemeColor::LeaderboardTextHovered).unwrap_or(Color::WHITE)
                // }
                // else {
                let text_color = theme.get_color(ThemeColor::LeaderboardText).unwrap_or(Color::WHITE);
                // };

                if let Some(mut img) = self.image.clone() {
                    img.pos = pos_offset;
                    img.origin = Vector2::ZERO;
                    img.color = color;
                    img.set_size(size);

                    list.push(img);
                } else {
                    // bounding rect
                    list.push(
                        Rectangle::new(
                            pos_offset,
                            size,
                            Color::new(0.2, 0.2, 0.2, 1.0),
                        )
                        .shape(Shape::Round(5.0))
                        .border(Border::new(color, 1.5 * scale.y))
                    );
                }


                // score text
                // list.push(Text::new(
                //     pos_offset + PADDING * scale,
                //     15.0 * scale.y,
                //     format!("{}: {}", score.username, format_number(score.score.score)),
                //     text_color,
                //     DefaultFont::Main
                // ));

                // combo text
                // list.push(Text::new(
                //     pos_offset + (PADDING + Vector2::new(0.0, PADDING.y + 15.0)) * scale,
                //     12.0 * scale.y,
                //     format!("{}x, {:.2}%, {score_mods}{time_diff_str}", format_number(score.max_combo), self.info.calc_acc(score) * 100.0),
                //     text_color,
                //     DefaultFont::Main
                // ));
            }

            pos_offset += Vector2::with_y(LEADERBOARD_ITEM_SIZE.y + 5.0) * scale;
        }

    }

    fn reload_skin(
        &mut self,
        source: &TextureSource,
        skin_manager: &mut dyn SkinProvider
    ) {
        self.image = skin_manager.get_texture(
            "menu-button-background",
            source,
            SkinUsage::Gamemode,
            false
        );
    }
}


pub const LEADERBOARD: GameplayWidgetBuilder = GameplayWidgetBuilder {
    name: "leaderboard",
    default_layout: GameplayWidgetLayout::new_default(
        GameplayWidgetAnchor::Screen,
        Alignment::CENTER_LEFT,
        None,
        None,
    ),
    build: LeaderboardElement::build,
};


// pub struct LeaderboardItem {
//     pos: Vector2,
//     size: Vector2,
//     hover: bool,
//     selected: bool,
//     tag: String,

//     score: IngameScore,
//     font: Font,

//     score_mods: String,

//     ui_scale: Vector2,

//     pub color_override: Option<Color>,
//     pub text_color_override: Option<Color>,
//     pub image: Option<Image>,
//     // pub theme: ThemeHelper,
//     theme: Theme,

//     info: GameModeInfo
// }
// impl LeaderboardItem {
//     pub fn new(
//         score: IngameScore,
//         info: GameModeInfo
//     ) -> LeaderboardItem {
//         let pos = Vector2::ZERO;
//         let size = LEADERBOARD_ITEM_SIZE;

//         let tag = score.hash(); //username.clone();
//         let font = Font::Main;
//         let score_mods = ModManager::short_mods_string(
//             &score.mods,
//             false,
//             &info
//         );

//         LeaderboardItem {
//             pos,
//             size,
//             score,
//             tag,
//             hover: false,
//             selected: false,
//             font,
//             score_mods,
//             ui_scale: Vector2::ONE,

//             color_override: None,
//             text_color_override: None,
//             image: None,
//             theme: Theme::default(),
//             info,
//         }
//     }
//     pub async fn load_image(mut self, image: Image) -> Self {
//         self.image = Some(image); // = SkinManager::get_texture("menu-button-background", true).await;
//         self
//     }
// }

// impl LeaderboardItem {
//     fn ui_scale_changed(&mut self, scale: Vector2) {
//         self.ui_scale = scale;
//         self.size = LEADERBOARD_ITEM_SIZE * scale;
//     }

//     fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
//         const PADDING:Vector2 = Vector2::new(5.0, 5.0);

//         let now = chrono::Utc::now().timestamp() as u64;
//         let time_diff = now as i64 - self.score.time as i64;
//         let time_diff_str = if time_diff < 60 * 5 {
//             format!(" | {time_diff}s")
//         } else {
//             String::new()
//         };

//         let color = if let Some(color) = self.color_override {
//             color
//         } else if self.selected {
//             self.theme.get_color(ThemeColor::LeaderboardSelect).unwrap_or(Color::BLUE)
//         } else if self.hover {
//             self.theme.get_color(ThemeColor::LeaderboardHover).unwrap_or(Color::RED)
//         } else {
//             self.theme.get_color(ThemeColor::LeaderboardBg).unwrap_or(Color::WHITE)
//         };

//         let text_color = if let Some(color) = self.text_color_override {
//             color
//         } else if self.selected {
//             self.theme.get_color(ThemeColor::LeaderboardTextSelected).unwrap_or(Color::WHITE)
//         } else if self.hover {
//             self.theme.get_color(ThemeColor::LeaderboardTextHovered).unwrap_or(Color::WHITE)
//         } else {
//             self.theme.get_color(ThemeColor::LeaderboardText).unwrap_or(Color::WHITE)
//         };

//         if let Some(mut img) = self.image.clone() {
//             img.pos = self.pos;
//             img.origin = Vector2::ZERO;
//             img.color = color;
//             img.set_size(LEADERBOARD_ITEM_SIZE * self.ui_scale);

//             list.push(img)
//         } else {
//             // bounding rect
//             list.push(Rectangle::new(
//                 self.pos + pos_offset,
//                 LEADERBOARD_ITEM_SIZE * self.ui_scale,
//                 Color::new(0.2, 0.2, 0.2, 1.0),
//                 Some(Border::new(color, 1.5 * self.ui_scale.y))
//             ).shape(Shape::Round(5.0)));
//         }


//         // score text
//         list.push(Text::new(
//             self.pos + pos_offset + PADDING * self.ui_scale,
//             15.0 * self.ui_scale.y,
//             format!("{}: {}", self.score.username, format_number(self.score.score.score)),
//             text_color,
//             self.font
//         ));

//         // combo text
//         list.push(Text::new(
//             self.pos + pos_offset + (PADDING + Vector2::new(0.0, PADDING.y + 15.0)) * self.ui_scale,
//             12.0 * self.ui_scale.y,
//             format!("{}x, {:.2}%, {}{time_diff_str}", format_number(self.score.max_combo), self.info.calc_acc(&self.score) * 100.0, self.score_mods),
//             text_color,
//             self.font
//         ));
//     }


// }
