use crate::prelude::*;

/// needed to fix text hitcircle skins
const TEXT_SCALE:f32 = 0.8;

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
}
impl HitCircleImageHelper {
    pub async fn new(base_pos: Vector2, scaling_helper: Arc<ScalingHelper>, color: Color, combo_num: u16) -> Self {
        let skin_settings = SkinManager::current_skin_config().await;
        Self {
            circle: None,
            overlay: None,
            base_pos,
            pos: scaling_helper.scale_coords(base_pos),
            skin_settings,
            combo_num,
            scaling_helper,

            combo_image: None,
            combo_text: None,

            alpha: 0.0,
            color,
        }
    }

    pub async fn reload_skin(&mut self) {
        self.skin_settings = SkinManager::current_skin_config().await;
        let radius = CIRCLE_RADIUS_BASE * self.scaling_helper.scaled_cs;

        self.circle = SkinManager::get_texture("hitcircle", true).await;
        if let Some(circle) = &mut self.circle {
            circle.pos = self.pos;
            circle.scale = Vector2::ONE * self.scaling_helper.scaled_cs;
            circle.color = self.color;
        }
        
        self.overlay = SkinManager::get_texture("hitcircleoverlay", true).await;
        if let Some(overlay) = &mut self.overlay {
            overlay.pos = self.pos;
            overlay.scale = Vector2::ONE * self.scaling_helper.scaled_cs;
        }
        self.combo_image = SkinnedNumber::new(
            self.pos, 
            self.combo_num as f64,
            Color::WHITE, 
            &self.skin_settings.hitcircle_prefix,
            None,
            0
        ).await.ok();

        let rect = Bounds::new(self.pos - Vector2::ONE * radius / 2.0, Vector2::ONE * radius);
        if let Some(combo) = &mut self.combo_image {
            combo.spacing_override = Some(-(self.skin_settings.hitcircle_overlap as f32));
            combo.scale = Vector2::ONE * self.scaling_helper.scaled_cs * TEXT_SCALE;
            combo.center_text(&rect);
            self.combo_text = None;
        } else if let Some(_text) = &mut self.combo_text {

        } else {
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

    pub fn draw(&mut self, list: &mut RenderableCollection) {

        if let Some(mut circle) = self.circle.clone() {
            circle.color.a = self.alpha;
            list.push(circle);
        } else {
            let radius = CIRCLE_RADIUS_BASE * self.scaling_helper.scaled_cs;
            list.push(Circle::new(
                self.pos,
                radius,
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


    pub fn ripple(&self, time: f32) -> TransformGroup {
        let scale = 1.0..1.3;
        let radius = CIRCLE_RADIUS_BASE * self.scaling_helper.scaled_cs;
        let mut group = TransformGroup::new(self.pos).alpha(1.0).border_alpha(1.0);

        // combo text
        if let Some(mut c) = self.combo_image.clone() {
            let bounds = Bounds::new(-Vector2::ONE * radius / 2.0, Vector2::ONE * radius);
            c.center_text(&bounds);
            group.push(c);
        }
        // else if let Some(mut text) = self.combo_text.clone() {
        //     text.pos = -text.measure_text() / 2.0;
        //     group.push(text);
        // }


        // hitcircle
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
                radius,
                self.color,
                Some(Border::new(
                    Color::BLACK,
                    self.scaling_helper.border_scaled
                ))
            ));
        }

        

        // make it ripple and add it to the list
        group.ripple_scale_range(0.0, 500.0, time, scale, None, Some(0.5));
        group
    }
}
