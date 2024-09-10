use crate::prelude::*;

/// needed to fix text hitcircle skins
const TEXT_SCALE:f32 = 0.8;

/// how long a single shake is
const SHAKE_TIME:f32 = 20.0;
/// how many shakes to perform
const SHAKE_COUNT:usize = 6;
/// can a shake request inturrupt another shake?
const SHAKE_INTURRUPT: bool = true;

#[derive(Clone)]
pub struct HitCircleImageHelper {
    pub base_pos: Vector2,
    /// scaled pos
    pub pos: Vector2,

    pub circle: Option<Image>,
    pub overlay: Option<Image>,
    pub combo_num: u16,

    pub scaling_helper: Arc<ScalingHelper>,
    alpha: f32,
    color: Color,
    
    /// combo num text cache
    combo_text: Option<Text>,
    combo_image: Option<SkinnedNumber>,

    skin_settings: Arc<SkinSettings>,
    shake_group: Option<TransformGroup>
}
impl HitCircleImageHelper {
    pub async fn new(base_pos: Vector2, scaling_helper: Arc<ScalingHelper>, combo_num: u16) -> Self {
        Self {
            circle: None,
            overlay: None,
            base_pos,
            pos: scaling_helper.scale_coords(base_pos),
            skin_settings: Default::default(),
            combo_num,
            scaling_helper,

            combo_image: None,
            combo_text: None,

            alpha: 0.0,
            color: Color::WHITE,
            shake_group: None
        }
    }

    #[cfg(feature="graphics")]
    pub async fn reload_skin(&mut self, source: &TextureSource, skin_manager: &mut dyn SkinProvider) {
        self.skin_settings = skin_manager.skin().clone();
        let radius = CIRCLE_RADIUS_BASE * self.scaling_helper.scaled_cs;

        self.circle = skin_manager.get_texture_then("hitcircle", source, SkinUsage::Gamemode, false, |i| {
            i.pos = self.pos;
            i.scale = Vector2::ONE * self.scaling_helper.scaled_cs;
            i.color = self.color;
        }).await;
        
        self.overlay = skin_manager.get_texture_then("hitcircleoverlay", source, SkinUsage::Gamemode, false, |i| {
            i.pos = self.pos;
            i.scale = Vector2::ONE * self.scaling_helper.scaled_cs;
        }).await;
        
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
        ).await.ok();

        let rect = Bounds::new(self.pos - Vector2::ONE * radius / 2.0, Vector2::ONE * radius);
        if let Some(combo) = &mut self.combo_image {
            combo.spacing_override = Some(-(self.skin_settings.hitcircle_overlap as f32));
            combo.scale = Vector2::ONE * self.scaling_helper.scaled_cs * TEXT_SCALE;
            combo.center_text(&rect);
            self.combo_text = None;
        } else if self.combo_text.is_none() {
            let mut text = Text::new(
                self.pos,
                radius,
                self.combo_num.to_string(),
                Color::BLACK,
                Font::Main
            );
            text.center_text(&rect);

            self.combo_text = Some(text);
        }

    }
    
    pub fn playfield_changed(&mut self, new_scale: &Arc<ScalingHelper>) {
        self.pos = new_scale.scale_coords(self.base_pos);
        let scale = Vector2::ONE * new_scale.scaled_cs;
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
        let radius = CIRCLE_RADIUS_BASE * new_scale.scaled_cs;
        let rect = Bounds::new(self.pos - Vector2::ONE * radius / 2.0, Vector2::ONE * radius);
        
        if let Some(image) = &mut self.combo_image {
            image.spacing_override = Some(-(self.skin_settings.hitcircle_overlap as f32));
            image.scale = scale * TEXT_SCALE;
            image.center_text(&rect);
        }
        if let Some(text) = &mut self.combo_text {
            text.set_font_size(radius);
            text.center_text(&rect)
        }

    }

    pub fn set_alpha(&mut self, alpha: f32) {
        self.alpha = alpha;
    }
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
        self.circle.as_mut().map(|c| c.color = color);
    }

    pub fn update(&mut self, time: f32) {
        if let Some(group) = &mut self.shake_group { 
            group.update(time);

            if group.transforms.is_empty() {
                self.shake_group = None;
            }
        }
    }

    pub fn draw(&mut self, list: &mut RenderableCollection) {
        if let Some(group) = self.shake_group.clone() {
            list.push(group);
            return
        }

        if let Some(mut circle) = self.circle.clone() {
            circle.color.a = self.alpha;
            list.push(circle);
        } else {
            list.push(Circle::new(
                self.pos,
                CIRCLE_RADIUS_BASE * self.scaling_helper.scaled_cs,
                self.color.alpha(self.alpha),
                Some(Border::new(
                    Color::BLACK.alpha(self.alpha),
                    self.scaling_helper.border_scaled
                ))
            ));
        }

        if let Some(mut overlay) = self.overlay.clone() {
            overlay.color.a = self.alpha;
            list.push(overlay);
        }

        if let Some(mut image) = self.combo_image.clone() {
            image.color.a = self.alpha;
            list.push(image);
        } else if let Some(mut text) = self.combo_text.clone() {
            text.color.a = self.alpha;
            list.push(text);
        }

    }

    /// helper fn to reduce duplicate code
    fn get_group(&self, include_combo_num: bool) -> TransformGroup {
        let mut group = TransformGroup::new(self.pos).alpha(1.0).border_alpha(1.0);
        
        // hit circle
        if let Some(mut circle) = self.circle.clone() {
            circle.pos = Vector2::ZERO;
            group.push(circle);
        }

        if let Some(mut overlay) = self.overlay.clone() {
            overlay.pos = Vector2::ZERO;
            group.push(overlay);
        }
        
        if group.items.len() == 0 {
            group.push(Circle::new(
                Vector2::ZERO,
                self.scaling_helper.scaled_cs,
                self.color,
                Some(Border::new(
                    Color::BLACK,
                    self.scaling_helper.border_scaled
                ))
            ));
        }

        if include_combo_num {
            // let radius = CIRCLE_RADIUS_BASE * self.scaling_helper.scaled_cs;
            let size = self.scaling_helper.scaled_circle_size;
            let rect = Bounds::new(-size / 2.0, size);

            if let Some(mut image) = self.combo_image.clone() {
                image.center_text(&rect);
                group.push(image);
            } else if let Some(mut text) = self.combo_text.clone() {
                text.center_text(&rect);
                group.push(text);
            }
        }

        group
    }


    pub fn shake(&mut self, time: f32) {
        if self.shake_group.is_some() && !SHAKE_INTURRUPT { return }

        let mut group = self.get_group(true);
        group.shake(0.0, time, Vector2::new(8.0, 0.0) * self.scaling_helper.scale, SHAKE_TIME, SHAKE_COUNT);
        self.shake_group = Some(group);
    }

    pub fn ripple(&self, time: f32) -> TransformGroup {
        let scale = 1.0..1.4;
        let mut group = self.get_group(false);

        // make it ripple and add it to the list
        group.ripple_scale_range(0.0, 240.0, time, scale, None, Some(0.5));
        group
    }

}
