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

// todo: merge this type with judgement counter
// todo: wrap both in a container that is centre aligned, so they look nicer
struct KeyCounterElement {
    counts: Vec<Cached>,
    button_image: Option<Image>,

    // reload_skins doesn't have widget manager and doesn't really need it
    dirty: bool,
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
            dirty: true,
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
impl GameplayWidget for KeyCounterElement {
    fn display_name(&self) -> &'static str { "Key Counter" }

    fn preferred_size(&self) -> Vector2 {
        let box_size = self.button_image.as_ref()
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

                let layout = Self::layout(
                    &info.label,
                    self.button_image.as_ref(),
                    &shell.scale,
                    shell.font_context,
                );

                self.counts.push(Cached {
                    count: info.count,
                    held: info.held,
                    press,
                    layout,
                });
            }

            self.dirty = true;
        } else {
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

                let layout = Self::layout(
                    &text,
                    self.button_image.as_ref(),
                    &shell.scale,
                    shell.font_context
                );

                i.layout = layout;
            }
        }

        if self.dirty {
            self.dirty = false;

            shell.manager.mark_dirty(KEY_COUNTER.name);
        }
    }

    fn draw(&self, shell: &mut GameplayWidgetDrawShell) {
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

            // draw bg box
            if let Some(btn) = self.button_image.clone() {
                let transform = if cached.held {
                    // todo: pad preferred_size by x1.1
                    graphics::Transform {
                        origin: btn.size() / 2.0, // use center origin for scaling
                        scale: Vector2::ONE * 1.1,
                        pos: pos + btn.size() / 2.0, // top left
                        ..graphics::Transform::identity()
                    }
                } else {
                    graphics::Transform {
                        pos,
                        ..graphics::Transform::identity()
                    }
                };

                shell.list.push(btn.with_transform(shell.transform * transform.matrix()));
            } else {
                shell.list.push(graphics::Rectangle::new(
                    box_size,
                    if cached.held {
                        Color::new(0.8, 0.0, 0.8, 0.8)
                    } else {
                        Color::new(0.0, 0.0, 0.0, 0.8)
                    },
                ).border(Border::new(
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
                    cached.layout.width(),
                    cached.layout.height()
                ),
                true,
                true,
            );

            shell.list.push(graphics::Text::new(cached.layout.clone())
                .with_transform(shell.transform * tataku::Matrix::identity()
                    .trans(centered),
            ));
        }
    }

    fn reload_skin(
        &mut self,
        shell: &mut GameplayWidgetReloadSkinShell
    ) {
        let size = self.preferred_size();

        // let mut background_image = shell.skin_manager.get_texture("inputoverlay-background", false;
        // if let Some(image) = &mut background_image {
        //     image.current_rotation = 90f64.to_radians();
        //     image.origin = Vector2::new(image.size().x, 0.0);
        //     // image.current_pos = pos - Vector2::new(image.size().x, 0.0);
        //     image.depth = -100.0;
        // }

        self.button_image = shell.skin_manager.get_texture(
            Path::new("inputoverlay-key"),
            shell.source,
            graphics::SkinUsage::Gamemode,
            false
        );

        if size != self.preferred_size() {
            self.dirty = true;
        }
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
}
