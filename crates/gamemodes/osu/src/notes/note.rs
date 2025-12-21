use crate::prelude::*;
use tataku::{
    Color,
    Easing,
    Vector2,
};

use engine::{
    beatmaps::{
        osu::*,
        NoteType,
        map_difficulty,
    },
    gameplay::{
        Hitsound,
        HitObject,
    }
};

#[derive(Default)]
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
    standard_settings: Arc<OsuSettings>,

    #[cfg(feature="graphics")] circle_image: HitCircle,
    #[cfg(feature="graphics")] approach_circle: ApproachCircle,

    hitsounds: Vec<Hitsound>,
}
impl OsuNote {
    pub fn new(
        def: NoteDef,
        ar: f32,
        combo_num: u16,
        scaling_helper: Arc<ScalingHelper>, 
        standard_settings: Arc<OsuSettings>,
        hitsounds: Vec<Hitsound>,
    ) -> Self {
        let time = def.time;
        let time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);

        let pos = scaling_helper.scale_coords(def.pos);
        let radius = CIRCLE_RADIUS_BASE * scaling_helper.cs;
        
        Self {
            pos,
            time,
            radius,
            time_preempt,
            standard_settings,
            
            #[cfg(feature="graphics")]
            circle_image: HitCircle::new(
                def.pos,
                scaling_helper.clone(),
                combo_num
            ),

            #[cfg(feature="graphics")]
            approach_circle: ApproachCircle::new(
                def.pos,
                time,
                radius,
                time_preempt,
                scaling_helper.clone()
            ),

            def,
            scaling_helper,
            hitsounds,

            ..Self::default()
        }
    }

    fn get_alpha(&self) -> u8 {
        // fade in
        let mut alpha = (1.0 - ((self.time - (self.time_preempt * (2.0/3.0))) - self.map_time) / (self.time_preempt * (1.0/3.0))) / 3.0;

        // if after time, fade out
        if self.map_time >= self.time {
            alpha = ((self.time + self.hitwindow_miss) - self.map_time) / self.hitwindow_miss;
        }

        Color::to_u8(alpha.clamp(0.0, 1.0))
    }
}

impl HitObject for OsuNote {
    fn note_type(&self) -> NoteType { NoteType::Note }
    fn time(&self) -> f32 { self.time }
    fn end_time(&self, hw_miss:f32) -> f32 { self.time + hw_miss }
    fn update(&mut self, beatmap_time: f32) {
        self.map_time = beatmap_time;
        #[cfg(feature="graphics")]
        self.approach_circle.update(beatmap_time);
        #[cfg(feature="graphics")]
        self.circle_image.update(beatmap_time);
    }

    #[cfg(feature="graphics")]
    fn draw(
        &mut self, 
        _beatmap_time: f32, 
        list: &mut tataku_graphics::RenderableCollection
    ) {

        // if its not time to draw anything else, leave
        if self.time - self.map_time > self.time_preempt || self.time + self.hitwindow_miss < self.map_time || self.hit {
            return
        }

        let alpha = self.get_alpha();

        // note
        self.circle_image.set_alpha(alpha);
        self.circle_image.draw(list);

        // timing circle
        self.approach_circle.set_alpha(alpha);
        self.approach_circle.draw(list);

        // draw shapes
        // for shape in self.shapes.iter() {
        //     list.push(shape.clone());
        // }
    }

    fn reset(&mut self) {
        self.hit = false;
        self.missed = false;

        #[cfg(feature="graphics")]
        self.approach_circle.reset();
    }

    fn time_jump(&mut self, new_time: f32) {
        if new_time > self.time {
            self.hit = true;
            self.missed = true;
        } else {
            self.hit = false;
            self.missed = false;
        }
    }


    #[cfg(feature="graphics")]
    fn reload_skin(
        &mut self, 
        source: &tataku_graphics::TextureSource, 
        skin_manager: &mut dyn tataku_graphics::SkinProvider
    ) {
        self.circle_image.reload_skin(source, skin_manager);
        self.approach_circle.reload_texture(source, skin_manager);
    }
}

impl OsuHitObject for OsuNote {
    fn miss(&mut self) { self.missed = true }
    fn was_hit(&self) -> bool { self.hit || self.missed }
    fn mouse_move(&mut self, pos: Vector2) { self.mouse_pos = pos }
    fn get_preempt(&self) -> f32 { self.time_preempt }
    fn new_combo(&self) -> bool { self.def.new_combo }
    fn pos_at(&self, _time: f32) -> Vector2 { self.pos }
    fn set_hitwindow_miss(&mut self, window: f32) {
        self.hitwindow_miss = window;
    }

    fn check_distance(&self, _mouse_pos: Vector2) -> bool {
        let distance = (self.pos.x - self.mouse_pos.x).powi(2) + (self.pos.y - self.mouse_pos.y).powi(2);
        distance <= self.radius.powi(2)
    }

    fn hit(&mut self, _time: f32) {
        self.hit = true;

        // if self.standard_settings.hit_ripples {
        //     let mut group = TransformGroup::new(self.pos).alpha(0.0).border_alpha(1.0);

        //     group.push(Circle::new(
        //         Vector2::ZERO,
        //         self.radius,
        //         Color::TRANSPARENT,
        //     ).border(Border::new(self.color, 2.0)));

        //     let duration = 500.0;
        //     group.ripple(
        //         0.0,
        //         duration,
        //         time,
        //         self.standard_settings.ripple_scale,
        //         true,
        //         None
        //     );

        //     self.shapes.push(group);
        // }

        // self.ripple_start();
    }

    #[cfg(feature="graphics")]
    fn playfield_changed(&mut self, new_scale: Arc<ScalingHelper>) {
        self.pos = new_scale.scale_coords(self.def.pos);
        self.radius = CIRCLE_RADIUS_BASE * new_scale.cs;
        self.scaling_helper = new_scale.clone();

        #[cfg(feature="graphics")] {
            self.approach_circle.scale_changed(new_scale, self.radius);
            self.circle_image.playfield_changed(&self.scaling_helper);
        }
    }

    fn set_settings(&mut self, settings: Arc<OsuSettings>) {
        self.standard_settings = settings;
    }

    fn set_ar(&mut self, ar: f32) {
        self.time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);
    }

    #[cfg(feature="graphics")]
    fn point_draw_pos(&self, _: f32) -> Vector2 { self.pos }
    #[cfg(feature="graphics")] 
    fn set_combo_color(&mut self, color: Color) {
        self.color = color;
        self.circle_image.set_color(color);
        if self.standard_settings.approach_combo_color {
            self.approach_circle.set_color(color);
        }
    }

    #[cfg(feature="graphics")]
    fn set_approach_easing(&mut self, easing: Easing) {
        self.approach_circle.easing_type = easing;
    }

    #[cfg(feature="gameplay")]
    fn get_hitsound(&self) -> Vec<Hitsound> {
        self.hitsounds.clone()
    }

    #[cfg(feature="graphics")]
    fn shake(&mut self, time: f32) { self.circle_image.shake(time) }
}
