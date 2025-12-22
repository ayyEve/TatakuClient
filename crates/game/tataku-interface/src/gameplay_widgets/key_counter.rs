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

struct KeyCounterElement {
    counts: Vec<Cached>,
    button_image: Option<Image>
}
impl KeyCounterElement {
    fn build(
        _: &GamemodeInfo,
        _: &Arc<CommonGameplaySettings>
    ) -> Box<dyn GameplayWidget> {
        Box::new(Self {
            counts: Vec::new(),
            // counter: KeyCounter::default(),

            // background_image,
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
impl GameplayWidget for KeyCounterElement {
    fn display_name(&self) -> &'static str { "Key Counter" }

    fn preferred_size(&self) -> Vector2 {
        let box_size = self.button_image
            .as_ref()
            .map_or(BOX_SIZE, Image::size);
        Vector2::new(box_size.x, box_size.y * self.counts.len() as f32)
    }

    fn update(
        &mut self,
        shell: &mut GameplayWidgetUpdateShell,
    ) {
        let counter = shell.manager.key_counter();

        if self.counts.is_empty() {
            for i in 0..counter.key_order.len() {
                let press = counter.key_order[i];
                let info = &counter.keys[&press];

                let (layout, size) = Self::layout(
                    &info.label,
                    self.button_image.as_ref(),
                    &shell.scale,
                    shell.font_context,
                );

                self.counts.push(Cached {
                    count: info.count,
                    held: info.held,
                    press,
                    size,
                    layout,
                });
            }

            return;
        }

        for i in self.counts.iter_mut() {
            let info = &counter.keys[&i.press];
            i.held = info.held;

            if i.count == info.count { continue }
            i.count = info.count;

            let text = if info.count == 0 {
                Cow::Borrowed(&*info.label)
            } else {
                Cow::Owned(tataku::format_number(&i.count))
            };

            let (layout, size) = Self::layout(
                &text,
                self.button_image.as_ref(),
                &shell.scale,
                shell.font_context
            );

            i.size = size;
            i.layout = layout;
        }
    }

    fn draw(
        &mut self,
        shell: &mut GameplayWidgetDrawShell,
    ) {
        let box_size = self.button_image
            .as_ref()
            .map_or(BOX_SIZE, Image::size);

        // if let Some(bg) = &self.background_image {
        //     let mut bg = bg.clone();
        //     bg.current_pos = base_pos + Vector2::new(pad.x, pad.y * self.key_order.len() as f64);
        //     list.push(Box::new(bg));
        // }

        for (i, cached) in self.counts.iter().enumerate() {
            let pos = Vector2::new(
                0.0,
                box_size.y * i as f32
            );
            let bounds = Bounds::new(
                pos,
                box_size,
            );

            // draw bg box
            if let Some(mut btn) = self.button_image.clone() {
                btn.pos = pos + box_size / 2.0;
                if cached.held {
                    btn.scale *= 1.1;
                }

                shell.list.push(graphics::Transformed {
                    transform: shell.transform,
                    drawable: Box::new(btn),
                });
            } else {
                shell.list.push(graphics::Transformed {
                    transform: shell.transform,
                    drawable: Box::new(graphics::Rectangle::new_bounds(
                        bounds,
                        if cached.held {
                            Color::new(0.8, 0.0, 0.8, 0.8)
                        } else {
                            Color::new(0.0, 0.0, 0.0, 0.8)
                        },
                    ).border(Border::new(
                        Color::BLACK,
                        2.0
                    )))
                });
            }

            let centered = Alignment::CENTER.resolve(
                &bounds,
                cached.size,
                true,
                true,
            );

            shell.list.push(graphics::Transformed {
                transform: shell.transform * tataku::Matrix::identity()
                    .trans(centered),
                drawable: Box::new(graphics::Text::new(cached.layout.clone())),
            });
        }
    }

    fn reload_skin(
        &mut self,
        shell: &mut GameplayWidgetReloadSkinShell
    ) {
        // let mut background_image = shell.skin_manager.get_texture("inputoverlay-background", false;
        // if let Some(image) = &mut background_image {
        //     image.current_rotation = 90f64.to_radians();
        //     image.origin = Vector2::new(image.size().x, 0.0);
        //     // image.current_pos = pos - Vector2::new(image.size().x, 0.0);
        //     image.depth = -100.0;
        // }

        self.button_image = shell.skin_manager.get_texture(
            "inputoverlay-key",
            shell.source,
            graphics::SkinUsage::Gamemode,
            false
        );
    }
}

pub const KEY_COUNTER: GameplayWidgetBuilder = GameplayWidgetBuilder {
    name: "key_counter",
    default_layout: GameplayWidgetLayout {
        anchor: GameplayWidgetAnchor::Screen,
        align: Alignment::CENTER_RIGHT,
        transform: graphics::Transform::identity(),
    },
    build: KeyCounterElement::build,
};

struct Cached {
    count: u16,
    held: bool,
    press: common::replays::KeyPress,
    layout: Arc<parley::Layout<Color>>,
    size: Vector2,
}
