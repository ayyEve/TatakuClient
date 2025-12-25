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
    ) -> Arc<parley::Layout<Color>> {
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
            BOX_SIZE.x * scale.x
        };

        let max_width = box_width - 10.0; // padding of 5 from both sides
        if layout.width() >= max_width {
            style.font_size = 20.0 * scale.y / layout.width() * max_width;
            layout = font_contexts.simple_text(
                text,
                &style,
            );
            layout.break_all_lines(None);
        }

        Arc::new(layout)
    }
}
impl GameplayWidget for JudgementCounterElement {
    fn display_name(&self) -> &'static str { "Judgement Counter" }

    fn preferred_size(&self) -> Vector2 {
        let box_size = self.button_image.as_ref()
            .map_or(BOX_SIZE, Image::size);

        Vector2::new(box_size.x, box_size.y * self.counts.len() as f32)
    }

    fn update(&mut self, shell: &mut GameplayWidgetUpdateShell) {
        let score = &shell.manager.score().score;

        if self.counts.is_empty() {
            for j in shell.manager.judgments().iter().copied() {
                if j.display_name.is_empty() { continue }

                let layout = Self::layout(
                    j.display_name,
                    self.button_image.as_ref(),
                    &shell.scale,
                    shell.font_context,
                );

                self.counts.push(CachedJudgment {
                    judge: j,
                    count: score.get_judgment(j),
                    layout,
                });
            }

            shell.manager.mark_dirty(JUDGMENT_COUNTER.name);

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

            cached.layout = Self::layout(
                &text,
                self.button_image.as_ref(),
                &shell.scale,
                shell.font_context
            );
        }
    }

    fn draw(&self, shell: &mut GameplayWidgetDrawShell) {
        let box_size = self.button_image
            .as_ref()
            .map_or(BOX_SIZE, Image::size);

        for (i, cache) in self.counts.iter().enumerate() {
            let pos = Vector2::new(0.0, box_size.y * i as f32);

            if let Some(mut btn) = self.button_image.clone() {
                let transform = graphics::Transform {
                    pos,
                    ..graphics::Transform::identity()
                };

                // todo: setting?
                btn.color = cache.judge.color;

                shell.list.push(btn.with_transform(shell.transform * transform.matrix()));
            } else {
                // draw bg box
                shell.list.push(graphics::Rectangle::new(
                    box_size,
                    cache.judge.color,
                )
                .border(Border::new(
                    Color::BLACK,
                    2.0
                ))
                .with_transform(shell.transform * tataku::Matrix::identity()
                    .trans(pos)
                ));
            }

            let centered = Alignment::CENTER.resolve(
                &Bounds::new(pos, box_size),
                Vector2::new(
                    cache.layout.width(),
                    cache.layout.height()
                ),
                true,
                true,
            );

            // draw text/count
            shell.list.push(graphics::Text::new(cache.layout.clone())
                .with_transform(shell.transform * tataku::Matrix::identity()
                    .trans(centered)
            ));
        }
    }

    fn reload_skin(
        &mut self,
        shell: &mut GameplayWidgetReloadSkinShell
    ) {
        self.button_image = shell.skin_manager.get_texture(
            Path::new("inputoverlay-key"),
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
}
