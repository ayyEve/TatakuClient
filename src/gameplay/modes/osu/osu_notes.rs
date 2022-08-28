use crate::prelude::*;

use super::OsuHitJudgments;

const SPINNER_RADIUS:f64 = 200.0;
const SLIDER_DOT_RADIUS:f64 = 8.0;

pub const NOTE_BORDER_SIZE:f64 = 2.0;
pub const CIRCLE_RADIUS_BASE:f64 = 64.0;
const APPROACH_CIRCLE_MULT:f64 = 4.0;
const PREEMPT_MIN:f32 = 450.0;

// temp var for testing alternate slider rendering
const USE_BROKEN_SLIDERS:bool = false;

#[async_trait]
pub trait StandardHitObject: HitObject {
    /// return the window-scaled coords of this object at time
    fn pos_at(&self, time:f32) -> Vector2;
    /// does this object count as a miss if it is not hit?
    fn causes_miss(&self) -> bool; //TODO: might change this to return an enum of "no", "yes". "yes_combo_only" 
    // fn get_points(&mut self, is_press:bool, time:f32, hit_windows:(f32,f32,f32,f32)) -> ScoreHit;


    /// return negative for combo break
    fn pending_combo(&mut self) -> Vec<OsuHitJudgments> {Vec::new()}

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

        let mut combo_text =  Box::new(Text::new(
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
            image.center_text(Rectangle::bounds_only(
                self.pos - Vector2::one() * self.radius / 2.0,
                Vector2::one() * self.radius,
            ));
           image.initial_scale = Vector2::one() * self.scaling_helper.scaled_cs;
           image.current_pos   = image.initial_pos;
           image.current_scale = image.initial_scale;
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
    pending_combo: Vec<OsuHitJudgments>,

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
    /// alpha multiplier, used for background game
    alpha_mult: f32,

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


    // lines_cache: Vec<Box<Line>>,
    // circles_cache: Vec<Box<Circle>>
    slider_draw: SliderPath,
    // slider_draw2: SliderPath,


    /// cached settings for this game
    standard_settings: Arc<StandardSettings>,
    /// list of shapes to be drawn
    shapes: Vec<TransformGroup>,


    start_circle_image: Option<HitCircleImageHelper>,
    end_circle_image: Option<Image>,
    slider_reverse_image: Option<Image>,

    hitwindow_miss: f32
}
impl StandardSlider {
    pub async fn new(def:SliderDef, curve:Curve, ar:f32, color:Color, combo_num: u16, scaling_helper:Arc<ScalingHelper>, slider_depth:f64, circle_depth:f64, standard_settings:Arc<StandardSettings>, diff_calc_only: bool) -> Self {
        let time = def.time;
        let time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);
        
        let pos = scaling_helper.scale_coords(def.pos);
        let visual_end_pos = scaling_helper.scale_coords(curve.smooth_lines.last().unwrap().p2);
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

        
        let slider_reverse_image = if diff_calc_only {None} else {SkinManager::get_texture("reversearrow", true).await};

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
            alpha_mult: 1.0,

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
            slider_draw: SliderPath::default(),


            standard_settings,
            shapes: Vec::new(),
            hitwindow_miss: 0.0,

            start_circle_image,
            end_circle_image,
            slider_reverse_image
        };
    
        slider.make_dots().await;
        slider.make_body().await;
        slider
    }

    async fn make_body(&mut self) {
        let mut side1_total = Vec::new();
        let mut side2_total = Vec::new();

        for segment in self.curve.path.iter() {
            let mut side1 = Vec::new();
            let mut side2 = Vec::new();


            macro_rules! check_sides {
                ($p1:expr, $direction:expr, $perpendicular1:expr, $perpendicular2:expr) => {{
                    let s1 = $p1 + $perpendicular1;
                    let s2 = $p1 + $perpendicular2;
                    let origin = $p1;
                    
                    if side1_total.len() > 0 {
                        let last_point = *side1_total.last().unwrap();
                        let middle_of_curve = origin + $direction * self.radius;

                        let (center, radius, t_initial, t_final) = circle_through_points(last_point, middle_of_curve, s1);
                        let curve_length = ((t_final - t_initial) * radius).abs();
                        let segments = (curve_length * 0.125) as u32;

                        let mut curve = Vec::new();
                        curve.push(last_point);

                        for i in 0..segments {
                            let progress = i as f64 / segments as f64;
                            let t = t_final * progress + t_initial * (1.0 - progress);
                            let new_point = circle_point(center, radius, t);
                            side1.push(new_point);
                        }
                    }

                    if side2_total.len() > 0 {
                        let last_point = *side2_total.last().unwrap();
                        let middle_of_curve = origin + $direction * self.radius;

                        let (center, radius, t_initial, t_final) = circle_through_points(last_point, middle_of_curve, s2);
                        let curve_length = ((t_final - t_initial) * radius).abs();
                        let segments = (curve_length * 0.125) as u32;

                        let mut curve = Vec::new();
                        curve.push(last_point);

                        for i in 0..segments {
                            let progress = i as f64 / segments as f64;
                            let t = t_final * progress + t_initial * (1.0 - progress);
                            let new_point = circle_point(center, radius, t);
                            side2.push(new_point);
                        }
                    }

                    side1.push(s1);
                    side2.push(s2);
                }}
            }

            match segment {
                CurveSegment::Bezier { curve } 
                | CurveSegment::Catmull { curve }
                | CurveSegment::Perfect { curve } => {
                    for i in 1..curve.len() {
                        let p1 = self.scaling_helper.scale_coords(curve[i - 1]);
                        let p2 = self.scaling_helper.scale_coords(curve[i]);

                        let direction = Vector2::normalize(p2 - p1);
                        let perpendicular1 = Vector2::new(direction.y, -direction.x) * self.radius;
                        let perpendicular2 = Vector2::new(-direction.y, direction.x) * self.radius;

                        // if this is the first entry in this list
                        if i == 1 {
                            check_sides!(p1, direction, perpendicular1, perpendicular2)
                        }
                        side1.push(p2 + perpendicular1);
                        side2.push(p2 + perpendicular2);
                    }
                },

                &CurveSegment::Linear { p1, p2 } => {
                    let p1 = self.scaling_helper.scale_coords(p1);
                    let p2 = self.scaling_helper.scale_coords(p2);

                    let direction = Vector2::normalize(p2 - p1);
                    let perpendicular1 = Vector2::new(direction.y, -direction.x) * self.radius;
                    let perpendicular2 = Vector2::new(-direction.y, direction.x) * self.radius;

                    check_sides!(p1, direction, perpendicular1, perpendicular2);
                },
            }

            side1_total.extend(side1.iter());
            side2_total.extend(side2.iter());
        }

        // for (i, line) in self.curve.smooth_lines.iter().enumerate() {
        //     let p1 = self.scaling_helper.scale_coords(line.p1);
        //     let p2 = self.scaling_helper.scale_coords(line.p2);

        //     let direction = Vector2::normalize(p2 - p1);
        //     let perpendicular1 = Vector2::new(direction.y, -direction.x);
        //     let perpendicular2 = Vector2::new(-direction.y, direction.x);

        //     // if this is the first entry in the list
        //     if i == 0 {
        //         side1.push(p1 + perpendicular1 * self.radius);
        //         side2.push(p1 + perpendicular2 * self.radius);
        //         // og_path.push(p1);
        //     }
        //     side1.push(p2 + perpendicular1 * self.radius);
        //     side2.push(p2 + perpendicular2 * self.radius);
        //     // og_path.push(p2);
        // }

        let mut full:Vec<Vector2> = Vec::new();
        // full.extend(start_cap);
        full.extend(side1_total.iter());
        full.extend(side2_total.iter().rev());

        // snippy(&og_path, &mut full, self.radius);

        self.slider_draw = SliderPath::new(full, Color::BLUE, self.slider_depth)
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
        let distance = ((self.slider_ball_pos.x - self.mouse_pos.x).powi(2) + (self.slider_ball_pos.y - self.mouse_pos.y).powi(2)).sqrt();
        self.sliding_ok = self.holding && distance <= self.radius * 2.0;

        if self.time - beatmap_time > self.time_preempt || self.curve.end_time < beatmap_time { return }

        // check if the start of the slider was missed.
        // if it was, perform a miss
        if !self.start_checked && beatmap_time >= self.time + self.hitwindow_miss {
            self.start_checked = true;
            self.start_judgment = OsuHitJudgments::Miss;
            self.pending_combo.insert(0, OsuHitJudgments::Miss);
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

            // increment index
            self.sound_index += 1;

            // check cursor
            if self.sliding_ok {
                self.pending_combo.push(OsuHitJudgments::SliderEnd);
                self.sound_queue.push((
                    beatmap_time,
                    self.get_hitsound(),
                    self.get_hitsamples().clone()
                ));
                self.add_ripple(beatmap_time, self.slider_ball_pos, false);
            } else {
                // set it to negative, we broke combo
                self.pending_combo.push(OsuHitJudgments::SliderEndMiss);
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
                    
                    self.pending_combo.push(OsuHitJudgments::SliderDot);
                    self.sound_queue.push((
                        beatmap_time,
                        0,
                        hitsamples.clone()
                    ));
                } else {
                    self.pending_combo.push(OsuHitJudgments::SliderDotMiss);
                    self.dots_missed += 1
                }
            }
        }
        self.hit_dots = dots;
    }

    async fn draw(&mut self, _args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list:Vec<Box<dyn Renderable>> = Vec::new();

        // draw shapes
        for shape in self.shapes.iter_mut() {
            shape.draw(&mut list)
        }

        // if its not time to draw anything else, leave
        if self.time - self.map_time > self.time_preempt || self.map_time > self.curve.end_time + self.hitwindow_miss {return list}
        
        let mut alpha = (1.0 - ((self.time - (self.time_preempt * (2.0/3.0))) - self.map_time) / (self.time_preempt * (1.0/3.0))).clamp(0.0, 1.0);
        if self.map_time >= self.curve.end_time {
            alpha = ((self.curve.end_time + self.hitwindow_miss) - self.map_time) / self.hitwindow_miss;
        }
        
        let alpha = alpha * self.alpha_mult;
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



        if USE_BROKEN_SLIDERS {
            self.slider_draw.color.a = alpha;
            list.push(Box::new(self.slider_draw.clone()));

            for line in self.curve.smooth_lines.iter() {
                let p1 = self.scaling_helper.scale_coords(line.p1);
                let p2 = self.scaling_helper.scale_coords(line.p2);
                let line = Line::new(p1, p2, 6.0, -999999.9, Color::YELLOW);
                list.push(Box::new(line));
            }

        } else {
            let mut color = color.alpha(alpha);
            const DARKER:f32 = 2.0/3.0;
            color.r *= DARKER;
            color.g *= DARKER;
            color.b *= DARKER;

            const BORDER_RADIUS: f64 = 6.0;
            // border
            for line in self.curve.smooth_lines.iter() {
                let p1 = self.scaling_helper.scale_coords(line.p1);
                let p2 = self.scaling_helper.scale_coords(line.p2);
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

            for line in self.curve.smooth_lines.iter() {
                let p1 = self.scaling_helper.scale_coords(line.p1);
                let p2 = self.scaling_helper.scale_coords(line.p2);

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
            
            // add extra circle to start of slider as well
            list.push(Box::new(Circle::new(
                color,
                self.slider_depth,
                self.scaling_helper.scale_coords(self.curve.smooth_lines[0].p1),
                self.radius,
                None
            )))
        }


        // for line in self.curve.path.iter() {
        //     let p1 = self.scaling_helper.scale_coords(line.p1);
        //     let p2 = self.scaling_helper.scale_coords(line.p2);
            
        //     let line = Line::new(
        //         p1,
        //         p2,
        //         5.0,
        //         self.slider_depth - 1.0,
        //         Color::YELLOW
        //     );
        //     list.push(Box::new(line));
        // }


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

                    let l = self.curve.smooth_lines[self.curve.smooth_lines.len() - 1];
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

                    let l = self.curve.smooth_lines[0];
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

    // // called on hit and release
    // fn get_points(&mut self, is_press:bool, time:f32, (h_miss, h50, h100, h300):(f32,f32,f32,f32)) -> ScoreHit {
    //     // if slider was held to end, no hitwindow to check
    //     if h_miss == -1.0 {
    //         // let distance = ((self.time_end_pos.x - self.mouse_pos.x).powi(2) + (self.time_end_pos.y - self.mouse_pos.y).powi(2)).sqrt();

    //         // #[cfg(feature="debug_sliders")] {
    //         //     trace!("checking end window (held to end)");
    //         //     if distance > self.radius * 2.0 {trace!("slider end miss (out of radius)")}
    //         //     if !self.holding {trace!("slider end miss (not held)")}
    //         // }
            

    //         return self.check_end_points(time);

    //         // self.end_checked = true;
    //         // self.start_checked = true;

    //         // return if distance > self.radius * 2.0 || !self.holding {
    //         //     ScoreHit::Miss
    //         // } else {
    //         //     self.sound_index = self.def.edge_sounds.len() - 1;
    //         //     ScoreHit::X300
    //         // }
    //     }

    //     // check press
    //     if time > self.time - h_miss && time < self.time + h_miss {
    //         // within starting time frame

    //         // make sure the cursor is in the radius
    //         let distance = ((self.pos.x - self.mouse_pos.x).powi(2) + (self.pos.y - self.mouse_pos.y).powi(2)).sqrt();

    //         #[cfg(feature="debug_sliders")] {
    //             trace!("checking start window");
    //             if distance > self.radius * 2.0 {trace!("slider end miss (out of radius)")}
    //         }

    //         // if already hit, or this is a release, return None
    //         if self.start_checked || !is_press || distance > self.radius {return ScoreHit::None}
            
    //         // start wasnt hit yet, set it to true
    //         self.start_checked = true;
    //         // self.sound_index += 1;
            
    //         // get the points
    //         let diff = (time - self.time).abs();

    //         let ripple_pos = if self.end_checked {self.visual_end_pos} else {self.pos};

    //         let score = if diff < h300 {
    //             self.add_ripple(time, ripple_pos, false);
    //             ScoreHit::X300
    //         } else if diff < h100 {
    //             self.add_ripple(time, ripple_pos, false);
    //             ScoreHit::X100
    //         } else if diff < h50 {
    //             self.add_ripple(time, ripple_pos, false);
    //             ScoreHit::X50
    //         } else {
    //             ScoreHit::Miss
    //         };

    //         self.start_judgment = score;
    //         score
    //     } else 

    //     // check release
    //     if time > self.curve.end_time - h_miss && time < self.curve.end_time + h_miss {
    //         // within ending time frame
    //         #[cfg(feature="debug_sliders")]
    //         trace!("checking end window");

    //         // make sure the cursor is in the radius
    //         let distance = ((self.time_end_pos.x - self.mouse_pos.x).powi(2) + (self.time_end_pos.y - self.mouse_pos.y).powi(2)).sqrt();

    //         // if already hit, return None
    //         if self.end_checked || distance > self.radius * 2.0 {return ScoreHit::None}

    //         // make sure the last hitsound in the list is played
    //         self.sound_index = self.def.edge_sounds.len() - 1;

    //         self.check_end_points(time)
    //     } 
    //     // not in either time frame, exit
    //     else {
    //         ScoreHit::None
    //     }

    // }


    fn get_sound_queue(&mut self) -> Vec<(f32, u8, HitSamples)> {
        std::mem::take(&mut self.sound_queue)
    }
    fn pending_combo(&mut self) -> Vec<OsuHitJudgments> {
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
            image.center_text(Rectangle::bounds_only(
                self.pos - Vector2::one() * self.radius / 2.0,
                Vector2::one() * self.radius,
            ));
           image.initial_scale = Vector2::one() * self.scaling_helper.scaled_cs;
           image.current_pos   = image.initial_pos;
           image.current_scale = image.initial_scale;
        }

        self.combo_text = Some(combo_text);
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
pub struct SliderPath {
    path: Vec<[f64; 2]>,
    geom: Vec<[[f64;2]; 3]>,
    color: Color,
    depth: f64
}
impl SliderPath {
    fn new(path: Vec<Vector2>, color: Color, depth: f64,) -> Self {

        if !USE_BROKEN_SLIDERS {
            return Self {
                path: Vec::new(),
                geom: Vec::new(),
                color: Color::WHITE,
                depth: 0.0
            }
        }


        macro_rules! point {
            ($v: expr) => {
                lyon_tessellation::geom::Point::new($v.x as f32, $v.y as f32)
            }
        }

        use lyon_tessellation::*;
        use lyon_tessellation::geometry_builder::simple_builder;
        use lyon_tessellation::math::Point;

        let mut path_builder = lyon_tessellation::path::Path::builder();
        path_builder.begin(point!(path[0]));

        for i in 1..path.len() {
            path_builder.line_to(point!(path[i]));
        }
        path_builder.end(true);

        let path2 = path_builder.build();

        
        let mut buffers: &mut VertexBuffers<Point, u16> = &mut VertexBuffers::new();
        {
            // Create the destination vertex and index buffers.
            let mut vertex_builder = simple_builder(&mut buffers);
        
            // Create the tessellator.
            let mut tessellator = FillTessellator::new();

            let mut fill_options = FillOptions::default();
            fill_options.fill_rule = FillRule::NonZero;
        
            // Compute the tessellation.
            let result = tessellator.tessellate_path(
                path2.as_slice(), //.path_iter().flattened(0.05),
                &fill_options,
                &mut vertex_builder
            );
            assert!(result.is_ok());
        }

        let mut geom = Vec::new();
        for i in (0..buffers.indices.len()).step_by(3) {
            let i1 = buffers.indices[i + 0];
            let i2 = buffers.indices[i + 1];
            let i3 = buffers.indices[i + 2];

            let v1 = buffers.vertices[i1 as usize];
            let v2 = buffers.vertices[i2 as usize];
            let v3 = buffers.vertices[i3 as usize];

            let p1 = [v1.x as f64, v1.y as f64];
            let p2 = [v2.x as f64, v2.y as f64];
            let p3 = [v3.x as f64, v3.y as f64];

            geom.push([p1, p2, p3]);
        }


        let path = path.iter().map(|a|(*a).into()).collect();
        Self {path, color, depth, geom}
    }
}
impl Renderable for SliderPath {
    fn get_depth(&self) -> f64 {self.depth}

    fn draw(&self, g: &mut opengl_graphics::GlGraphics, c:graphics::Context) {
        for tri in self.geom.iter() {
            graphics::polygon(self.color.into(), tri, c.transform, g);
        }

        // outline
        for i in 0..self.path.len() - 1 {
            graphics::line(
                Color::BLACK.into(),
                1.0,
                [
                    self.path[i][0], self.path[i][1],
                    self.path[i+1][0], self.path[i+1][1],
                ],
                c.transform,
                g
            )
        }
    }
}
impl Default for SliderPath {
    fn default() -> Self {
        Self { 
            path: Default::default(), 
            geom: Default::default(), 
            color: Color::WHITE, 
            depth: Default::default() 
        }
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

