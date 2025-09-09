use crate::prelude::*;
pub const LEADERBOARD_ITEM_SIZE:Vector2 = Vector2::new(200.0, 50.0);
const PADDING:Vector2 = Vector2::new(5.0, 5.0);

struct LeaderboardElement {
    cache: Vec<Cache>,
    current: Cache,
    // order: Vec<CachePos>,

    image: Option<Image>,
}
impl LeaderboardElement {
    fn build(
        _: &GamemodeInfo,
        _: &Arc<CommonGameplaySettings>
    ) -> Box<dyn GameplayWidget> {
        Box::new(Self {
            cache: Vec::new(),
            current: Cache::default(),
            // order: Vec::new(),
            image: None,
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


    fn update(&mut self, shell: &mut GameplayWidgetUpdateShell) {
        // //TODO: make this better?
        // self.scores = manager.all_scores().into_iter().cloned().collect();
        
        let theme = Theme::default();
        let scores = shell.manager.all_non_user_scores();
        let current = shell.manager.score();

        let properties = shell.manager.properties();
        let info = properties.info;

        if self.cache.len() != scores.len() {
            self.cache.clear();
            for score in scores {
                self.cache.push(Cache::new(
                    score, 
                    &theme,
                    &shell.scale,
                    info,
                    shell.font_context,
                ));
            }

            return;
        }

        for (cache, score) in self.cache.iter_mut()
            .zip(scores.iter())
            .chain([(&mut self.current, current)])
        {
            cache.update(
                &theme, 
                score, 
                &shell.scale, 
                info, 
                shell.font_context
            );
        }
    }

    fn draw(
        &mut self,
        shell: &mut GameplayWidgetDrawShell
    ) {
        // draw scores
        let theme = Theme::default();


        let mut order = self.cache.iter()
            .chain([&self.current])
            .collect::<Vec<_>>();

        order.sort();

        for (n, cache) in order.into_iter().enumerate() {
            let pos = shell.pos_offset 
                + Vector2::with_y(LEADERBOARD_ITEM_SIZE.y + 5.0) 
                * (n as f32) 
                * shell.scale;

            const PADDING:Vector2 = Vector2::new(5.0, 5.0);
            let size = LEADERBOARD_ITEM_SIZE * shell.scale;

            let color = if let Some(color) = cache.color_override {
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

            if let Some(mut img) = self.image.clone() {
                img.pos = pos;
                img.origin = Vector2::ZERO;
                img.color = color;
                img.set_size(size);

                shell.list.push(img);
            } else {
                // bounding rect
                shell.list.push(
                    Rectangle::new(
                        pos,
                        size,
                        Color::new(0.2, 0.2, 0.2, 1.0),
                    )
                    .shape(Shape::Round(5.0))
                    .border(Border::new(color, 1.5 * shell.scale.y))
                );
            }

            // score text
            if let Some(layout) = cache.score_text.clone() {
                shell.list.push(Transformed::new(
                    Transform::default()
                        .translate(pos + PADDING * shell.scale),
                    Box::new(Text::new(layout))
                ));
            }

            // combo text
            if let Some(layout) = cache.combo_text.clone() {
                shell.list.push(Transformed::new(
                    Transform::default()
                        .translate(pos + (PADDING + Vector2::new(0.0, PADDING.y + 15.0)) * shell.scale),
                    Box::new(Text::new(layout))
                ));
            }
        }

    }

    fn reload_skin(
        &mut self,
        shell: &mut GameplayWidgetReloadSkinShell
    ) {
        self.image = shell.skin_manager.get_texture(
            "menu-button-background",
            shell.source,
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

#[derive(Default)]
struct Cache {
    color_override: Option<Color>,
    order_value: f32,

    score_text: Option<Arc<parley::Layout<Color>>>,
    combo_text: Option<Arc<parley::Layout<Color>>>,
}
impl Cache {
    fn layout(
        text: &str,
        font_size: f32,
        color: Color,
        font_contexts: &mut TextLayoutContexts,
    ) -> Arc<parley::Layout<Color>> {
        let mut layout = font_contexts.simple_text(
            text, 
            &TextStyle {
                font_size,
                color,
                ..Default::default()
            }
        );
        layout.break_all_lines(None);

        Arc::new(layout)
    }

    fn combo_text(
        score: &IngameScore,
        info: &GamemodeInfo,
    ) -> String {
        let score_mods = ModManager::short_mods_string(
            &score.mods,
            false,
            &info
        );

        let now = chrono::Utc::now().timestamp() as u64;
        let time_diff = now as i64 - score.time as i64;
        let time_diff_str = (time_diff < 60 * 5)
            .then(|| format!(" | {time_diff}s"))
            .unwrap_or_default();
        
        format!(
            "{}x, {:.2}%, {score_mods}{time_diff_str}", 
            format_number(score.max_combo), 
            info.calc_acc(score) * 100.0
        )
    }

    fn new(
        score: &IngameScore,
        theme: &Theme,
        scale: &Vector2,
        info: &GamemodeInfo,
        font_contexts: &mut TextLayoutContexts,
    ) -> Self {
        let mut is_pb = true;
        let mut color_override = None;

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
        let text_color = theme
            .get_color(ThemeColor::LeaderboardText)
            .unwrap_or(Color::WHITE);
        // };
        

        let score_mods = ModManager::short_mods_string(
            &score.mods,
            false,
            info
        );

        let now = chrono::Utc::now().timestamp() as u64;
        let time_diff = now as i64 - score.time as i64;
        let time_diff_str = (time_diff < 60 * 5)
            .then(|| format!(" | {time_diff}s"))
            .unwrap_or_default();

        // score text
        // pos: pos_offset + PADDING * scale,
        let score_text = Self::layout(
            &format!("{}: {}", score.username, format_number(score.score.score)),
            15.0 * scale.y,
            text_color,
            font_contexts,
        );

        // combo text
        // pos: pos_offset + (PADDING + Vector2::new(0.0, PADDING.y + 15.0)) * scale
        let combo_text = Self::layout(
            &format!(
                "{}x, {:.2}%, {score_mods}{time_diff_str}", 
                format_number(score.max_combo), 
                info.calc_acc(score) * 100.0
            ),
            12.0 * scale.y,
            text_color,
            font_contexts,
        );

        Self {
            color_override,
            order_value: score.score.score as f32,
            score_text: Some(score_text),
            combo_text: Some(combo_text),
        }
    }

    fn update(
        &mut self, 
        theme: &Theme,
        score: &IngameScore,
        scale: &Vector2,
        info: &GamemodeInfo,
        font_contexts: &mut TextLayoutContexts,
    ) {

        let score_mods = ModManager::short_mods_string(
            &score.mods,
            false,
            info
        );

        let now = chrono::Utc::now().timestamp() as u64;
        let time_diff = now as i64 - score.time as i64;
        let time_diff_str = (time_diff < 60 * 5)
            .then(|| format!(" | {time_diff}s"))
            .unwrap_or_default();
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
        let text_color = theme
            .get_color(ThemeColor::LeaderboardText)
            .unwrap_or(Color::WHITE);
        // };

        // score text
        // pos: pos_offset + PADDING * scale,
        self.score_text = Some(Self::layout(
            &format!("{}: {}", score.username, format_number(score.score.score)),
            15.0 * scale.y,
            text_color,
            font_contexts,
        ));

        // combo text
        // pos: pos_offset + (PADDING + Vector2::new(0.0, PADDING.y + 15.0)) * scale
        self.combo_text = Some(Self::layout(
            &format!(
                "{}x, {:.2}%, {score_mods}{time_diff_str}", 
                format_number(score.max_combo), 
                info.calc_acc(score) * 100.0
            ),
            12.0 * scale.y,
            text_color,
            font_contexts,
        ));
    }
}

impl Eq for Cache {}
impl PartialEq for Cache {
    fn eq(&self, other: &Self) -> bool {
        self.order_value == other.order_value
    }
}
impl PartialOrd for Cache {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Cache {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order_value.total_cmp(&other.order_value)
    }
}


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
