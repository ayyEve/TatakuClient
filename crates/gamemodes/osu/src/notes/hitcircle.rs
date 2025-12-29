use crate::prelude::*;
use tataku::{
    Color,
    Border,
    Bounds,
    Vector2,
};

use engine::{
    graphics,
    ui,
};

use interface::parley;

/// needed to fix text hitcircle skins
const TEXT_SCALE:f32 = 0.8;

#[derive(Clone, Default)]
pub struct HitCircle {
    pub circle: Option<graphics::Image>,
    pub overlay: Option<graphics::Image>,
    pub combo_num: u16,

    /// combo num text cache
    combo_layout: Option<Arc<parley::Layout<Color>>>,
    combo_scale: f32,
    combo_size: Vector2,

    combo_image: Option<graphics::SkinnedNumber>,
}
impl HitCircle {
    pub fn new(combo_num: u16) -> Self {
        Self {
            circle: None,
            overlay: None,
            combo_num,

            combo_layout: None,
            combo_size: Vector2::ZERO,
            combo_scale: 0.0,

            combo_image: None,
        }
    }

    fn text_layout(
        text: &str,
        scale: f32,
        context: &mut ui::widget::TextLayoutContexts,
    ) -> (Arc<parley::Layout<Color>>, Vector2) {
        let style = ui::style::TextStyle {
            font_size: 30.0 * scale,
            ..Default::default()
        };

        let mut layout = context.simple_text(
            text,
            &style,
        );
        layout.break_all_lines(None);

        let size = Vector2::new(
            layout.width(),
            layout.height(),
        );

        (Arc::new(layout), size)
    }

    pub fn update(&mut self, shell: &mut UpdateShell) {
        if self.combo_layout.is_none()
        || self.combo_scale != shell.coords.cs {
            let (layout, size) = Self::text_layout(
                &self.combo_num.to_string(),
                shell.coords.cs,
                shell.font_contexts,
            );

            self.combo_size = size;
            self.combo_layout = Some(layout);
            self.combo_scale = shell.coords.cs;
        }
    }

    #[cfg(feature="graphics")]
    pub fn reload_skin(
        &mut self,
        source: &graphics::TextureSource,
        skin_manager: &mut dyn graphics::SkinProvider,
    ) {
        use graphics::{
            SkinUsage,
            SkinnedNumber,
        };

        self.circle = skin_manager.get_texture(
            Path::new("hitcircle"),
            source,
            SkinUsage::Gamemode,
            false,
        );

        self.overlay = skin_manager.get_texture(
            Path::new("hitcircleoverlay"),
            source,
            SkinUsage::Gamemode,
            false,
        );

        self.combo_image = SkinnedNumber::new(
            self.combo_num as f64,
            Color::WHITE,
            skin_manager.skin().hitcircle_prefix.clone(),
            None,
            0,
            skin_manager,

            source,
            SkinUsage::Gamemode,
        ).ok();

        self.combo_layout = None;

        // if let Some(combo) = &mut self.combo_image {
        //     combo.spacing_override = Some(-(settings.hitcircle_overlap as f32));
        // }

    }

    pub fn draw(
        &self,
        color_index: usize,
        shell: &mut DrawShell
    ) {
        let color = shell.combo_color(color_index)
            .alpha8(shell.alpha);

        // hit circle
        if let Some(mut overlay) = self.overlay.clone() {
            overlay.color.a = shell.alpha;

            let transform = graphics::Transform {
                origin: overlay.size() / 2.0,
                ..graphics::Transform::identity()
            };

            shell.list.push(overlay.with_transform(
                shell.transform * transform.matrix()
            ));
        }

        if let Some(mut circle) = self.circle.clone() {
            circle.color = color;

            let transform = graphics::Transform {
                origin: circle.size() / 2.0,
                ..graphics::Transform::identity()
            };

            shell.list.push(circle.with_transform(
                shell.transform * transform.matrix()
            ));
        } else {
            let mut circle = graphics::Circle::new(color);

            let transform = graphics::Transform {
                scale: Vector2::ONE * CIRCLE_RADIUS_BASE,
                ..graphics::Transform::identity()
            };

            if self.overlay.is_none() {
                circle.border = Some(Border::new(
                    Color::WHITE.alpha8(shell.alpha),
                    shell.coords.border_width
                ));
            }

            shell.list.push(circle.with_transform(
                shell.transform * transform.matrix()
            ));
        }

        let include_combo_num = true;

        if include_combo_num {
            if let Some(mut image) = self.combo_image.clone() {
                image.color.a = shell.alpha;

                let text_size = image.measure_text();

                let transform = graphics::Transform {
                    origin: text_size / 2.0,
                    ..graphics::Transform::identity()
                };

                shell.list.push(image.with_transform(
                    shell.transform * transform.matrix()
                ));
            } else if let Some(layout) = self.combo_layout.clone() {
                let transform = graphics::Transform {
                    origin: self.combo_size / 2.0,
                    scale: Vector2::ONE * shell.coords.cs.recip(),
                    ..graphics::Transform::identity()
                };

                shell.list.push(graphics::Text::new(layout)
                    .with_transform(shell.transform * transform.matrix())
                    .merge_draw_options(graphics::DrawOptions {
                        alpha: Some(shell.alpha),
                        ..Default::default()
                    })
                );
            }
        }
    }
}
