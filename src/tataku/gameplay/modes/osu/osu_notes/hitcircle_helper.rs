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
    depth: f32,
    color: Color,
    
    /// combo num text cache
    combo_text: Option<Text>,
    combo_image: Option<SkinnedNumber>,

    skin_settings: Arc<SkinSettings>,
}
impl HitCircleImageHelper {
    pub async fn new(base_pos: Vector2, scaling_helper: Arc<ScalingHelper>, depth: f32, color: Color, combo_num: u16) -> Self {
        let skin_settings = SkinManager::current_skin_config().await;
        
        let mut s = Self {
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
            depth
        };
        s.reload_skin().await;
        s
    }

    pub async fn reload_skin(&mut self) {
        self.skin_settings = SkinManager::current_skin_config().await;
        let radius = CIRCLE_RADIUS_BASE * self.scaling_helper.scaled_cs;

        self.circle = SkinManager::get_texture("hitcircle", true).await;
        if let Some(circle) = &mut self.circle {
            circle.depth = self.depth;
            circle.pos = self.pos;
            circle.scale = Vector2::ONE * self.scaling_helper.scaled_cs;
            circle.color = self.color;
        }
        
        self.overlay = SkinManager::get_texture("hitcircleoverlay", true).await;
        if let Some(overlay) = &mut self.overlay {
            overlay.depth = self.depth - 0.0000001;
            overlay.pos = self.pos;
            overlay.scale = Vector2::ONE * self.scaling_helper.scaled_cs;
        }

        let rect = Rectangle::bounds_only(self.pos - Vector2::ONE * radius / 2.0, Vector2::ONE * radius);

        self.combo_image = SkinnedNumber::new(
            Color::WHITE, 
            self.depth - 0.0000001, 
            self.pos, 
            self.combo_num as f64,
            &self.skin_settings.hitcircle_prefix,
            None,
            0
        ).await.ok();

        if let Some(combo) = &mut self.combo_image {
            combo.spacing_override = Some(-(self.skin_settings.hitcircle_overlap as f32));
            combo.scale = Vector2::ONE * self.scaling_helper.scaled_cs * TEXT_SCALE;
            combo.center_text(&rect);
            self.combo_text = None;
        } else if let Some(_text) = &mut self.combo_text {

        } else {
            let mut text = Text::new(
                Color::BLACK,
                self.depth - 0.0000001,
                self.pos,
                radius,
                self.combo_num.to_string(),
                get_font()
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
        let rect = Rectangle::bounds_only(self.pos - Vector2::ONE * radius / 2.0, Vector2::ONE * radius);
        
        if let Some(image) = &mut self.combo_image {
            image.spacing_override = Some(-(self.skin_settings.hitcircle_overlap as f32));
            image.scale = scale * TEXT_SCALE;
            image.center_text(&rect);
        }
        if let Some(text) = &mut self.combo_text {
            text.font_size = radius as f32;
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
                self.color.alpha(self.alpha),
                self.depth,
                self.pos,
                radius,
                Some(Border::new(
                    Color::BLACK.alpha(self.alpha),
                    self.scaling_helper.border_scaled
                ))
            ));
        }

        if let Some(mut overlay) = self.overlay.clone() {
            overlay.color.a = self.alpha;
            list.push(overlay);
        } else {}

        if let Some(mut image) = self.combo_image.clone() {
            image.color.a = self.alpha;
            list.push(image);

        } else if let Some(mut text) = self.combo_text.clone() {
            text.color.a = self.alpha;
            list.push(text);
        }

    }


    pub fn ripple(&self, time: f32) -> TransformGroup {
        let scale = 0.0..1.3;
        let radius = CIRCLE_RADIUS_BASE * self.scaling_helper.scaled_cs;

        // broken
        // // combo text
        // let mut combo_group = TransformGroup::new();
        // if let Some(mut c) = self.combo_image.clone() {
        //     c.origin = c.measure_text() / 2.0;
        //     c.current_pos -= c.origin;
        //     combo_group.items.push(DrawItem::SkinnedNumber(c));
        // } else {
        //     combo_group.items.push(DrawItem::Text(*self.combo_text.as_ref().unwrap().clone()));
        // }
        // combo_group.ripple_scale_range(0.0, 500.0, self.map_time as f64, scale.clone(), None, Some(0.8));
        // self.shapes.push(combo_group);


        // hitcircle
        let mut circle_group = TransformGroup::new(self.pos, self.depth).alpha(1.0).border_alpha(1.0);

        if let Some(mut i_circle) = self.circle.clone() {
            i_circle.pos = Vector2::ZERO;
            circle_group.push(i_circle);
        }

        if let Some(mut i_overlay) = self.overlay.clone() {
            i_overlay.pos = Vector2::ZERO;
            circle_group.push(i_overlay);
        }
        
        if circle_group.items.len() == 0 {
            circle_group.push(Circle::new(
                self.color,
                self.depth,
                Vector2::ZERO,
                radius,
                Some(Border::new(
                    Color::BLACK,
                    self.scaling_helper.border_scaled
                ))
            ));
        }

        

        // make it ripple and add it to the list
        circle_group.ripple_scale_range(0.0, 500.0, time, scale, None, Some(0.5));
        circle_group
    }
}
