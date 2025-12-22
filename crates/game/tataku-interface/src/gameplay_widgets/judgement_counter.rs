use crate::prelude::*;
use graphics::Image;
use tataku::{
    Alignment,
    Border,
    Bounds,
    Vector2,
    Color,
};
use engine::{
    settings::common_gameplay::CommonGameplaySettings,
    gameplay::{
        widgets::*,
        GamemodeInfo,
    },
};

const BOX_SIZE:Vector2 = Vector2::new(40.0, 40.0);

struct JudgementCounterElement {
    counts: Vec<CachedJudgment>,
    button_image: Option<Image>,
}
impl JudgementCounterElement {
    fn build(
        _: &GamemodeInfo,
        _: &Arc<CommonGameplaySettings>
    ) -> Box<dyn GameplayWidget> {
        Box::new(Self {
            counts: Vec::new(),
            button_image: None,
        })
    }

    fn layout(
        text: &str,
        button_image: Option<&Image>,
        scale: &Vector2,
        font_contexts: &mut ui::widget::TextLayoutContexts,
    ) -> (Arc<parley::Layout<Color>>, Vector2) {
        let mut style = ui::style::TextStyle {
            font_size: 20.0 * scale.y,
            color: Color::WHITE,
            ..Default::default()
        };

        let mut layout = font_contexts.simple_text(
            text,
            &style,
        );
        layout.break_all_lines(None);

        let box_width = if let Some(btn) = button_image {
            btn.size().x * scale.x
        } else {
            (BOX_SIZE * *scale).x
        };

        let mut text_size = Vector2::new(
            layout.width(),
            layout.height(),
        );

        let max_width = box_width - 10.0; // padding of 10
        if text_size.x >= max_width {
            style.font_size = 20.0 * scale.x * max_width / text_size.x;
            layout = font_contexts.simple_text(
                text,
                &style,
            );
            layout.break_all_lines(None);

            text_size = Vector2::new(
                layout.width(),
                layout.height(),
            );
        }

        (Arc::new(layout), text_size)
    }
}
impl GameplayWidget for JudgementCounterElement {
    fn display_name(&self) -> &'static str { "Judgement Counter" }

    fn max_size(&self) -> Vector2 {
        let box_size = self.button_image.as_ref()
            .map_or(BOX_SIZE, Image::size);

        Vector2::new(box_size.x, box_size.y * self.counts.len() as f32)
    }

    fn update(&mut self, shell: &mut GameplayWidgetUpdateShell) {
        let score = &shell.manager.score().score;

        if self.counts.is_empty() {
            for j in shell.manager.judgments() {
                if j.display_name.is_empty() { continue }

                let (layout, size) = Self::layout(
                    j.display_name,
                    self.button_image.as_ref(),
                    &shell.scale,
                    shell.font_context,
                );

                self.counts.push(CachedJudgment {
                    judge: *j,
                    count: score.get_judgment(j),
                    size,
                    layout,
                });
            }

            return;
        }

        for (i, judge) in shell.manager
            .judgments()
            .iter()
            .filter(|j| !j.display_name.is_empty())
            .copied()
            .enumerate()
        {
            let Some(cached) = self.counts.get_mut(i)
            else { continue };

            let new_count = score.get_judgment(judge);
            if cached.count == new_count { continue }

            cached.count = new_count;

            let text = if new_count == 0 {
                Cow::Borrowed(judge.display_name)
            } else {
                tataku::format_number(&new_count).into()
            };

            let (layout, size) = Self::layout(
                &text,
                self.button_image.as_ref(),
                &shell.scale,
                shell.font_context
            );

            cached.size = size;
            cached.layout = layout;
        }

    }

    fn draw(
        &mut self,
        shell: &mut GameplayWidgetDrawShell
    ) {
        let box_size = self.button_image
            .as_ref()
            .map_or(BOX_SIZE, Image::size);

        for (i, cache) in self.counts.iter().enumerate() {
            let pos = Vector2::new(0.0, box_size.y * i as f32);
            let box_bounds = Bounds::new(pos, box_size);

            if let Some(mut btn) = self.button_image.clone() {
                btn.pos = pos + box_size / 2.0;
                btn.color = cache.judge.color;

                shell.list.push(graphics::Transformed {
                    transform: shell.transform,
                    drawable: Box::new(btn)
                });
            } else {
                // draw bg box
                shell.list.push(graphics::Rectangle::new_bounds(
                    box_bounds,
                    cache.judge.color,
                )
                .border(Border::new(
                    Color::BLACK,
                    2.0
                )));
            }

            let centered = Alignment::CENTER.resolve(
                &box_bounds,
                cache.size,
                true,
                true
            );

            // draw text/count
            shell.list.push(graphics::Transformed {
                transform: shell.transform * tataku::Matrix::identity()
                    .trans(centered),
                drawable: Box::new(graphics::Text::new(cache.layout.clone())),
            });
        }
    }

    fn reload_skin(
        &mut self,
        shell: &mut GameplayWidgetReloadSkinShell
    ) {
        self.button_image = shell.skin_manager.get_texture(
            "inputoverlay-key",
            shell.source,
            graphics::SkinUsage::Gamemode,
            false
        );
    }
}

pub const JUDGMENT_COUNTER: GameplayWidgetBuilder = GameplayWidgetBuilder {
    name: "judgement_counter",
    default_layout: GameplayWidgetLayout {
        anchor: GameplayWidgetAnchor::Element {
            element: Cow::Borrowed("key_counter"),
            horizontal_side: Side::Inside,
            vertical_side: Side::Outside,
        },
        align: Alignment::BOTTOM_RIGHT,
        transform: graphics::Transform::identity(),
    },
    build: JudgementCounterElement::build,
};


struct CachedJudgment {
    judge: engine::gameplay::judgments::HitJudgment,
    count: u16,
    layout: Arc<parley::Layout<Color>>,
    size: Vector2,
}
