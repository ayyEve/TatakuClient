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

    /// note radius (scaled by cs and size)
    radius: f32,
    /// when the hitcircle should start being drawn
    time_preempt: f32,
    /// what is the scaling value? needed for approach circle
    // (lol)
    scaling_helper: Arc<ScalingHelper>,
    
    /// current map time
    map_time: f32,
    /// current mouse pos
    mouse_pos: Vector2,

    /// cached settings for this game
    standard_settings: Arc<StandardSettings>,
    /// list of shapes to be drawn
    shapes: Vec<TransformGroup>,

    circle_image: HitCircleImageHelper,
    approach_circle: ApproachCircle,

    hitsounds: Vec<Hitsound>,
}
impl OsuNote {
    pub async fn new(def:NoteDef, ar:f32, color:Color, combo_num:u16, scaling_helper: Arc<ScalingHelper>, standard_settings:Arc<StandardSettings>, hitsounds: Vec<Hitsound>) -> Self {
        let time = def.time;
        let time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);

        let pos = scaling_helper.scale_coords(def.pos);
        let radius = CIRCLE_RADIUS_BASE * scaling_helper.scaled_cs;
        
        let approach_circle = ApproachCircle::new(def.pos, time, radius, time_preempt, if standard_settings.approach_combo_color { color } else { Color::WHITE }, scaling_helper.clone());
        let circle_image = HitCircleImageHelper::new(
            def.pos,
            scaling_helper.clone(),
            color,
            combo_num
        ).await;

        Self {
            def,
            pos,
            time, 
            color,
            
            hit: false,
            missed: false,

            map_time: 0.0,
            mouse_pos: Vector2::ZERO,
            circle_image,
            time_preempt,
            hitwindow_miss: 0.0,
            radius,
            scaling_helper,

            standard_settings,
            shapes: Vec::new(),
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
        
        self.shapes.push(self.circle_image.ripple(self.map_time));
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
        
        self.shapes.retain_mut(|shape| {
            shape.update(beatmap_time);
            shape.visible()
        });
    }

    async fn draw(&mut self, list: &mut RenderableCollection) {

        // if its not time to draw anything else, leave
        if self.time - self.map_time > self.time_preempt || self.time + self.hitwindow_miss < self.map_time || self.hit { 
            // draw shapes
            for shape in self.shapes.iter() {
                list.push(shape.clone())
            }
            
            return 
        }

        let alpha = self.get_alpha();

        // note
        self.circle_image.set_alpha(alpha);
        self.circle_image.draw(list);

        // timing circle
        self.approach_circle.draw(list);

        // draw shapes
        for shape in self.shapes.iter() {
            list.push(shape.clone())
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
        self.circle_image.reload_skin().await;
        self.approach_circle.reload_texture().await;
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
            let mut group = TransformGroup::new(self.pos).alpha(0.0).border_alpha(1.0);

            group.push(Circle::new(
                Vector2::ZERO,
                self.radius,
                Color::TRANSPARENT_WHITE,
                Some(Border::new(self.color, 2.0))
            ));

            let duration = 500.0;
            group.ripple(0.0, duration, time, self.standard_settings.ripple_scale, true, None);

            self.shapes.push(group);
        }

        self.ripple_start();
    }

    async fn playfield_changed(&mut self, new_scale: Arc<ScalingHelper>) {
        self.pos = new_scale.scale_coords(self.def.pos);
        self.radius = CIRCLE_RADIUS_BASE * new_scale.scaled_cs;
        self.scaling_helper = new_scale.clone();
        self.approach_circle.scale_changed(new_scale, self.radius);
        self.circle_image.playfield_changed(&self.scaling_helper);
    }

    
    fn pos_at(&self, _time: f32) -> Vector2 {
        self.pos
    }

    async fn set_settings(&mut self, settings: Arc<StandardSettings>) {
        self.standard_settings = settings;
    }

    fn set_ar(&mut self, ar: f32) {
        self.time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);
    }

    fn get_hitsound(&self) -> Vec<Hitsound> {
        self.hitsounds.clone()
    }
}
