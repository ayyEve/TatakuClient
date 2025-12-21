use crate::prelude::*;
use tataku::{
    Color,
    Border,
    Bounds,
    Easing,
    Vector2,
    Animate,
    AnimationTimeline,
};

use engine::graphics;


/// needed to fix text hitcircle skins
const TEXT_SCALE:f32 = 0.8;

/// how long a single shake is
const SHAKE_TIME:f32 = 20.0;
/// how many shakes to perform
const SHAKE_COUNT:usize = 6;
/// can a shake request inturrupt another shake?
const SHAKE_INTURRUPT: bool = true;

#[derive(Clone, Default)]
pub struct HitCircle {
    pub base_pos: Vector2,
    /// scaled pos
    pub pos: Vector2,

    pub circle: Option<graphics::Image>,
    pub overlay: Option<graphics::Image>,
    pub combo_num: u16,

    pub scaling_helper: Arc<ScalingHelper>,
    alpha: u8,
    color: Color,

    /// combo num text cache
    // combo_text: Option<Text>,
    combo_image: Option<graphics::SkinnedNumber>,

    skin_settings: Arc<graphics::SkinSettings>,
    shake: Option<AnimationTimeline<f32>>
}
impl HitCircle {
    pub fn new(
        base_pos: Vector2,
        scaling_helper: Arc<ScalingHelper>,
        combo_num: u16
    ) -> Self {
        Self {
            circle: None,
            overlay: None,
            base_pos,
            pos: scaling_helper.scale_coords(base_pos),
            skin_settings: Arc::default(),
            combo_num,
            scaling_helper,

            combo_image: None,
            // combo_text: None,

            alpha: 0,
            color: Color::WHITE,
            shake: None
        }
    }

    #[cfg(feature="graphics")]
    pub fn reload_skin(
        &mut self,
        source: &graphics::TextureSource,
        skin_manager: &mut dyn graphics::SkinProvider
    ) {
        use graphics::{
            SkinUsage,
            SkinnedNumber,
        };

        self.skin_settings = skin_manager.skin().clone();
        let radius = CIRCLE_RADIUS_BASE * self.scaling_helper.cs;

        self.circle = skin_manager.get_texture_then("hitcircle", source, SkinUsage::Gamemode, false, |i| {
            i.pos = self.pos;
            i.scale = Vector2::ONE * self.scaling_helper.cs;
            i.color = self.color;
        });

        self.overlay = skin_manager.get_texture_then("hitcircleoverlay", source, SkinUsage::Gamemode, false, |i| {
            i.pos = self.pos;
            i.scale = Vector2::ONE * self.scaling_helper.cs;
        });

        self.combo_image = SkinnedNumber::new(
            self.pos,
            self.combo_num as f64,
            Color::WHITE,
            &self.skin_settings.hitcircle_prefix,
            None,
            0,
            skin_manager,

            source,
            SkinUsage::Gamemode,
        ).ok();

        let rect = Bounds::new(self.pos - Vector2::ONE * radius / 2.0, Vector2::ONE * radius);
        if let Some(combo) = &mut self.combo_image {
            combo.spacing_override = Some(-(self.skin_settings.hitcircle_overlap as f32));
            combo.scale = Vector2::ONE * self.scaling_helper.cs * TEXT_SCALE;
            combo.center_text(&rect);
            // self.combo_text = None;
        // } else if self.combo_text.is_none() {
        //     let mut text = Text::new(
        //         Vector2::ZERO,
        //         radius,
        //         self.combo_num,
        //         Color::WHITE,
        //         DefaultFont::Main
        //     );
        //     text.line_height = radius / 2.0;
        //     text.center_text(&rect);

        //     self.combo_text = Some(text);
        }

    }

    pub fn playfield_changed(&mut self, new_scale: &Arc<ScalingHelper>) {
        self.pos = new_scale.scale_coords(self.base_pos);
        let scale = Vector2::ONE * new_scale.cs;
        self.scaling_helper = new_scale.clone();

        // update circle positions
        if let Some(overlay) = &mut self.overlay {
            overlay.pos = self.pos;
            overlay.scale = scale;
        }
        if let Some(circle) = &mut self.circle {
            circle.pos = self.pos;
            circle.scale = scale;
        }

        // update combo text position
        let radius = CIRCLE_RADIUS_BASE * new_scale.cs;
        let rect = Bounds::new(self.pos - Vector2::ONE * radius / 2.0, Vector2::ONE * radius);

        if let Some(image) = &mut self.combo_image {
            image.spacing_override = Some(-(self.skin_settings.hitcircle_overlap as f32));
            image.scale = scale * TEXT_SCALE;
            image.center_text(&rect);
        }
        // if let Some(text) = &mut self.combo_text {
        //     text.set_font_size(radius);
        //     text.center_text(&rect);
        // }

    }

    pub fn set_alpha(&mut self, alpha: u8) {
        self.alpha = alpha;
    }
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
        if let Some(circle) = &mut self.circle {
            circle.color = color;
        }
    }

    pub fn update(&mut self, time: f32) {
        if let Some(shake) = &mut self.shake {
            shake.update(time);

            if shake.is_empty() {
                self.shake = None;
            }
        }
    }

    pub fn draw(&mut self, list: &mut graphics::RenderableCollection) {
        let note = self.note(true);

        if let Some(shake) = &self.shake {
            let shake = shake.last_value();

            let transform = graphics::Transform {
                pos: Vector2::new(shake * 8.0 * self.scaling_helper.scale, 0.0),
                ..Default::default()
            };

            let elements = note.list.into_iter()
                .map(|element| graphics::Transformed::new(
                    transform,
                    element
                ))
                .map(|element| Box::new(element) as Box<dyn graphics::TatakuRenderable>);

            list.list.extend(elements);
        } else {
            list.list.extend(note.list);
        }
    }

    fn note(&self, include_combo_num: bool) -> graphics::RenderableCollection {
        let mut collection = graphics::RenderableCollection::default();

        // hit circle
        if let Some(mut circle) = self.circle.clone() {
            circle.pos = self.pos;
            circle.color.a = self.alpha;
            collection.push(circle);
        }

        if let Some(mut overlay) = self.overlay.clone() {
            overlay.pos = self.pos;
            overlay.color.a = self.alpha;
            collection.push(overlay);
        }

        if collection.list.is_empty() {
            collection.push(graphics::Circle::new(
                self.pos,
                CIRCLE_RADIUS_BASE * self.scaling_helper.cs,
                self.color.alpha8(self.alpha),
            ).border(Border::new(
                Color::WHITE.alpha8(self.alpha),
                self.scaling_helper.border_width
            )));
        }

        if include_combo_num {
            let size = self.scaling_helper.circle_size;
            let rect = Bounds::new(self.pos - size / 2.0, size);

            if let Some(mut image) = self.combo_image.clone() {
                image.color.a = self.alpha;
                image.center_text(&rect);
                collection.push(image);
            // } else if let Some(mut text) = self.combo_text.clone() {
                // text.color.a = self.alpha;
                // text.center_text(&rect);
                // collection.push(text);
            }
        }

        collection
    }


    pub fn shake(&mut self, time: f32) {
        if self.shake.is_some() && !SHAKE_INTURRUPT { return }

        let shake = shake(
            time,
            SHAKE_TIME,
            SHAKE_COUNT,
            Easing::Linear,
        );

        self.shake = Some(shake);
    }

    // pub fn ripple(&self, time: f32) -> TransformGroup {
    //     let scale = 1.0..1.4;
    //     let mut group = self.note(false);

    //     // make it ripple and add it to the list
    //     group.ripple_scale_range(0.0, 240.0, time, scale, None, Some(0.5));
    //     group
    // }

}

pub fn shake(
    start_time: f32,
    time_between_shakes: f32,
    shake_count: usize,
    easing: Easing,
) -> AnimationTimeline<f32> {
    let mut animations = Vec::with_capacity(shake_count);

    animations.push(Animate::new(
        start_time,
        time_between_shakes / 2.0,
        easing,
        0.0,
        1.0,
    ));

    if shake_count > 1 {
        for i in 0..shake_count-1 {
            let (start, end) = if i % 2 == 0 {
                (1.0, -1.0)
            } else {
                (-1.0, 1.0)
            };

            animations.push(Animate::new(
                start_time + time_between_shakes * (i as f32 + 0.5),
                time_between_shakes,
                easing,
                start,
                end,
            ));
        }
    }

    let (start, end) = if shake_count % 2 == 0 {
        (-1.0, 0.0)
    } else {
        (1.0, 0.0)
    };

    animations.push(Animate::new(
        start_time + time_between_shakes * (shake_count as f32 + 0.5) ,
        time_between_shakes / 2.0,
        easing,
        start,
        end,
    ));

    AnimationTimeline::new(animations, 0.0)
}
