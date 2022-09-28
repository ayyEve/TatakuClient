use graphics::Graphics;

use crate::prelude::*;

use super::{ OsuHitJudgments, osu::ScalingHelper };

const SPINNER_RADIUS:f64 = 200.0;
const SLIDER_DOT_RADIUS:f64 = 8.0;

pub const NOTE_BORDER_SIZE:f64 = 2.0;
pub const CIRCLE_RADIUS_BASE:f64 = 64.0;
const APPROACH_CIRCLE_MULT:f64 = 4.0;
const PREEMPT_MIN:f32 = 450.0;

#[async_trait]
pub trait StandardHitObject: HitObject {
    /// return the window-scaled coords of this object at time
    fn pos_at(&self, time:f32) -> Vector2;
    /// does this object count as a miss if it is not hit?
    fn causes_miss(&self) -> bool; //TODO: might change this to return an enum of "no", "yes". "yes_combo_only" 
    // fn get_points(&mut self, is_press:bool, time:f32, hit_windows:(f32,f32,f32,f32)) -> ScoreHit;


    /// return negative for combo break
    fn pending_combo(&mut self) -> Vec<(OsuHitJudgments, Vector2)> {Vec::new()}

    async fn playfield_changed(&mut self, new_scale: Arc<ScalingHelper>);

    fn press(&mut self, _time:f32) {}
    fn release(&mut self, _time:f32) {}
    fn mouse_move(&mut self, pos:Vector2);

    fn get_preempt(&self) -> f32;
    fn point_draw_pos(&self, time: f32) -> Vector2;

    fn was_hit(&self) -> bool;


    fn get_hitsound(&self) -> u8;
    fn get_hitsamples(&self) -> HitSamples;
    fn get_sound_queue(&mut self) -> Vec<(f32, u8, HitSamples)> {vec![]}

    fn set_hitwindow_miss(&mut self, window: f32);


    fn miss(&mut self);
    fn hit(&mut self, time: f32);
    fn set_judgment(&mut self, _j:&OsuHitJudgments) {}

    fn check_distance(&self, mouse_pos: Vector2) -> bool;
    fn check_release_points(&mut self, _time: f32) -> OsuHitJudgments { OsuHitJudgments::Miss } // miss default, bc we only care about sliders

}


// note
#[derive(Clone)]
pub struct StandardNote {
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
    combo_text: Option<Box<Text>>,


    /// current map time
    map_time: f32,
    /// current mouse pos
    mouse_pos: Vector2,

    /// alpha multiplier, used for background game
    alpha_mult: f32,

    /// cached settings for this game
    standard_settings: Arc<StandardSettings>,
    /// list of shapes to be drawn
    shapes: Vec<TransformGroup>,

    circle_image: Option<HitCircleImageHelper>,
    combo_image: Option<SkinnedNumber>,
}
impl StandardNote {
    pub async fn new(def:NoteDef, ar:f32, color:Color, combo_num:u16, scaling_helper: Arc<ScalingHelper>, base_depth:f64, standard_settings:Arc<StandardSettings>, diff_calc_only:bool) -> Self {
        let time = def.time;
        let time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);

        let pos = scaling_helper.scale_coords(def.pos);
        let radius = CIRCLE_RADIUS_BASE * scaling_helper.scaled_cs;

        let combo_text = if diff_calc_only {None} else {
            let mut combo_text =  Box::new(Text::new(
                Color::BLACK,
                base_depth - 0.0000001,
                pos,
                radius as u32,
                format!("{}", combo_num),
                get_font()
            ));
            combo_text.center_text(Rectangle::bounds_only(
                pos - Vector2::one() * radius / 2.0,
                Vector2::one() * radius,
            ));

            Some(combo_text)
        };

        
        let mut combo_image = if diff_calc_only {None} else {
            SkinnedNumber::new(
            Color::WHITE,  // TODO: setting: colored same as note or just white?
            combo_text.as_ref().unwrap().depth, 
            combo_text.as_ref().unwrap().current_pos, 
            combo_num as f64,
            "default",
            None,
            0
        ).await.ok()};
        if let Some(combo) = &mut combo_image {
            combo.center_text(Rectangle::bounds_only(
                pos - Vector2::one() * radius / 2.0,
                Vector2::one() * radius,
            ));
        }


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
            circle_image: if diff_calc_only {None} else {HitCircleImageHelper::new(pos, &scaling_helper, base_depth, color).await},

            time_preempt,
            hitwindow_miss: 0.0,
            radius,
            scaling_helper,
            alpha_mult: 1.0,
            
            combo_text,

            standard_settings,
            shapes: Vec::new(),
            combo_image
        }
    }

}
#[async_trait]
impl HitObject for StandardNote {
    fn note_type(&self) -> NoteType {NoteType::Note}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self, hw_miss:f32) -> f32 {self.time + hw_miss}
    async fn update(&mut self, beatmap_time: f32) {
        self.map_time = beatmap_time;
        
        let time = beatmap_time as f64;
        self.shapes.retain_mut(|shape| {
            shape.update(time);
            shape.items.find(|di|di.visible()).is_some()
        });
    }

    async fn draw(&mut self, _args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list:Vec<Box<dyn Renderable>> = Vec::new();

        // draw shapes
        for shape in self.shapes.iter_mut() {
            shape.draw(&mut list)
        }

        // if its not time to draw anything else, leave
        if self.time - self.map_time > self.time_preempt || self.time + self.hitwindow_miss < self.map_time || self.hit {return list}

        // fade im
        let mut alpha = (1.0 - ((self.time - (self.time_preempt * (2.0/3.0))) - self.map_time) / (self.time_preempt * (1.0/3.0))).clamp(0.0, 1.0);

        // if after time, fade out
        if self.map_time >= self.time {
            alpha = ((self.time + self.hitwindow_miss) - self.map_time) / self.hitwindow_miss;
            // debug!("fading out: {}", alpha)
        }

        alpha *= self.alpha_mult;
        if let Some(image) = &mut self.circle_image {
            image.set_alpha(alpha);
        }

        // timing circle
        let approach_circle_color = if self.standard_settings.approach_combo_color {self.color} else {Color::WHITE};
        list.push(approach_circle(self.pos, self.radius, self.time - self.map_time, self.time_preempt, self.base_depth, self.scaling_helper.scaled_cs, alpha, approach_circle_color).await);


        // combo number
        if let Some(combo) = &mut self.combo_image {
            combo.current_color.a = alpha;
            list.push(Box::new(combo.clone()));
        } else {
            self.combo_text.as_mut().unwrap().current_color.a = alpha;
            list.push(self.combo_text.clone().unwrap());
        }

        // note
        if let Some(image) = &mut self.circle_image {
            image.draw(&mut list);
        } else {
            list.push(Box::new(Circle::new(
                self.color.alpha(alpha),
                self.base_depth,
                self.pos,
                self.radius,
                Some(Border::new(Color::BLACK.alpha(alpha), NOTE_BORDER_SIZE * self.scaling_helper.scale))
            )));
        }

        list
    }

    async fn reset(&mut self) {
        self.hit = false;
        self.missed = false;
        
        self.shapes.clear();
    }
}
#[async_trait]
impl StandardHitObject for StandardNote {
    fn miss(&mut self) {self.missed = true}
    fn was_hit(&self) -> bool {self.hit || self.missed}
    fn get_hitsamples(&self) -> HitSamples {self.def.hitsamples.clone()}
    fn get_hitsound(&self) -> u8 {self.def.hitsound}
    fn point_draw_pos(&self, _: f32) -> Vector2 {self.pos}
    fn causes_miss(&self) -> bool {true}
    fn mouse_move(&mut self, pos:Vector2) {self.mouse_pos = pos}
    fn get_preempt(&self) -> f32 {self.time_preempt}
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
            let mut group = TransformGroup::new();

            group.items.push(DrawItem::Circle(Circle::new(
                Color::TRANSPARENT_WHITE,
                self.base_depth,
                self.pos,
                self.radius,
                Some(Border::new(self.color, 2.0))
            )));

            let duration = 500.0;
            group.ripple(0.0, duration, time as f64, self.standard_settings.ripple_scale, true, None);

            self.shapes.push(group);
        }
    }

    async fn playfield_changed(&mut self, new_scale: Arc<ScalingHelper>) {
        self.pos = new_scale.scale_coords(self.def.pos);
        self.radius = CIRCLE_RADIUS_BASE * new_scale.scaled_cs;
        self.scaling_helper = new_scale;

        let mut combo_text = Box::new(Text::new(
            Color::BLACK,
            self.base_depth - 0.0000001,
            self.pos,
            self.radius as u32,
            format!("{}", self.combo_num),
            get_font()
        ));
        combo_text.center_text(Rectangle::bounds_only(
            self.pos - Vector2::one() * self.radius / 2.0,
            Vector2::one() * self.radius,
        ));

        
        if let Some(image) = &mut self.circle_image {
            image.playfield_changed(&self.scaling_helper)
        }
        
        if let Some(image) = &mut self.combo_image {
            image.initial_scale = Vector2::one() * self.scaling_helper.scaled_cs;
            image.current_scale = image.initial_scale;

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
}




// slider
#[derive(Clone)]
pub struct StandardSlider {
    /// slider definition for this slider
    def: SliderDef,
    /// curve that defines the slider
    curve: Curve,

    /// start pos
    pos: Vector2,
    /// visual end pos
    visual_end_pos: Vector2,
    /// time end pos
    time_end_pos: Vector2,

    /// hit dots. if the slider isnt being held for these
    hit_dots: Vec<SliderDot>,

    /// used for repeat sliders
    pending_combo: Vec<(OsuHitJudgments, Vector2)>,

    /// start time
    time: f32,
    /// what is the current sound index?
    sound_index: usize,
    /// how many slides have been completed?
    slides_complete: u64,
    /// used to check if a slide has been completed
    moving_forward: bool,
    /// song's current time
    map_time: f32,

    /// combo color
    color: Color,
    /// combo number
    combo_num: u16,
    /// note size
    radius: f64,
    
    /// was the start checked?
    start_checked: bool,
    /// was the release checked?
    end_checked: bool,

    /// was a slider dot missed
    dots_missed: usize,
    /// how many dots is there
    dot_count: usize,
    /// what did the user get on the start of the slider?
    start_judgment: OsuHitJudgments,

    /// if the mouse is being held
    holding: bool,
    /// stored mouse pos
    mouse_pos: Vector2,

    /// slider curve depth
    slider_depth: f64,
    /// start/end circle depth
    circle_depth: f64,
    /// when should the note start being drawn (specifically the )
    time_preempt:f32,

    /// combo text cache, probably not needed but whatever
    combo_text: Option<Box<Text>>,
    combo_image: Option<SkinnedNumber>,

    /// list of sounds waiting to be played (used by repeat and slider dot sounds)
    /// (time, hitsound, samples, override sample name)
    sound_queue: Vec<(f32, u8, HitSamples)>,

    /// scaling helper, should greatly improve rendering speed due to locking
    scaling_helper: Arc<ScalingHelper>,

    /// is the mouse in a good state for sliding? (pos + key down)
    sliding_ok: bool,

    /// cached slider ball pos
    slider_ball_pos: Vector2,

    /// cached settings for this game
    standard_settings: Arc<StandardSettings>,
    /// list of shapes to be drawn
    shapes: Vec<TransformGroup>,


    start_circle_image: Option<HitCircleImageHelper>,
    end_circle_image: Option<Image>,
    slider_reverse_image: Option<Image>,

    hitwindow_miss: f32,

    slider_body_render_target: Option<RenderTarget>,
}
impl StandardSlider {
    pub async fn new(def:SliderDef, curve:Curve, ar:f32, color:Color, combo_num: u16, scaling_helper:Arc<ScalingHelper>, slider_depth:f64, circle_depth:f64, standard_settings:Arc<StandardSettings>, diff_calc_only: bool) -> Self {
        let time = def.time;
        let time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);
        
        let pos = scaling_helper.scale_coords(def.pos);
        let visual_end_pos = scaling_helper.scale_coords(curve.curve_lines.last().unwrap().p2);
        let time_end_pos = if def.slides % 2 == 1 {visual_end_pos} else {pos};
        let radius = CIRCLE_RADIUS_BASE * scaling_helper.scaled_cs;

        let combo_text = if diff_calc_only {None} else {
            let mut combo_text =  Box::new(Text::new(
                Color::BLACK,
                circle_depth - 0.0000001,
                pos,
                radius as u32,
                format!("{}", combo_num),
                get_font()
            ));
            combo_text.center_text(Rectangle::bounds_only(
                pos - Vector2::one() * radius / 2.0,
                Vector2::one() * radius,
            ));
            Some(combo_text)
        };

        let start_circle_image = if diff_calc_only {None} else {HitCircleImageHelper::new(pos, &scaling_helper, circle_depth, color).await};
        let end_circle_image = if diff_calc_only {None} else {SkinManager::get_texture("sliderendcircle", true).await};

        let mut combo_image = if diff_calc_only {None} else {SkinnedNumber::new(
            Color::WHITE,  // TODO: setting: colored same as note or just white?
            combo_text.as_ref().unwrap().depth, 
            combo_text.as_ref().unwrap().current_pos, 
            combo_num as f64,
            "default",
            None,
            0
        ).await.ok()};
        if let Some(combo) = &mut combo_image {
            combo.center_text(Rectangle::bounds_only(
                pos - Vector2::one() * radius / 2.0,
                Vector2::one() * radius,
            ));
        }

        
        let slider_reverse_image = if diff_calc_only { None } else { SkinManager::get_texture("reversearrow", true).await };

        let mut slider = Self {
            def,
            curve,
            color,
            combo_num,
            time_preempt,
            slider_depth,
            circle_depth,
            radius,

            pos,
            visual_end_pos,
            time_end_pos,

            time, 
            hit_dots: Vec::new(),
            pending_combo: Vec::new(),
            sound_index: 0,
            slides_complete: 0,
            moving_forward: true,
            map_time: 0.0,

            start_checked: false,
            end_checked: false,
            holding: false,
            mouse_pos: Vector2::zero(),
            
            dots_missed: 0,
            dot_count: 0,
            start_judgment: OsuHitJudgments::Miss,

            combo_text,
            combo_image,
            sound_queue: Vec::new(),

            scaling_helper,
            sliding_ok: false,
            slider_ball_pos: Vector2::zero(),


            standard_settings,
            shapes: Vec::new(),
            hitwindow_miss: 0.0,

            start_circle_image,
            end_circle_image,
            slider_reverse_image,
            slider_body_render_target: None
        };
    
        slider.make_dots().await;
        slider
    }

    async fn make_body(&mut self) {
        let mut list:Vec<Box<dyn Renderable>> = Vec::new();
        let alpha = 1.0;
        let window_size = WindowSize::get().0;
        
        let mut color = self.color.alpha(alpha);
        const DARKER:f32 = 2.0/3.0;
        color.r *= DARKER;
        color.g *= DARKER;
        color.b *= DARKER;

        const BORDER_RADIUS: f64 = 6.0;
        // border
        for line in self.curve.curve_lines.iter() {
            let mut p1 = self.scaling_helper.scale_coords(line.p1);
            let mut p2 = self.scaling_helper.scale_coords(line.p2);

            p1.y = window_size.y - p1.y;
            p2.y = window_size.y - p2.y;

            let border = Line::new(
                p1,
                p2,
                self.radius,
                self.slider_depth,
                Color::WHITE.alpha(alpha)
            );
            list.push(Box::new(border));

            // add a circle to smooth out the corners
            // border
            list.push(Box::new(Circle::new(
                Color::WHITE.alpha(alpha),
                self.slider_depth,
                p2,
                self.radius,
                None
            )));
        }

        for line in self.curve.curve_lines.iter() {
            let mut p1 = self.scaling_helper.scale_coords(line.p1);
            let mut p2 = self.scaling_helper.scale_coords(line.p2);

            p1.y = window_size.y - p1.y;
            p2.y = window_size.y - p2.y;

            let l = Line::new(
                p1,
                p2,
                self.radius - BORDER_RADIUS,
                self.slider_depth,
                color
            );
            list.push(Box::new(l));


            // add a circle to smooth out the corners
            list.push(Box::new(Circle::new(
                color,
                self.slider_depth,
                p2,
                self.radius - BORDER_RADIUS,
                None
            )));
            
        }

        let mut p = self.scaling_helper.scale_coords(self.curve.curve_lines[0].p1);
        p.y = window_size.y - p.y;
        
        // add extra circle to start of slider as well
        list.push(Box::new(Circle::new(
            color,
            self.slider_depth,
            p,
            self.radius,
            None
        )));

        
        let mut slider_body_render_target = RenderTarget::new(window_size.x, window_size.y, |_,_| {}).await.expect("error creating slider body");
        slider_body_render_target.image.origin = Vector2::zero();
        slider_body_render_target.image.depth = self.slider_depth;
        self.slider_body_render_target = Some(slider_body_render_target);


        if let Some(rt) = &mut self.slider_body_render_target {
            rt.update(move |rt, g| {
                let c = g.draw_begin(rt.viewport());
                g.clear_color(Color::TRANSPARENT_WHITE.into());

                for i in list {
                    i.draw(g, c);
                }

                g.draw_end();
            }).await;
        }
    }

    async fn make_dots(&mut self) {
        self.hit_dots.clear();
        self.dot_count = 0;

        let mut slide_counter = 0;
        let mut moving_forwards = true;

        for t in self.curve.score_times.iter() {
            // check for new slide
            let pos = (t - self.time) / (self.curve.length() / self.def.slides as f32);
            let current_moving_forwards = pos % 2.0 <= 1.0;
            if current_moving_forwards != moving_forwards {
                slide_counter += 1;
                moving_forwards = current_moving_forwards;
                // dont add dot if it conflicts with a repeat point
                continue
            }

            // dont add dot if it conflicts with the end circle
            if *t == self.end_time(0.0) {continue}

            let dot = SliderDot::new(
                *t,
                self.scaling_helper.scale_coords(self.curve.position_at_time(*t)),
                self.circle_depth - 0.000001,
                self.scaling_helper.scale,
                slide_counter
            ).await;

            self.dot_count += 1;
            self.hit_dots.push(dot);
        }
    }

    fn add_ripple(&mut self, time: f32, pos: Vector2, is_tick: bool) {
        if self.standard_settings.hit_ripples {
            let mut group = TransformGroup::new();

            // border is white if ripple caused by slider tick
            let border_color = if is_tick {Color::WHITE} else {self.color};

            group.items.push(DrawItem::Circle(Circle::new(
                Color::TRANSPARENT_WHITE,
                self.slider_depth, // slider depth?
                pos,
                self.radius,
                Some(Border::new(border_color, 2.0))
            )));

            let duration = 500.0;
            group.ripple(0.0, duration, time as f64, self.standard_settings.ripple_scale, true, None);

            self.shapes.push(group);
        }
    }

    fn get_alpha(&self) -> f32 {
        let mut alpha = (1.0 - ((self.time - (self.time_preempt * (2.0/3.0))) - self.map_time) / (self.time_preempt * (1.0/3.0))).clamp(0.0, 1.0);
        if self.map_time >= self.curve.end_time {
            alpha = ((self.curve.end_time + self.hitwindow_miss) - self.map_time) / self.hitwindow_miss;
        }
        alpha
    }

}
#[async_trait]
impl HitObject for StandardSlider {
    fn note_type(&self) -> NoteType {NoteType::Slider}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self,_:f32) -> f32 {self.curve.end_time}

    async fn update(&mut self, beatmap_time: f32) {
        self.map_time = beatmap_time;

        // update shapes
        let time = beatmap_time as f64;
        self.shapes.retain_mut(|shape| {
            shape.update(time);
            shape.items.find(|di|di.visible()).is_some()
        });

        // check sliding ok
        self.slider_ball_pos = self.scaling_helper.scale_coords(self.curve.position_at_time(beatmap_time));
        let distance = self.slider_ball_pos.distance(self.mouse_pos); //((self.slider_ball_pos.x - self.mouse_pos.x).powi(2) + (self.slider_ball_pos.y - self.mouse_pos.y).powi(2)).sqrt();
        self.sliding_ok = self.holding && distance <= self.radius * 2.0;

        
        let alpha = self.get_alpha();
        if self.time - beatmap_time > self.time_preempt || self.curve.end_time < beatmap_time {
            if self.slider_body_render_target.is_some() && alpha <= 0.0 {
                self.slider_body_render_target = None;
            }

            return 
        }

        // check if the start of the slider was missed.
        // if it was, perform a miss
        if !self.start_checked && beatmap_time >= self.time + self.hitwindow_miss {
            self.start_checked = true;
            self.start_judgment = OsuHitJudgments::Miss;
            self.pending_combo.insert(0, (OsuHitJudgments::Miss, self.pos));
        }

        // find out if a slide has been completed
        let pos = (beatmap_time - self.time) / (self.curve.length() / self.def.slides as f32);



        let current_moving_forwards = pos % 2.0 <= 1.0;
        if current_moving_forwards != self.moving_forward {
            // direction changed
            self.moving_forward = current_moving_forwards;
            self.slides_complete += 1;
            #[cfg(feature="debug_sliders")]
            debug!("slide complete: {}", self.slides_complete);

            // TODO: calc this properly? use to determine if ripple is on visual start or end
            let pos = self.slider_ball_pos;

            // increment index
            self.sound_index += 1;

            // check cursor
            if self.sliding_ok {
                self.pending_combo.push((OsuHitJudgments::SliderEnd, pos));
                self.sound_queue.push((
                    beatmap_time,
                    self.get_hitsound(),
                    self.get_hitsamples().clone()
                ));
                self.add_ripple(beatmap_time, pos, false);
            } else {
                // we broke combo
                self.pending_combo.push((OsuHitJudgments::SliderEndMiss, pos));
            }
        }

        const SAMPLE_SETS:[&str; 4] = ["normal", "normal", "soft", "drum"];
        let hitsamples = self.get_hitsamples();
        let hitsamples = HitSamples {
            normal_set: 0,
            addition_set: 0,
            index: 0,
            volume: 0,
            filename: Some(format!("{}-slidertick", SAMPLE_SETS[hitsamples.addition_set as usize]))
        };

        let mut dots = std::mem::take(&mut self.hit_dots);
        for dot in dots.iter_mut() {
            if let Some(was_hit) = dot.update(beatmap_time, self.holding) {
                if was_hit {
                    self.add_ripple(beatmap_time, dot.pos, true);
                    
                    self.pending_combo.push((OsuHitJudgments::SliderDot, dot.pos));
                    self.sound_queue.push((
                        beatmap_time,
                        0,
                        hitsamples.clone()
                    ));
                } else {
                    self.pending_combo.push((OsuHitJudgments::SliderDotMiss, dot.pos));
                    self.dots_missed += 1
                }
            }
        }
        self.hit_dots = dots;

        if alpha > 0.0 && self.slider_body_render_target.is_none() {
            self.make_body().await;
        }

    }

    async fn draw(&mut self, _args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list:Vec<Box<dyn Renderable>> = Vec::new();

        // draw shapes
        for shape in self.shapes.iter_mut() {
            shape.draw(&mut list)
        }

        // if its not time to draw anything else, leave
        if self.time - self.map_time > self.time_preempt || self.map_time > self.curve.end_time + self.hitwindow_miss { return list }
        
        
        let alpha = self.get_alpha();
        let color = self.color.alpha(alpha);

        if self.time > self.map_time {
            // timing circle
            let approach_circle_color = if self.standard_settings.approach_combo_color {self.color} else {Color::WHITE};
            list.push(approach_circle(self.pos, self.radius, self.time - self.map_time, self.time_preempt, self.circle_depth, self.scaling_helper.scaled_cs, alpha, approach_circle_color).await);

            // combo number
            if let Some(combo) = &mut self.combo_image {
                combo.current_color.a = alpha;
                list.push(Box::new(combo.clone()));
            } else {
                self.combo_text.as_mut().unwrap().current_color.a = alpha;
                list.push(self.combo_text.clone().unwrap());
            }
        } else if self.map_time < self.curve.end_time {
            // slider ball
            // inner
            list.push(Box::new(Circle::new(
                color,
                self.circle_depth - 0.0000001,
                self.slider_ball_pos,
                self.radius,
                Some(Border::new(Color::WHITE.alpha(alpha), 2.0))
            )));

            list.push(Box::new(Circle::new(
                Color::TRANSPARENT_WHITE,
                self.circle_depth - 0.0000001,
                self.slider_ball_pos,
                self.radius * 2.0,
                Some(Border::new(if self.sliding_ok {Color::LIME} else {Color::RED}.alpha(alpha),2.0)
            ))));
        }


        if let Some(rt) = &self.slider_body_render_target {
            let mut b = rt.image.clone();
            b.current_color.a = alpha;
            list.push(Box::new(b));
        }

        
        // start and end circles
        let slides_remaining = self.def.slides - self.slides_complete;
        let end_repeat = slides_remaining > self.def.slides % 2 + 1;
        let start_repeat = slides_remaining > 2 - self.def.slides % 2;


        // end pos
        if let Some(end_circle) = &self.end_circle_image {
            let mut im = end_circle.clone();
            im.current_color.a = alpha;
            list.push(Box::new(im));

            if end_repeat {
                if let Some(reverse_arrow) = &self.slider_reverse_image {
                    let mut im = reverse_arrow.clone();
                    im.current_pos = self.visual_end_pos;
                    im.depth = self.circle_depth;
                    im.current_color.a = alpha;
                    im.current_scale = Vector2::one() * self.scaling_helper.scaled_cs;

                    let l = self.curve.curve_lines[self.curve.curve_lines.len() - 1];
                    im.current_rotation = Vector2::atan2(l.p1 - l.p2);

                    list.push(Box::new(im));
                }
            }

        } else {
            list.push(Box::new(Circle::new(
                color,
                self.circle_depth, // should be above curves but below slider ball
                self.visual_end_pos,
                self.radius,
                Some(Border::new(
                    if end_repeat {Color::RED} else {Color::BLACK}.alpha(alpha),
                    self.scaling_helper.border_scaled
                ))
            )));
        }

        // start pos
        if let Some(start_circle) = &mut self.start_circle_image {
            start_circle.set_alpha(alpha);
            start_circle.draw(&mut list);
            
            if start_repeat {
                if let Some(reverse_arrow) = &self.slider_reverse_image {
                    let mut im = reverse_arrow.clone();
                    im.current_pos = self.pos;
                    im.depth = self.circle_depth;
                    im.current_color.a = alpha;
                    im.current_scale = Vector2::one() * self.scaling_helper.scaled_cs;

                    let l = self.curve.curve_lines[0];
                    im.current_rotation = Vector2::atan2(l.p2 - l.p1);

                    list.push(Box::new(im));
                }
            }
        } else {
            list.push(Box::new(Circle::new(
                self.color.alpha(alpha),
                self.circle_depth, // should be above curves but below slider ball
                self.pos,
                self.radius,
                Some(Border::new(
                    if start_repeat {Color::RED} else {Color::BLACK}.alpha(alpha),
                    self.scaling_helper.border_scaled
                ))
            )));
        }

        // draw hit dots
        // for dot in self.hit_dots.as_slice() {
        //     if dot.done {continue}
        //     renderables.extend(dot.draw());
        // }

        for dot in self.hit_dots.iter_mut() {
            if dot.slide_layer == self.slides_complete {
                dot.draw(&mut list)
            }
        }

        // for t in self.curve.score_times.iter() {
        //     let pos = self.scaling_helper.scale_coords(self.curve.position_at_time(*t));

        //     let mut c = Circle::new(
        //         Color::WHITE.alpha(alpha),
        //         self.circle_depth, // should be above curves but below slider ball
        //         pos,
        //         SLIDER_DOT_RADIUS * self.scaling_helper.scale
        //     );
        //     c.border = Some(Border::new(
        //         Color::BLACK.alpha(alpha),
        //         self.scaling_helper.border_scaled / 2.0
        //     ));
        //     list.push(Box::new(c))
        // }
        list
    }

    async fn reset(&mut self) {
        self.shapes.clear();
        self.sound_queue.clear();

        self.map_time = 0.0;
        self.holding = false;
        self.start_checked = false;
        self.end_checked = false;
        
        self.pending_combo.clear();
        self.sound_index = 0;
        self.slides_complete = 0;
        self.moving_forward = true;

        self.dots_missed = 0;
        self.dot_count = 0;
        self.start_judgment = OsuHitJudgments::Miss;
        
        self.make_dots().await;
    }
}

#[async_trait]
impl StandardHitObject for StandardSlider {
    fn miss(&mut self) { self.end_checked = true }
    fn was_hit(&self) -> bool { self.end_checked }
    fn get_hitsamples(&self) -> HitSamples {
        let mut samples = self.def.hitsamples.clone();
        let [normal_set, addition_set] = self.def.edge_sets[self.sound_index.min(self.def.edge_sets.len() - 1)];
        samples.normal_set = normal_set;
        samples.addition_set = addition_set;

        samples
    }
    fn get_hitsound(&self) -> u8 {
        // trace!("{}: getting hitsound at index {}/{}", self.time, self.sound_index, self.def.edge_sounds.len() - 1);
        self.def.edge_sounds[self.sound_index.min(self.def.edge_sounds.len() - 1)]
    }
    fn causes_miss(&self) -> bool {false}
    fn point_draw_pos(&self, time: f32) -> Vector2 {self.pos_at(time)}

    fn get_preempt(&self) -> f32 {self.time_preempt}
    fn press(&mut self, _:f32) {self.holding = true}
    fn release(&mut self, _:f32) {self.holding = false}
    fn mouse_move(&mut self, pos:Vector2) {self.mouse_pos = pos}
    fn set_hitwindow_miss(&mut self, window: f32) {
        self.hitwindow_miss = window;
    }

    fn set_judgment(&mut self, j: &OsuHitJudgments) {
        self.start_checked = true;
        self.start_judgment = *j;
    }

    fn hit(&mut self, time: f32) {
        self.start_checked = true;

        if self.standard_settings.hit_ripples {
            self.add_ripple(time, self.pos_at(time), false);
        }
    }

    fn check_release_points(&mut self, time: f32) -> OsuHitJudgments {
        self.end_checked = true;
        self.sound_index = self.def.edge_sounds.len() - 1;

        macro_rules! ripple {
            () => {
                self.add_ripple(time, self.visual_end_pos, false);
            }
        }

        match self.start_judgment {
            OsuHitJudgments::Miss => {
                if self.dot_count == 0 {
                    let distance = ((self.time_end_pos.x - self.mouse_pos.x).powi(2) + (self.time_end_pos.y - self.mouse_pos.y).powi(2)).sqrt();
                    if distance > self.radius * 2.0 || !self.holding {
                        OsuHitJudgments::Miss
                    } else {
                        self.sound_index = self.def.edge_sounds.len() - 1;
                        OsuHitJudgments::X100
                    }

                } else if self.dots_missed == self.dot_count {
                    OsuHitJudgments::Miss
                } else if self.dots_missed == 0 {
                    ripple!();
                    OsuHitJudgments::X100
                } else {
                    ripple!();
                    OsuHitJudgments::X50
                }
            }

            _ => {
                if self.dots_missed == 0 {
                    ripple!();
                    OsuHitJudgments::X300
                } else {
                    ripple!();
                    OsuHitJudgments::X100
                }
            }
        }
    }

    
    fn check_distance(&self, _mouse_pos: Vector2) -> bool {
        let distance = if self.start_checked {
            (self.time_end_pos.x - self.mouse_pos.x).powi(2) + (self.time_end_pos.y - self.mouse_pos.y).powi(2)
        } else {
            (self.pos.x - self.mouse_pos.x).powi(2) + (self.pos.y - self.mouse_pos.y).powi(2)
        };

        distance <= self.radius.powi(2)
    }

    fn get_sound_queue(&mut self) -> Vec<(f32, u8, HitSamples)> {
        std::mem::take(&mut self.sound_queue)
    }
    fn pending_combo(&mut self) -> Vec<(OsuHitJudgments, Vector2)> {
        std::mem::take(&mut self.pending_combo)
    }


    async fn playfield_changed(&mut self, new_scale: Arc<ScalingHelper>) {
        self.scaling_helper = new_scale;
        self.pos = self.scaling_helper.scale_coords(self.def.pos);
        self.radius = CIRCLE_RADIUS_BASE * self.scaling_helper.scaled_cs;
        self.visual_end_pos =  self.scaling_helper.scale_coords(self.curve.position_at_length(self.curve.length()));
        self.time_end_pos = if self.def.slides % 2 == 1 {self.visual_end_pos} else {self.pos};
        
        let mut combo_text =  Box::new(Text::new(
            Color::BLACK,
            self.circle_depth - 0.0000001,
            self.pos,
            self.radius as u32,
            format!("{}", self.combo_num),
            get_font()
        ));
        combo_text.center_text(Rectangle::bounds_only(
            self.pos - Vector2::one() * self.radius / 2.0,
            Vector2::one() * self.radius,
        ));

        if let Some(image) = &mut self.start_circle_image {
            image.playfield_changed(&self.scaling_helper)
        }
        if let Some(image) = &mut self.end_circle_image {
           image.initial_pos   = self.scaling_helper.scale_coords(self.visual_end_pos);
           image.initial_scale = Vector2::one() * self.scaling_helper.scaled_cs;
           image.current_pos   = image.initial_pos;
           image.current_scale = image.initial_scale;
        }

        
        
        if let Some(image) = &mut self.combo_image {
            image.initial_scale = Vector2::one() * self.scaling_helper.scaled_cs;
            image.current_scale = image.initial_scale;

            image.center_text(Rectangle::bounds_only(
                self.pos - Vector2::one() * self.radius / 2.0,
                Vector2::one() * self.radius,
            ));
        }

        self.combo_text = Some(combo_text);
        if self.slider_body_render_target.is_some() {
            self.make_body().await;
        }
        self.make_dots().await;
    }

    fn pos_at(&self, time: f32) -> Vector2 {
        if time >= self.curve.end_time {
            self.time_end_pos
        } else {
            self.scaling_helper.scale_coords(self.curve.position_at_time(time))
        }
    }
}


/// helper struct for drawing hit slider points
#[derive(Clone)]
struct SliderDot {
    time: f32,
    pos: Vector2,
    checked: bool,
    hit: bool,
    depth: f64,
    scale: f64,

    /// which slide "layer" is this on?
    slide_layer: u64,
    dot_image: Option<Image>
}
impl SliderDot {
    pub async fn new(time:f32, pos:Vector2, depth: f64, scale: f64, slide_layer: u64) -> SliderDot {

        SliderDot {
            time,
            pos,
            depth,
            scale,
            slide_layer,

            hit: false,
            checked: false,
            dot_image: SkinManager::get_texture("sliderscorepoint", true).await
        }
    }
    /// returns true if the hitsound should play
    pub fn update(&mut self, beatmap_time:f32, mouse_down: bool) -> Option<bool> {
        if beatmap_time >= self.time && !self.checked {
            self.checked = true;
            self.hit = mouse_down;
            Some(self.hit)
        } else {
            None
        }
    }
    
    pub fn draw(&self, list:&mut Vec<Box<dyn Renderable>>) {
        if self.hit {return}

        if let Some(image) = &self.dot_image {
            let mut image = image.clone();
            image.current_pos = self.pos;
            image.depth = self.depth;
            list.push(Box::new(image));
        } else {
            list.push(Box::new(Circle::new(
                Color::WHITE,
                self.depth,
                self.pos,
                SLIDER_DOT_RADIUS * self.scale,
                Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE * self.scale))
            )));
        }

    }
}



// spinner
#[derive(Clone)]
pub struct StandardSpinner {
    def: SpinnerDef,
    pos: Vector2,
    time: f32, // ms
    end_time: f32, // ms
    last_update: f32,

    /// current angle of the spinner
    rotation: f64,
    /// how fast the spinner is spinning
    rotation_velocity: f64,
    mouse_pos: Vector2,

    /// what was the last rotation value?
    last_rotation_val: f64,
    /// how many rotations is needed to pass this spinner
    rotations_required: u16,
    /// how many rotations have been completed?
    rotations_completed: u16,

    /// should we count mouse movements?
    holding: bool,

    scaling_helper: Arc<ScalingHelper>,


    /// alpha multiplier, used for background game
    alpha_mult: f32,
}
impl StandardSpinner {
    pub fn new(def: SpinnerDef, scaling_helper: Arc<ScalingHelper>, _diff_calc_only: bool) -> Self {
        let time = def.time;
        let end_time = def.end_time;
        Self {
            pos: scaling_helper.window_size / 2.0,
            def,
            time, 
            end_time,

            holding: false,
            rotation: 0.0,
            rotation_velocity: 0.0,
            last_rotation_val: 0.0,
            scaling_helper,

            rotations_required: 0,
            rotations_completed: 0,
            mouse_pos: Vector2::zero(),

            last_update: 0.0,
            alpha_mult: 1.0,
        }
    }
}
#[async_trait]
impl HitObject for StandardSpinner {
    fn time(&self) -> f32 {self.time}
    fn end_time(&self,_:f32) -> f32 {self.end_time}
    fn note_type(&self) -> NoteType {NoteType::Spinner}

    async fn update(&mut self, beatmap_time: f32) {
        let mut diff = 0.0;
        let pos_diff = self.mouse_pos - self.pos;
        let mouse_angle = pos_diff.y.atan2(pos_diff.x);

        if beatmap_time >= self.time && beatmap_time <= self.end_time {
            if self.holding {
                diff = mouse_angle - self.last_rotation_val;
            }
            if diff > PI {diff -= 2.0 * PI}
            else if diff < -PI {diff += 2.0 * PI}
            // debug!("diff: {:.2}", diff / PI);
            
            // self.rotation_velocity = f64::lerp(-diff, self.rotation_velocity, 0.005 * (beatmap_time - self.last_update) as f64);
            self.rotation_velocity = f64::lerp(self.rotation_velocity, diff, 0.005 * (beatmap_time - self.last_update) as f64);
            self.rotation += self.rotation_velocity * (beatmap_time - self.last_update) as f64;
            // debug!("vel: {}", self.rotation_velocity);

            // debug!("rotation: {}, diff: {}", self.rotation, diff);
        }

        self.last_rotation_val = mouse_angle;
        self.last_update = beatmap_time;
    }
    async fn draw(&mut self, _args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        if !(self.last_update >= self.time && self.last_update <= self.end_time) {return list}

        let border = Some(Border::new(Color::BLACK.alpha(self.alpha_mult), NOTE_BORDER_SIZE));

        // bg circle
        list.push(Box::new(Circle::new(
            Color::YELLOW.alpha(self.alpha_mult),
            -10.0,
            self.pos,
            SPINNER_RADIUS,
            border.clone()
        )));

        // draw another circle on top which increases in radius as the counter gets closer to the reqired
        list.push(Box::new(Circle::new(
            Color::WHITE.alpha(self.alpha_mult),
            -11.0,
            self.pos,
            SPINNER_RADIUS * (self.rotations_completed as f64 / self.rotations_required as f64),
            border.clone()
        )));

        // draw line to show rotation
        {
            let p2 = self.pos + Vector2::new(self.rotation.cos(), self.rotation.sin()) * SPINNER_RADIUS;
            list.push(Box::new(Line::new(
                self.pos,
                p2,
                5.0,
                -20.0,
                Color::GREEN.alpha(self.alpha_mult)
            )));
        }
        
        // draw a counter
        let rpm = (self.rotation_velocity / (2.0 * PI)) * 1000.0 * 60.0;
        let mut txt = Text::new(
            Color::BLACK.alpha(self.alpha_mult),
            -999.9,
            Vector2::zero(),
            30,
            format!("{:.0}rpm", rpm.abs()), // format!("{:.0}rpm", rpm.abs()),
            get_font()
        );
        txt.center_text(Rectangle::bounds_only(
            Vector2::new(0.0, self.pos.y + 50.0),
            Vector2::new(self.pos.x * 2.0, 50.0)
        ));
        list.push(Box::new(txt));

        list
    }

    async fn reset(&mut self) {
        self.holding = false;
        self.rotation = 0.0;
        self.rotation_velocity = 0.0;
        self.rotations_completed = 0;
    }
}
#[async_trait]
impl StandardHitObject for StandardSpinner {
    fn miss(&mut self) {}
    fn was_hit(&self) -> bool {self.last_update >= self.end_time} 
    fn get_hitsamples(&self) -> HitSamples {self.def.hitsamples.clone()}
    fn get_hitsound(&self) -> u8 {self.def.hitsound}
    fn get_preempt(&self) -> f32 {0.0}
    fn point_draw_pos(&self, _: f32) -> Vector2 {Vector2::zero()} //TODO
    fn causes_miss(&self) -> bool {self.rotations_completed < self.rotations_required} // if the spinner wasnt completed in time, cause a miss
    fn set_hitwindow_miss(&mut self, _window: f32) {}

    // fn get_points(&mut self, _is_press:bool, _:f32, _:(f32,f32,f32,f32)) -> ScoreHit {
    //     ScoreHit::Other(100, false)
    // }

    fn press(&mut self, _time:f32) {
        self.holding = true;
    }
    fn release(&mut self, _time:f32) {
        self.holding = false;
    }
    fn mouse_move(&mut self, pos:Vector2) {
        self.mouse_pos = pos;
    }

    async fn playfield_changed(&mut self, new_scale: Arc<ScalingHelper>) {
        self.scaling_helper = new_scale;
        self.pos =  self.scaling_helper.window_size / 2.0
    } 

    fn pos_at(&self, time: f32) -> Vector2 {
        // debug!("time: {}, {}, {}", time, self.time, self.end_time);

        if time < self.time || time >= self.end_time {
            
            return self.pos
        }

        let r = self.last_rotation_val + (time - self.last_update) as f64 / (4.0*PI);
        self.pos + Vector2::new(
            r.cos(),
            r.sin()
        ) * self.scaling_helper.scale * 20.0
    }


    fn hit(&mut self, _time: f32) {}
    fn check_distance(&self, _:Vector2) -> bool { true }
}


async fn approach_circle(pos:Vector2, radius:f64, time_diff:f32, time_preempt:f32, depth:f64, scaled_cs:f64, alpha: f32, color: Color) -> Box<dyn Renderable> {

    if let Some(mut tex) = SkinManager::get_texture("approachcircle", true).await {
        tex.depth = depth - 100.0;
        let scale = 1.0 + (time_diff as f64 / time_preempt as f64) * (APPROACH_CIRCLE_MULT - 1.0);

        tex.initial_pos = pos;
        tex.initial_color = color.alpha(alpha);
        tex.initial_scale = Vector2::one() * scale * scaled_cs;

        tex.current_pos = tex.initial_pos;
        tex.current_color = tex.initial_color;
        tex.current_scale = tex.initial_scale;

        Box::new(tex)
    } else {
        Box::new(Circle::new(
            Color::TRANSPARENT_WHITE,
            depth - 100.0,
            pos,
            radius + (time_diff as f64 / time_preempt as f64) * (APPROACH_CIRCLE_MULT * CIRCLE_RADIUS_BASE * scaled_cs),
            Some(Border::new(color.alpha(alpha), NOTE_BORDER_SIZE * scaled_cs))
        ))
    }
}


#[derive(Clone)]
struct HitCircleImageHelper {
    pos: Vector2,
    circle: Image,
    overlay: Image,
}
impl HitCircleImageHelper {
    async fn new(pos: Vector2, scaling_helper: &Arc<ScalingHelper>, depth: f64, color: Color) -> Option<Self> {
        let mut circle = SkinManager::get_texture("hitcircle", true).await;
        if let Some(circle) = &mut circle {
            circle.depth = depth;
            circle.initial_pos = pos;
            circle.initial_scale = Vector2::one() * scaling_helper.scaled_cs;
            circle.initial_color = color;
            
            circle.current_pos = circle.initial_pos;
            circle.current_scale = circle.initial_scale;
            circle.current_color = circle.initial_color;
        }

        let mut overlay = SkinManager::get_texture("hitcircleoverlay", true).await;
        if let Some(overlay) = &mut overlay {
            overlay.depth = depth - 0.0000001;
            overlay.initial_pos = pos;
            overlay.initial_scale = Vector2::one() * scaling_helper.scaled_cs;
            // overlay.initial_color = color;
            
            overlay.current_pos = overlay.initial_pos;
            overlay.current_scale = overlay.initial_scale;
            // overlay.current_color = overlay.initial_color;
        }

        if overlay.is_none() || circle.is_none() {return None}

        Some(Self {
            circle: circle.unwrap(),
            overlay: overlay.unwrap(),
            pos: scaling_helper.descale_coords(pos)
        })
    }

    
    fn playfield_changed(&mut self, new_scale: &Arc<ScalingHelper>) {
        self.overlay.initial_pos = new_scale.scale_coords(self.pos);
        self.overlay.initial_scale = Vector2::one() * new_scale.scaled_cs;
        self.overlay.current_pos = self.overlay.initial_pos;
        self.overlay.current_scale = self.overlay.initial_scale;

        self.circle.initial_pos   = self.overlay.initial_pos;
        self.circle.initial_scale = self.overlay.initial_scale;
        self.circle.current_pos   = self.overlay.initial_pos;
        self.circle.current_scale = self.overlay.initial_scale;
    }

    fn set_alpha(&mut self, alpha: f32) {
        self.circle.current_color.a = alpha;
        self.overlay.current_color.a = alpha;
    }

    fn draw(&mut self, list: &mut Vec<Box<dyn Renderable>>) {
        list.push(Box::new(self.circle.clone()));
        list.push(Box::new(self.overlay.clone()));
    }
}

