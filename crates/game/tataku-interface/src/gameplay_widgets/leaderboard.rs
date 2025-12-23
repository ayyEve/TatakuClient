use crate::prelude::*;
use tataku::{
    Color,
    Border,
    Vector2,
    Alignment,
};
use graphics::{
    Text,
    Image,
    Shape,
    Rectangle,
    SkinUsage,
    ThemeColor,

    Transform,
    Transformed,
};
use engine::{
    settings::common_gameplay::CommonGameplaySettings,
    gameplay::{
        widgets::*,
        IngameScore,
        GamemodeInfo,
        mods::ModManager,
    },
};

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

    fn preferred_size(&self) -> Vector2 {
        Vector2::new(
            LEADERBOARD_ITEM_SIZE.x,
            LEADERBOARD_ITEM_SIZE.y * 10.0
        )
    }


    fn update(&mut self, shell: &mut GameplayWidgetUpdateShell) {
        // //TODO: make this better?
        // self.scores = manager.all_scores().into_iter().cloned().collect();

        let theme = graphics::Theme::default();
        let scores = shell.manager.all_non_user_scores();
        let current = shell.manager.score();

        let properties = shell.manager.properties();
        let info = properties.info;

        let mut is_pb = true;

        if self.cache.len() != scores.len() {
            self.cache.clear();
            for score in scores {
                self.cache.push(Cache::new(
                    score,
                    &theme,
                    &shell.scale,
                    &mut is_pb,
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

    fn draw(&self, shell: &mut GameplayWidgetDrawShell) {
        // draw scores
        let theme = graphics::Theme::default();


        let mut order = self.cache.iter()
            .chain([&self.current])
            .collect::<Vec<_>>();

        order.sort();

        for (n, cache) in order.into_iter().enumerate() {
            let pos = Vector2::with_y(LEADERBOARD_ITEM_SIZE.y + 5.0)
                * (n as f32);

            let size = LEADERBOARD_ITEM_SIZE;

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

                shell.list.push(graphics::Transformed {
                    transform: shell.transform,
                    drawable: Box::new(img),
                });
            } else {
                // bounding rect
                shell.list.push(graphics::Transformed {
                    transform: shell.transform,
                    drawable: Box::new(Rectangle::new(
                        pos,
                        size,
                        Color::new(0.2, 0.2, 0.2, 1.0),
                    )
                    .shape(Shape::Round(5.0))
                    .border(Border::new(color, 1.5)))
                });
            }

            // score text
            if let Some(layout) = cache.score_text.clone() {
                shell.list.push(Transformed {
                    transform: shell.transform * tataku::Matrix::identity()
                        .trans(pos + PADDING),
                    drawable: Box::new(Text::new(layout))
                });
            }

            // combo text
            if let Some(layout) = cache.combo_text.clone() {
                shell.list.push(graphics::Transformed {
                    transform: shell.transform * tataku::Matrix::identity()
                        .trans(pos + (PADDING + Vector2::new(0.0, PADDING.y + 15.0))),
                    drawable: Box::new(graphics::Text::new(layout))
                });
            }
        }

    }

    fn reload_skin(
        &mut self,
        shell: &mut GameplayWidgetReloadSkinShell
    ) {
        self.image = shell.skin_manager.get_texture(
            Path::new("menu-button-background"),
            shell.source,
            SkinUsage::Gamemode,
            false
        );
    }
}


pub const LEADERBOARD: GameplayWidgetBuilder = GameplayWidgetBuilder {
    name: "leaderboard",
    default_layout: GameplayWidgetLayout {
        anchor: GameplayWidgetAnchor::Screen,
        align: Alignment::CENTER_LEFT,
        transform: Transform::identity(),
    },
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
        font_contexts: &mut ui::widget::TextLayoutContexts,
    ) -> Arc<parley::Layout<Color>> {
        let mut layout = font_contexts.simple_text(
            text,
            &ui::style::TextStyle {
                font_size,
                color,
                ..Default::default()
            }
        );
        layout.break_all_lines(None);

        Arc::new(layout)
    }

    fn time_str(score_time: u64) -> String {
        let now = chrono::Utc::now().timestamp() as u64;
        let time_diff = now as i64 - score_time as i64;
        if time_diff < 60 * 5 {
            format!(" | {time_diff}s")
        } else {
            String::new()
        }
    }

    fn combo_text(
        score: &IngameScore,
        info: &GamemodeInfo,
    ) -> String {
        let score_mods = ModManager::short_mods_string(
            &score.mods,
            false,
            info
        );

        let time_diff_str = Self::time_str(score.time);
        format!(
            "{}x, {:.2}%, {score_mods}{time_diff_str}",
            tataku::format_number(&score.max_combo),
            info.calc_acc(score) * 100.0
        )
    }

    fn new(
        score: &engine::gameplay::IngameScore,
        theme: &graphics::Theme,
        scale: &Vector2,
        is_pb: &mut bool,
        info: &GamemodeInfo,
        font_contexts: &mut ui::widget::TextLayoutContexts,
    ) -> Self {
        let mut color_override = None;
        use engine::gameplay::ScoreType;

        match score.score_type {
            ScoreType::Current => {
                color_override = Some(theme
                    .get_color(ThemeColor::LeaderboardCurrentScore)
                    .unwrap_or(Color::RED)
                );
            }
            ScoreType::Previous => {
                if *is_pb {
                    *is_pb = false;
                    color_override = Some(theme
                        .get_color(ThemeColor::LeaderboardPreviousBest)
                        .unwrap_or(Color::BLUE)
                    );
                } else {
                    color_override = Some(theme
                        .get_color(ThemeColor::LeaderboardPreviousScores)
                        .unwrap_or(Color::BLUE)
                    );
                }
            }
            _ => {}
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

        // score text
        // pos: pos_offset + PADDING * scale,
        let score_text = Self::layout(
            &format!("{}: {}", score.username, tataku::format_number(&score.score.score)),
            15.0 * scale.y,
            text_color,
            font_contexts,
        );

        // combo text
        // pos: pos_offset + (PADDING + Vector2::new(0.0, PADDING.y + 15.0)) * scale
        let combo_text = Self::layout(
            &Self::combo_text(score, info),
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
        theme: &graphics::Theme,
        score: &IngameScore,
        scale: &Vector2,
        info: &GamemodeInfo,
        font_contexts: &mut ui::widget::TextLayoutContexts,
    ) {
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
            &format!("{}: {}", score.username, tataku::format_number(&score.score.score)),
            15.0 * scale.y,
            text_color,
            font_contexts,
        ));

        // combo text
        // pos: pos_offset + (PADDING + Vector2::new(0.0, PADDING.y + 15.0)) * scale
        self.combo_text = Some(Self::layout(
            &Self::combo_text(score, info),
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
