use crate::prelude::*;
use super::super::prelude::*;

pub struct OsuNote {
    /// note definition
    def: NoteDef,
    /// note position
    pos: Vector2,
    /// note time in ms
    time: f32,

    hitwindow_miss: f32,

    /// was the note hit?
    hit: bool,
    /// was the note missed?
    missed: bool,

    /// combo color
    color: Color, 
    /// combo number
    combo_num: u16,

    /// note depth
    base_depth: f64,
    /// note radius (scaled by cs and size)
    radius: f64,
    /// when the hitcircle should start being drawn
    time_preempt: f32,
    /// what is the scaling value? needed for approach circle
    // (lol)
    scaling_helper: Arc<ScalingHelper>,
    
    /// combo num text cache
    combo_text: Option<Text>,


    /// current map time
    map_time: f32,
    /// current mouse pos
    mouse_pos: Vector2,

    /// cached settings for this game
    standard_settings: Arc<StandardSettings>,
    /// list of shapes to be drawn
    shapes: Vec<TransformGroup>,

    circle_image: Option<HitCircleImageHelper>,
    combo_image: Option<SkinnedNumber>,
    approach_circle: ApproachCircle,

    hitsounds: Vec<Hitsound>,
}
impl OsuNote {
    pub async fn new(def:NoteDef, ar:f32, color:Color, combo_num:u16, scaling_helper: Arc<ScalingHelper>, base_depth:f64, standard_settings:Arc<StandardSettings>, hitsounds: Vec<Hitsound>) -> Self {
        let time = def.time;
        let time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);

        let pos = scaling_helper.scale_coords(def.pos);
        let radius = CIRCLE_RADIUS_BASE * scaling_helper.scaled_cs;
        
        let approach_circle = ApproachCircle::new(def.pos, time, radius, time_preempt, base_depth, if standard_settings.approach_combo_color { color } else { Color::WHITE }, scaling_helper.clone());

        Self {
            def,
            pos,
            time, 
            base_depth,
            color,
            combo_num,
            
            hit: false,
            missed: false,

            map_time: 0.0,
            mouse_pos: Vector2::zero(),
            circle_image: None,
            time_preempt,
            hitwindow_miss: 0.0,
            radius,
            scaling_helper,
            
            combo_text: None,

            standard_settings,
            shapes: Vec::new(),
            combo_image: None,
            approach_circle,

            hitsounds
        }
    }

    fn get_alpha(&self) -> f32 {
        // fade im
        let mut alpha = (1.0 - ((self.time - (self.time_preempt * (2.0/3.0))) - self.map_time) / (self.time_preempt * (1.0/3.0))).clamp(0.0, 1.0);

        // if after time, fade out
        if self.map_time >= self.time {
            alpha = ((self.time + self.hitwindow_miss) - self.map_time) / self.hitwindow_miss;
            // debug!("fading out: {}", alpha)
        }
        alpha
    }

    fn ripple_start(&mut self) {
        if !self.standard_settings.ripple_hitcircles { return }
        let scale = 1.0..1.5;

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
        let mut circle_group = TransformGroup::new(self.pos, self.base_depth).alpha(1.0).border_alpha(1.0);
        if let Some(circle) = &self.circle_image {
            let mut i_circle = circle.circle.clone();
            i_circle.pos = Vector2::ZERO;
            let mut i_overlay = circle.overlay.clone();
            i_overlay.pos = Vector2::ZERO;

            circle_group.push(i_circle);
            circle_group.push(i_overlay);
        } else {
            circle_group.push(Circle::new(
                self.color,
                self.base_depth, // should be above curves but below slider ball
                Vector2::ZERO,
                self.radius,
                Some(Border::new(
                    Color::BLACK,
                    self.scaling_helper.border_scaled
                ))
            ));
        }

        // make it ripple and add it to the list
        circle_group.ripple_scale_range(0.0, 500.0, self.map_time as f64, scale, None, Some(0.5));
        self.shapes.push(circle_group);
    }
}
#[async_trait]
impl HitObject for OsuNote {
    fn note_type(&self) -> NoteType { NoteType::Note }
    fn time(&self) -> f32 { self.time }
    fn end_time(&self, hw_miss:f32) -> f32 { self.time + hw_miss }
    async fn update(&mut self, beatmap_time: f32) {
        self.map_time = beatmap_time;
        self.approach_circle.update(beatmap_time, self.get_alpha());
        
        let time = beatmap_time as f64;
        self.shapes.retain_mut(|shape| {
            shape.update(time);
            shape.visible()
        });
    }

    async fn draw(&mut self, _args:RenderArgs, list: &mut RenderableCollection) {
        // draw shapes
        for shape in self.shapes.iter_mut() {
            // shape.draw(list)
            list.push(shape.clone())
        }

        // if its not time to draw anything else, leave
        if self.time - self.map_time > self.time_preempt || self.time + self.hitwindow_miss < self.map_time || self.hit { return }

        let alpha = self.get_alpha();

        if let Some(image) = &mut self.circle_image {
            image.set_alpha(alpha);
        }

        // timing circle
        self.approach_circle.draw(list);


        // combo number
        if let Some(combo) = &mut self.combo_image {
            combo.color.a = alpha;
            list.push(combo.clone());
        } else {
            self.combo_text.as_mut().unwrap().color.a = alpha;
            list.push(self.combo_text.clone().unwrap());
        }

        // note
        if let Some(image) = &mut self.circle_image {
            image.draw(list);
        } else {
            list.push(Circle::new(
                self.color.alpha(alpha),
                self.base_depth,
                self.pos,
                self.radius,
                Some(Border::new(Color::BLACK.alpha(alpha), NOTE_BORDER_SIZE * self.scaling_helper.scale))
            ));
        }

    }

    async fn reset(&mut self) {
        self.hit = false;
        self.missed = false;
        
        self.shapes.clear();
        self.approach_circle.reset();
    }

    async fn time_jump(&mut self, new_time: f32) {
        if new_time > self.time {
            self.hit = true;
            self.missed = true;
        } else {
            self.hit = false;
            self.missed = false;
        }
    }

    
    async fn reload_skin(&mut self) {
        self.circle_image = HitCircleImageHelper::new(self.pos, &self.scaling_helper, self.base_depth, self.color).await;
        self.approach_circle.reload_texture().await;
    
        self.combo_text = Some(Text::new(
            Color::BLACK,
            self.base_depth - 0.0000001,
            self.pos,
            self.radius as u32,
            format!("{}", self.combo_num),
            get_font()
        ));
        self.combo_text.as_mut().unwrap().center_text(&Rectangle::bounds_only(
            self.pos - Vector2::one() * self.radius / 2.0,
            Vector2::one() * self.radius,
        ));

        let mut combo_image = SkinnedNumber::new(
            Color::WHITE,  // TODO: setting: colored same as note or just white?
            self.base_depth - 0.0000001, 
            self.pos, 
            self.combo_num as f64,
            "default",
            None,
            0
        ).await.ok();
        if let Some(combo) = &mut combo_image {
            combo.center_text(Rectangle::bounds_only(
                self.pos - Vector2::one() * self.radius / 2.0,
                Vector2::one() * self.radius,
            ));
        }
        self.combo_image = combo_image;
    }
}

#[async_trait]
impl OsuHitObject for OsuNote {
    fn miss(&mut self) { self.missed = true }
    fn was_hit(&self) -> bool { self.hit || self.missed }
    fn point_draw_pos(&self, _: f32) -> Vector2 { self.pos }
    fn mouse_move(&mut self, pos:Vector2) { self.mouse_pos = pos }
    fn get_preempt(&self) -> f32 { self.time_preempt }
    fn set_hitwindow_miss(&mut self, window: f32) {
        self.hitwindow_miss = window;
    }

    fn check_distance(&self, _mouse_pos: Vector2) -> bool {
        let distance = (self.pos.x - self.mouse_pos.x).powi(2) + (self.pos.y - self.mouse_pos.y).powi(2);
        distance <= self.radius.powi(2)
    }

    fn hit(&mut self, time: f32) {
        self.hit = true;

        if self.standard_settings.hit_ripples {
            let mut group = TransformGroup::new(self.pos, self.base_depth).alpha(0.0).border_alpha(1.0);

            group.push(Circle::new(
                Color::TRANSPARENT_WHITE,
                0.0,
                Vector2::ZERO,
                self.radius,
                Some(Border::new(self.color, 2.0))
            ));

            let duration = 500.0;
            group.ripple(0.0, duration, time as f64, self.standard_settings.ripple_scale, true, None);

            self.shapes.push(group);
        }

        self.ripple_start();
    }

    async fn playfield_changed(&mut self, new_scale: Arc<ScalingHelper>) {
        self.pos = new_scale.scale_coords(self.def.pos);
        self.radius = CIRCLE_RADIUS_BASE * new_scale.scaled_cs;
        self.scaling_helper = new_scale.clone();
        self.approach_circle.scale_changed(new_scale);

        let mut combo_text = Text::new(
            Color::BLACK,
            self.base_depth - 0.0000001,
            self.pos,
            self.radius as u32,
            format!("{}", self.combo_num),
            get_font()
        );
        combo_text.center_text(&Rectangle::bounds_only(
            self.pos - Vector2::one() * self.radius / 2.0,
            Vector2::one() * self.radius,
        ));

        
        if let Some(image) = &mut self.circle_image {
            image.playfield_changed(&self.scaling_helper)
        }
        
        if let Some(image) = &mut self.combo_image {
            image.scale = Vector2::one() * self.scaling_helper.scaled_cs;

            image.center_text(Rectangle::bounds_only(
                self.pos - Vector2::one() * self.radius / 2.0,
                Vector2::one() * self.radius,
            ));
        }

        self.combo_text = Some(combo_text);
    }

    
    fn pos_at(&self, _time: f32) -> Vector2 {
        self.pos
    }

    fn set_settings(&mut self, settings: Arc<StandardSettings>) {
        self.standard_settings = settings;
    }

    fn set_ar(&mut self, ar: f32) {
        self.time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);
    }

    fn get_hitsound(&self) -> Vec<Hitsound> {
        self.hitsounds.clone()
    }
}
