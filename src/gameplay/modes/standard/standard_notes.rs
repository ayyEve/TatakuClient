use crate::prelude::*;

const SPINNER_RADIUS:f64 = 200.0;
const SLIDER_DOT_RADIUS:f64 = 8.0;

pub const NOTE_BORDER_SIZE:f64 = 2.0;
pub const CIRCLE_RADIUS_BASE:f64 = 64.0;
const HITWINDOW_CIRCLE_RADIUS:f64 = CIRCLE_RADIUS_BASE * 2.0;
const PREEMPT_MIN:f32 = 450.0;


pub trait StandardHitObject: HitObject {
    /// return the window-scaled coords of this object at time
    fn pos_at(&self, time:f32, scaling_helper:&ScalingHelper) -> Vector2;
    /// does this object count as a miss if it is not hit?
    fn causes_miss(&self) -> bool; //TODO: might change this to return an enum of "no", "yes". "yes_combo_only" 
    fn get_points(&mut self, is_press:bool, time:f32, hit_windows:(f32,f32,f32,f32)) -> ScoreHit;
    /// return negative for combo break
    fn pending_combo(&mut self) -> i8 {0}

    fn playfield_changed(&mut self, new_scale: &ScalingHelper);

    fn press(&mut self, _time:f32) {}
    fn release(&mut self, _time:f32) {}
    fn mouse_move(&mut self, pos:Vector2);

    fn get_preempt(&self) -> f32;
    fn point_draw_pos(&self, time: f32) -> Vector2;

    fn was_hit(&self) -> bool;

    fn miss(&mut self);
    fn get_hitsound(&self) -> u8;
    fn get_hitsamples(&self) -> HitSamples;
    fn get_sound_queue(&mut self) -> Vec<(f32, u8, HitSamples)> {vec![]}


    fn set_hitwindow_miss(&mut self, window: f32);
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
    scaling_scale: f64,
    
    /// combo num text cache
    combo_text: Box<Text>,


    /// current map time
    map_time: f32,
    /// current mouse pos
    mouse_pos: Vector2,

    /// alpha multiplier, used for background game
    alpha_mult: f32,

    /// cached settings for this game
    standard_settings: Arc<StandardSettings>,
    /// list of shapes to be drawn
    shapes: Vec<TransformGroup>
}
impl StandardNote {
    pub fn new(def:NoteDef, ar:f32, color:Color, combo_num:u16, scaling_helper:&ScalingHelper, base_depth:f64, standard_settings:Arc<StandardSettings>) -> Self {
        let time = def.time;
        let time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);

        let pos = scaling_helper.scale_coords(def.pos);
        let radius = CIRCLE_RADIUS_BASE * scaling_helper.scaled_cs;

        let mut combo_text =  Box::new(Text::new(
            Color::BLACK,
            base_depth - 0.0000001,
            pos,
            (radius) as u32,
            format!("{}", combo_num),
            get_font("main")
        ));
        combo_text.center_text(Rectangle::bounds_only(
            pos - Vector2::one() * radius / 2.0,
            Vector2::one() * radius,
        ));

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

            time_preempt,
            hitwindow_miss: 0.0,
            radius,
            scaling_scale: scaling_helper.scale,
            alpha_mult: 1.0,
            
            combo_text,

            standard_settings,
            shapes: Vec::new()
        }
    }

    fn do_hit(&mut self, time: f32) {
        self.hit = true;

        if self.standard_settings.hit_ripples {
            let mut group = TransformGroup::new();

            let mut ripple = Circle::new(
                Color::TRANSPARENT_WHITE,
                self.base_depth,
                self.pos,
                self.radius
            );
            ripple.border = Some(Border::new(self.color, 2.0));
            group.items.push(DrawItem::Circle(ripple));

            let duration = 500.0;
            group.ripple(0.0, duration, time as f64, self.standard_settings.ripple_scale, true, None);

            self.shapes.push(group);
        }
    }
}
impl HitObject for StandardNote {
    fn note_type(&self) -> NoteType {NoteType::Note}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self, hw_miss:f32) -> f32 {self.time + hw_miss}
    fn update(&mut self, beatmap_time: f32) {
        self.map_time = beatmap_time;
        
        let time = beatmap_time as f64;
        self.shapes.retain_mut(|shape| {
            shape.update(time);
            shape.items.find(|di|di.visible()).is_some()
        });
    }
    fn set_alpha(&mut self, alpha: f32) {self.alpha_mult = alpha}

    fn draw(&mut self, _args:RenderArgs, list:&mut Vec<Box<dyn Renderable>>) {
        // draw shapes
        for shape in self.shapes.iter_mut() {
            shape.draw(list)
        }

        // if its not time to draw anything else, leave
        if self.time - self.map_time > self.time_preempt || self.time + self.hitwindow_miss < self.map_time || self.hit {return}

        // fade im
        let mut alpha = (1.0 - ((self.time - (self.time_preempt * (2.0/3.0))) - self.map_time) / (self.time_preempt * (1.0/3.0))).clamp(0.0, 1.0);

        // if after time, fade out
        if self.map_time >= self.time {
            alpha = ((self.time + self.hitwindow_miss) - self.map_time) / self.hitwindow_miss;
            // println!("fading out: {}", alpha)
        }

        alpha *= self.alpha_mult;

        // timing circle
        let approach_circle_color = if self.standard_settings.approach_combo_color {self.color} else {Color::WHITE};
        list.push(approach_circle(self.pos, self.radius, self.time - self.map_time, self.time_preempt, self.base_depth, self.scaling_scale, alpha, approach_circle_color));


        // combo number
        self.combo_text.color.a = alpha;
        list.push(self.combo_text.clone());

        // note
        let mut note = Circle::new(
            self.color.alpha(alpha),
            self.base_depth,
            self.pos,
            self.radius
        );
        note.border = Some(Border::new(Color::BLACK.alpha(alpha), NOTE_BORDER_SIZE * self.scaling_scale));
        list.push(Box::new(note));

    }

    fn reset(&mut self) {
        self.hit = false;
        self.missed = false;
        
        self.shapes.clear();
    }
}
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

    fn get_points(&mut self, _is_press:bool, time:f32, (hitwindow_miss, hitwindow_50, hitwindow_100, hitwindow_300):(f32,f32,f32,f32)) -> ScoreHit {
        let diff = (time - self.time).abs();
        
        // make sure the cursor is in the radius
        let distance = ((self.pos.x - self.mouse_pos.x).powi(2) + (self.pos.y - self.mouse_pos.y).powi(2)).sqrt();
        if distance > self.radius {return ScoreHit::None}

        if diff < hitwindow_300 {
            self.do_hit(time);
            ScoreHit::X300
        } else if diff < hitwindow_100 {
            self.do_hit(time);
            ScoreHit::X100
        } else if diff < hitwindow_50 {
            self.do_hit(time);
            ScoreHit::X50
        } else if diff < hitwindow_miss { // too early, miss
            self.missed = true;
            ScoreHit::Miss
        } else { // way too early, ignore
            ScoreHit::None
        }
    }

    fn playfield_changed(&mut self, new_scale: &ScalingHelper) {
        self.pos = new_scale.scale_coords(self.def.pos);
        self.radius = CIRCLE_RADIUS_BASE * new_scale.scaled_cs;
        self.scaling_scale = new_scale.scale;

        let mut combo_text =  Box::new(Text::new(
            Color::BLACK,
            self.base_depth - 0.0000001,
            self.pos,
            self.radius as u32,
            format!("{}", self.combo_num),
            get_font("main")
        ));
        combo_text.center_text(Rectangle::bounds_only(
            self.pos - Vector2::one() * self.radius / 2.0,
            Vector2::one() * self.radius,
        ));

        self.combo_text = combo_text;
    }

    
    fn pos_at(&self, _time: f32, _scaling_helper:&ScalingHelper) -> Vector2 {
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
    pending_combo: i8,

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
    start_judgment: ScoreHit,

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
    combo_text: Box<Text>,

    /// list of sounds waiting to be played (used by repeat and slider dot sounds)
    /// (time, hitsound, samples, override sample name)
    sound_queue: Vec<(f32, u8, HitSamples)>,

    /// scaling helper, should greatly improve rendering speed due to locking
    scaling_helper: ScalingHelper,

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

    hitwindow_miss: f32
}
impl StandardSlider {
    pub fn new(def:SliderDef, curve:Curve, ar:f32, color:Color, combo_num: u16, scaling_helper:ScalingHelper, slider_depth:f64, circle_depth:f64, standard_settings:Arc<StandardSettings>) -> Self {
        let time = def.time;
        let time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);
        
        let pos = scaling_helper.scale_coords(def.pos);
        let visual_end_pos = scaling_helper.scale_coords(curve.path.last().unwrap().p2);
        let time_end_pos = if def.slides % 2 == 1 {visual_end_pos} else {pos};
        let radius = CIRCLE_RADIUS_BASE * scaling_helper.scaled_cs;

        let mut combo_text =  Box::new(Text::new(
            Color::BLACK,
            circle_depth - 0.0000001,
            pos,
            radius as u32,
            format!("{}", combo_num),
            get_font("main")
        ));
        combo_text.center_text(Rectangle::bounds_only(
            pos - Vector2::one() * radius / 2.0,
            Vector2::one() * radius,
        ));



        // let mut lines_cache = Vec::new();
        // let mut circles_cache = Vec::new();

        // // curves
        // for i in 0..curve.path.len() {
        //     let line = curve.path[i];

        //     let p1 = scaling_helper.scale_coords(line.p1);
        //     let p2 = scaling_helper.scale_coords(line.p2);
        //     lines_cache.push(Box::new(Line::new(
        //         p1,
        //         p2,
        //         radius,
        //         slider_depth,
        //         color
        //     )));

        //     // add a circle to smooth out the corners
        //     circles_cache.push(Box::new(Circle::new(
        //         color,
        //         slider_depth,
        //         p2,
        //         radius,
        //     )))
        // }



        // let side1_angle = PI / 2.0;
        // let side2_angle = 3.0*PI / 2.0;

        let mut side1 = vec![];
        let mut side2 = vec![];

        for (i, line) in curve.path.iter().enumerate() {
            let p1 = scaling_helper.scale_coords(line.p1);
            let p2 = scaling_helper.scale_coords(line.p2);
            let p3 = scaling_helper.scale_coords(
                if i + 1 < curve.path.len() {
                    curve.path[i + 1].p2
                } else {
                    p2
                }
            );

            let direction = Vector2::normalize(p2 - p1);
            let perpendicular1 = Vector2::new(direction.y, -direction.x);
            let perpendicular2 = Vector2::new(-direction.y, direction.x);
            side1.push(p1 + perpendicular1 * radius);
            // side2.push(p1 + perpendicular2 * radius);
            
            side2.insert(0, p1 + perpendicular2 * radius);

            // let theta = Vector2::atan2(p2 - p1);
            // let s1 = p1 + Vector2::from_angle(theta + side1_angle) * radius;
            // let s2 = p1 + Vector2::from_angle(theta + side2_angle) * radius;
            // side1.push(s1);
            // side2.push(s2);

            if i == curve.path.len() - 1 {
                // let theta = Vector2::atan2(p1 - p2);
                // let s1 = p2 + Vector2::from_angle(theta + side1_angle) * radius;
                // let s2 = p2 + Vector2::from_angle(theta + side2_angle) * radius;
                // side1.push(s2);
                // side2.push(s1);

                let direction = Vector2::normalize(p2 - p1);
                let perpendicular1 = Vector2::new(direction.y, -direction.x);
                let perpendicular2 = Vector2::new(-direction.y, direction.x);
                side1.push(p2 + perpendicular1 * radius);
                // side2.push(p2 + perpendicular2 * radius);
                side2.insert(0, p2 + perpendicular2 * radius);
            }
        }


        let mut full:Vec<Vector2> = Vec::new();
        full.extend(side1.iter());
        full.extend(side2.iter());

        // close the loop
        // full.push(full[0]);

        // if let CurveType::Perfect = def.curve_type {
        //     for i in full.iter() {
        //         println!("{}, {}", i.x, i.y)
        //     }
        //     println!("\n\n")
        //     // println!("{:?}", full.iter().map(|a|(a.x, a.y)).collect::<Vec<(f64, f64)>>());
        //     // println!("{:?}\n\n", aids);
        // }

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
            pending_combo: 0,
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
            start_judgment: ScoreHit::None,

            combo_text,
            sound_queue: Vec::new(),

            scaling_helper,
            sliding_ok: false,
            slider_ball_pos: Vector2::zero(),
            // lines_cache,
            // circles_cache,
            slider_draw: SliderPath::new(full, Color::BLUE, slider_depth),
            // slider_draw2: SliderPath::new(side2, Color::GREEN, slider_depth),


            standard_settings,
            shapes: Vec::new(),
            hitwindow_miss: 0.0,
        };
    
        slider.make_dots();
        slider
    }

    fn make_dots(&mut self) {
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
            );

            self.dot_count += 1;
            self.hit_dots.push(dot);
        }
    }

    
    fn add_ripple(&mut self, time: f32, pos: Vector2, is_tick: bool) {
        if self.standard_settings.hit_ripples {
            let mut group = TransformGroup::new();

            // border is white if ripple caused by slider tick
            let border_color = if is_tick {Color::WHITE} else {self.color};

            let mut ripple = Circle::new(
                Color::TRANSPARENT_WHITE,
                self.slider_depth, // slider depth?
                pos,
                self.radius
            );

            ripple.border = Some(Border::new(border_color, 2.0));
            group.items.push(DrawItem::Circle(ripple));

            let duration = 500.0;
            group.ripple(0.0, duration, time as f64, self.standard_settings.ripple_scale, true, None);

            self.shapes.push(group);
        }
    }


    fn check_end_points(&mut self, time: f32) -> ScoreHit {
        self.end_checked = true;
        self.sound_index = self.def.edge_sounds.len() - 1;

        macro_rules! ripple {
            () => {
                self.add_ripple(time, self.visual_end_pos, false);
            }
        }

        match self.start_judgment {
            ScoreHit::None | ScoreHit::Miss => {
                if self.dot_count == 0 {
                    let distance = ((self.time_end_pos.x - self.mouse_pos.x).powi(2) + (self.time_end_pos.y - self.mouse_pos.y).powi(2)).sqrt();
                    return if distance > self.radius * 2.0 || !self.holding {
                        ScoreHit::Miss
                    } else {
                        self.sound_index = self.def.edge_sounds.len() - 1;
                        ScoreHit::X100
                    }

                } else if self.dots_missed == self.dot_count {
                    ScoreHit::Miss
                } else if self.dots_missed == 0 {
                    ripple!();
                    ScoreHit::X100
                } else {
                    ripple!();
                    ScoreHit::X50
                }
            }

            _ => {
                if self.dots_missed == 0 {
                    ripple!();
                    ScoreHit::X300
                } else {
                    ripple!();
                    ScoreHit::X100
                }
            }
        }
    }
}
impl HitObject for StandardSlider {
    fn note_type(&self) -> NoteType {NoteType::Slider}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self,_:f32) -> f32 {self.curve.end_time}
    fn set_alpha(&mut self, alpha: f32) {self.alpha_mult = alpha}

    fn update(&mut self, beatmap_time: f32) {
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

        if self.time - beatmap_time > self.time_preempt || self.curve.end_time < beatmap_time {return}

        // find out if a slide has been completed
        let pos = (beatmap_time - self.time) / (self.curve.length() / self.def.slides as f32);

        let current_moving_forwards = pos % 2.0 <= 1.0;
        if current_moving_forwards != self.moving_forward {
            // direction changed
            self.moving_forward = current_moving_forwards;
            self.slides_complete += 1;
            #[cfg(feature="debug_sliders")]
            println!("slide complete: {}", self.slides_complete);

            // increment index
            self.sound_index += 1;

            // check cursor
            if self.sliding_ok {
                // increment pending combo
                self.pending_combo += 1;
                self.sound_queue.push((
                    beatmap_time,
                    self.get_hitsound(),
                    self.get_hitsamples().clone()
                ));
                self.add_ripple(beatmap_time, self.slider_ball_pos, false);
            } else {
                // set it to negative, we broke combo
                self.pending_combo = -1;
            }
        }

        const SAMPLE_SETS:[&str; 4] = ["normal", "normal", "soft", "drum"];
        let hitsamples = self.get_hitsamples();
        let hitsamples = HitSamples {
            normal_set: 0,
            addition_set: 0,
            index: 0,
            volume: 0,
            filename: Some(format!("{}-slidertick.wav", SAMPLE_SETS[hitsamples.addition_set as usize]))
        };

        let mut dots = std::mem::take(&mut self.hit_dots);
        for dot in dots.iter_mut() {
            if let Some(was_hit) = dot.update(beatmap_time, self.holding) {
                if was_hit {
                    println!("[Dots] dot hit");
                    self.add_ripple(beatmap_time, dot.pos, true);
                    self.sound_queue.push((
                        beatmap_time,
                        0,
                        hitsamples.clone()
                    ));
                } else {
                    println!("[Dots] dot missed");
                    self.dots_missed += 1
                }
            }
        }
        self.hit_dots = dots;
    }

    fn draw(&mut self, _args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        // draw shapes
        for shape in self.shapes.iter_mut() {
            shape.draw(list)
        }

        // if its not time to draw anything else, leave
        if self.time - self.map_time > self.time_preempt || self.map_time > self.curve.end_time + self.hitwindow_miss {return}

        // let alpha = (self.time_preempt / 4.0) / ((self.time - self.time_preempt / 4.0) - self.map_time).clamp(0.0, 1.0);
        let mut alpha = (1.0 - ((self.time - (self.time_preempt * (2.0/3.0))) - self.map_time) / (self.time_preempt * (1.0/3.0))).clamp(0.0, 1.0);
        
        
        if self.map_time >= self.curve.end_time {
            alpha = ((self.curve.end_time + self.hitwindow_miss) - self.map_time) / self.hitwindow_miss;
        }
        
        let alpha = alpha * self.alpha_mult;
        let color = self.color.alpha(alpha);

        if self.time > self.map_time {
            // timing circle
            let approach_circle_color = if self.standard_settings.approach_combo_color {self.color} else {Color::WHITE};
            list.push(approach_circle(self.pos, self.radius, self.time - self.map_time, self.time_preempt, self.circle_depth, self.scaling_helper.scale, alpha, approach_circle_color));

            // combo number
            self.combo_text.color.a = alpha;
            list.push(self.combo_text.clone());
        } else if self.map_time < self.curve.end_time {
            // slider ball
            let mut inner = Circle::new(
                color,
                self.circle_depth - 0.0000001,
                self.slider_ball_pos,
                self.radius
            );
            inner.border = Some(Border::new(
                Color::WHITE.alpha(alpha),
                2.0
            ));
            list.push(Box::new(inner));


            let mut outer = Circle::new(
                Color::TRANSPARENT_WHITE,
                self.circle_depth - 0.0000001,
                self.slider_ball_pos,
                self.radius* 2.0
            );
            outer.border = Some(Border::new(
                if self.sliding_ok {
                    Color::GREEN
                } else {
                    Color::RED
                }.alpha(alpha),
                2.0
            ));
            list.push(Box::new(outer));
        }



        // curve
        // self.slider_draw.color.a = alpha;
        // list.push(Box::new(self.slider_draw.clone()));


        
        // curve

        for line in self.curve.path.iter() {
            let p1 = self.scaling_helper.scale_coords(line.p1);
            let p2 = self.scaling_helper.scale_coords(line.p2);
            let l = Line::new(
                p1,
                p2,
                self.radius,
                self.slider_depth,
                color
            );
            list.push(Box::new(l));

            // let line = Line::new(
            //     line.p1,
            //     p2,
            //     5.0,
            //     self.slider_depth - 1.0,
            //     Color::YELLOW
            // );
            // list.push(Box::new(line));

            // add a circle to smooth out the corners
            list.push(Box::new(Circle::new(
                color,
                self.slider_depth,
                p2,
                self.radius,
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
        let mut c = Circle::new(
            color,
            self.circle_depth, // should be above curves but below slider ball
            self.visual_end_pos,
            self.radius
        );
        c.border = Some(Border::new(
            if end_repeat {Color::RED} else {Color::BLACK}.alpha(alpha),
            self.scaling_helper.border_scaled
        ));
        list.push(Box::new(c));

        // start pos
        let mut c = Circle::new(
            self.color.alpha(alpha),
            self.circle_depth, // should be above curves but below slider ball
            self.pos,
            self.radius
        );
        c.border = Some(Border::new(
            if start_repeat {Color::RED} else {Color::BLACK}.alpha(alpha),
            self.scaling_helper.border_scaled
        ));
        list.push(Box::new(c));

        // draw hit dots
        // for dot in self.hit_dots.as_slice() {
        //     if dot.done {continue}
        //     renderables.extend(dot.draw());
        // }

        for dot in self.hit_dots.iter_mut() {
            if dot.slide_layer == self.slides_complete {
                dot.draw(list)
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
    }

    fn reset(&mut self) {
        self.shapes.clear();
        self.sound_queue.clear();

        self.map_time = 0.0;
        self.holding = false;
        self.start_checked = false;
        self.end_checked = false;
        
        self.pending_combo = 0;
        self.sound_index = 0;
        self.slides_complete = 0;
        self.moving_forward = true;

        self.dots_missed = 0;
        self.dot_count = 0;
        self.start_judgment = ScoreHit::None;
        
        self.make_dots();
    }
}
impl StandardHitObject for StandardSlider {
    fn miss(&mut self) {self.end_checked = true}
    fn was_hit(&self) -> bool {self.end_checked}
    fn get_hitsamples(&self) -> HitSamples {
        let mut samples = self.def.hitsamples.clone();
        let [normal_set, addition_set] = self.def.edge_sets[self.sound_index.min(self.def.edge_sets.len() - 1)];
        samples.normal_set = normal_set;
        samples.addition_set = addition_set;

        samples
    }
    fn get_hitsound(&self) -> u8 {
        // println!("{}: getting hitsound at index {}/{}", self.time, self.sound_index, self.def.edge_sounds.len() - 1);
        self.def.edge_sounds[self.sound_index.min(self.def.edge_sounds.len() - 1)]
    }
    fn causes_miss(&self) -> bool {false}
    fn point_draw_pos(&self, time: f32) -> Vector2 {
        self.pos_at(time, &self.scaling_helper)
    }
    fn get_preempt(&self) -> f32 {self.time_preempt}
    fn press(&mut self, _:f32) {self.holding = true}
    fn release(&mut self, _:f32) {self.holding = false}
    fn mouse_move(&mut self, pos:Vector2) {self.mouse_pos = pos}
    fn set_hitwindow_miss(&mut self, window: f32) {
        self.hitwindow_miss = window;
    }

    // called on hit and release
    fn get_points(&mut self, is_press:bool, time:f32, (h_miss, h50, h100, h300):(f32,f32,f32,f32)) -> ScoreHit {
        // if slider was held to end, no hitwindow to check
        if h_miss == -1.0 {
            // let distance = ((self.time_end_pos.x - self.mouse_pos.x).powi(2) + (self.time_end_pos.y - self.mouse_pos.y).powi(2)).sqrt();

            // #[cfg(feature="debug_sliders")] {
            //     println!("checking end window (held to end)");
            //     if distance > self.radius * 2.0 {println!("slider end miss (out of radius)")}
            //     if !self.holding {println!("slider end miss (not held)")}
            // }
            

            return self.check_end_points(time);

            // self.end_checked = true;
            // self.start_checked = true;

            // return if distance > self.radius * 2.0 || !self.holding {
            //     ScoreHit::Miss
            // } else {
            //     self.sound_index = self.def.edge_sounds.len() - 1;
            //     ScoreHit::X300
            // }
        }

        // check press
        if time > self.time - h_miss && time < self.time + h_miss {
            // within starting time frame

            // make sure the cursor is in the radius
            let distance = ((self.pos.x - self.mouse_pos.x).powi(2) + (self.pos.y - self.mouse_pos.y).powi(2)).sqrt();

            #[cfg(feature="debug_sliders")] {
                println!("checking start window");
                if distance > self.radius * 2.0 {println!("slider end miss (out of radius)")}
            }

            // if already hit, or this is a release, return None
            if self.start_checked || !is_press || distance > self.radius {return ScoreHit::None}
            
            // start wasnt hit yet, set it to true
            self.start_checked = true;
            // self.sound_index += 1;
            
            // get the points
            let diff = (time - self.time).abs();

            let ripple_pos = if self.end_checked {self.visual_end_pos} else {self.pos};

            let score = if diff < h300 {
                self.add_ripple(time, ripple_pos, false);
                ScoreHit::X300
            } else if diff < h100 {
                self.add_ripple(time, ripple_pos, false);
                ScoreHit::X100
            } else if diff < h50 {
                self.add_ripple(time, ripple_pos, false);
                ScoreHit::X50
            } else {
                ScoreHit::Miss
            };

            self.start_judgment = score;
            score
        } else 

        // check release
        if time > self.curve.end_time - h_miss && time < self.curve.end_time + h_miss {
            // within ending time frame
            #[cfg(feature="debug_sliders")]
            println!("checking end window");

            // make sure the cursor is in the radius
            let distance = ((self.time_end_pos.x - self.mouse_pos.x).powi(2) + (self.time_end_pos.y - self.mouse_pos.y).powi(2)).sqrt();

            // if already hit, return None
            if self.end_checked || distance > self.radius * 2.0 {return ScoreHit::None}

            // make sure the last hitsound in the list is played
            self.sound_index = self.def.edge_sounds.len() - 1;

            self.check_end_points(time)
        } 
        // not in either time frame, exit
        else {
            ScoreHit::None
        }

    }


    fn get_sound_queue(&mut self) -> Vec<(f32, u8, HitSamples)> {
        std::mem::take(&mut self.sound_queue)
    }
    fn pending_combo(&mut self) -> i8 {
        std::mem::take(&mut self.pending_combo)
    }


    fn playfield_changed(&mut self, new_scale: &ScalingHelper) {
        self.scaling_helper = new_scale.clone();
        self.pos = new_scale.scale_coords(self.def.pos);
        self.radius = CIRCLE_RADIUS_BASE * new_scale.scaled_cs;
        self.visual_end_pos = new_scale.scale_coords(self.curve.position_at_length(self.curve.length()));
        self.time_end_pos = if self.def.slides % 2 == 1 {self.visual_end_pos} else {self.pos};
        
        let mut combo_text =  Box::new(Text::new(
            Color::BLACK,
            self.circle_depth - 0.0000001,
            self.pos,
            self.radius as u32,
            format!("{}", self.combo_num),
            get_font("main")
        ));
        combo_text.center_text(Rectangle::bounds_only(
            self.pos - Vector2::one() * self.radius / 2.0,
            Vector2::one() * self.radius,
        ));

        self.combo_text = combo_text;
        self.make_dots();
    }

    fn pos_at(&self, time: f32, scaling_helper:&ScalingHelper) -> Vector2 {
        if time >= self.curve.end_time {
            self.time_end_pos
        } else {
            scaling_helper.scale_coords(self.curve.position_at_time(time))
        }
    }
}

/// helper struct for drawing hit slider points
#[derive(Clone, Copy)]
struct SliderDot {
    time: f32,
    pos: Vector2,
    checked: bool,
    hit: bool,
    depth: f64,
    scale: f64,

    /// which slide "layer" is this on?
    slide_layer: u64,
}
impl SliderDot {
    pub fn new(time:f32, pos:Vector2, depth: f64, scale: f64, slide_layer: u64) -> SliderDot {
        SliderDot {
            time,
            pos,
            depth,
            scale,
            slide_layer,

            hit: false,
            checked: false
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

        let mut c = Circle::new(
            Color::WHITE,
            self.depth,
            self.pos,
            SLIDER_DOT_RADIUS * self.scale
        );
        c.border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE * self.scale));
        list.push(Box::new(c));
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


    /// alpha multiplier, used for background game
    alpha_mult: f32,
}
impl StandardSpinner {
    pub fn new(def: SpinnerDef, scaling_helper: &ScalingHelper) -> Self {
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

            rotations_required: 0,
            rotations_completed: 0,
            mouse_pos: Vector2::zero(),

            last_update: 0.0,
            alpha_mult: 1.0,
        }
    }
}
impl HitObject for StandardSpinner {
    fn time(&self) -> f32 {self.time}
    fn end_time(&self,_:f32) -> f32 {self.end_time}
    fn note_type(&self) -> NoteType {NoteType::Spinner}
    fn set_alpha(&mut self, alpha: f32) {self.alpha_mult = alpha}

    fn update(&mut self, beatmap_time: f32) {

        let mut diff = 0.0;
        let pos_diff = self.mouse_pos - self.pos;
        let mouse_angle = pos_diff.y.atan2(pos_diff.x);

        if beatmap_time >= self.time && beatmap_time <= self.end_time {
            if self.holding {
                diff = mouse_angle - self.last_rotation_val;
            }
            if diff > PI {diff -= 2.0 * PI}
            else if diff < -PI {diff += 2.0 * PI}
            // println!("diff: {:.2}", diff / PI);
            
            // self.rotation_velocity = f64::lerp(-diff, self.rotation_velocity, 0.005 * (beatmap_time - self.last_update) as f64);
            self.rotation_velocity = f64::lerp(self.rotation_velocity, diff, 0.005 * (beatmap_time - self.last_update) as f64);
            self.rotation += self.rotation_velocity * (beatmap_time - self.last_update) as f64;
            // println!("vel: {}", self.rotation_velocity);

            // println!("rotation: {}, diff: {}", self.rotation, diff);
        }

        self.last_rotation_val = mouse_angle;
        self.last_update = beatmap_time;
    }
    fn draw(&mut self, _args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if !(self.last_update >= self.time && self.last_update <= self.end_time) {return}

        let border = Some(Border::new(Color::BLACK.alpha(self.alpha_mult), NOTE_BORDER_SIZE));

        // bg circle
        let mut bg = Circle::new(
            Color::YELLOW.alpha(self.alpha_mult),
            -10.0,
            self.pos,
            SPINNER_RADIUS
        );
        bg.border = border.clone();
        list.push(Box::new(bg));

        // draw another circle on top which increases in radius as the counter gets closer to the reqired
        let mut fg = Circle::new(
            Color::WHITE.alpha(self.alpha_mult),
            -11.0,
            self.pos,
            SPINNER_RADIUS * (self.rotations_completed as f64 / self.rotations_required as f64)
        );
        fg.border = border.clone();
        list.push(Box::new(fg));

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
            get_font("main")
        );
        txt.center_text(Rectangle::bounds_only(
            Vector2::new(0.0, self.pos.y + 50.0),
            Vector2::new(self.pos.x * 2.0, 50.0)
        ));
        list.push(Box::new(txt));
    }

    fn reset(&mut self) {
        self.holding = false;
        self.rotation = 0.0;
        self.rotation_velocity = 0.0;
        self.rotations_completed = 0;
    }
}
impl StandardHitObject for StandardSpinner {
    fn miss(&mut self) {}
    fn was_hit(&self) -> bool {self.last_update >= self.end_time} 
    fn get_hitsamples(&self) -> HitSamples {self.def.hitsamples.clone()}
    fn get_hitsound(&self) -> u8 {self.def.hitsound}
    fn get_preempt(&self) -> f32 {0.0}
    fn point_draw_pos(&self, _: f32) -> Vector2 {Vector2::zero()} //TODO
    fn causes_miss(&self) -> bool {self.rotations_completed < self.rotations_required} // if the spinner wasnt completed in time, cause a miss
    fn set_hitwindow_miss(&mut self, _window: f32) {}

    fn get_points(&mut self, _is_press:bool, _:f32, _:(f32,f32,f32,f32)) -> ScoreHit {
        ScoreHit::Other(100, false)
    }

    fn press(&mut self, _time:f32) {
        self.holding = true;
    }
    fn release(&mut self, _time:f32) {
        self.holding = false;
    }
    fn mouse_move(&mut self, pos:Vector2) {
        self.mouse_pos = pos;
    }

    fn playfield_changed(&mut self, new_scale: &ScalingHelper) {
        self.pos = new_scale.window_size / 2.0
    } 

    fn pos_at(&self, time: f32, scaling_helper:&ScalingHelper) -> Vector2 {
        // println!("time: {}, {}, {}", time, self.time, self.end_time);

        if time < self.time || time >= self.end_time {
            
            return self.pos
        }

        let r = self.last_rotation_val + (time - self.last_update) as f64 / (4.0*PI);
        self.pos + Vector2::new(
            r.cos(),
            r.sin()
        ) * scaling_helper.scale * 20.0
    }
}


fn approach_circle(pos:Vector2, radius:f64, time_diff:f32, time_preempt:f32, depth:f64, scale:f64, alpha: f32, color: Color) -> Box<Circle> {
    let mut c = Circle::new(
        Color::TRANSPARENT_WHITE,
        depth - 100.0,
        pos,
        radius + (time_diff as f64 / time_preempt as f64) * (HITWINDOW_CIRCLE_RADIUS * scale)
    );
    c.border = Some(Border::new(color.alpha(alpha), NOTE_BORDER_SIZE * scale));
    Box::new(c)
}


#[derive(Clone)]
pub struct SliderPath {
    path: Vec<[f64; 2]>,
    color: Color,
    depth: f64
}
impl SliderPath {
    fn new(path: Vec<Vector2>, color: Color, depth: f64) -> Self {
        let path = path.iter().map(|a|(*a).into()).collect();
        Self {path, color, depth}
    }
}
impl Renderable for SliderPath {
    fn get_depth(&self) -> f64 {self.depth}

    fn get_lifetime(&self) -> u64 {0}

    fn set_lifetime(&mut self, _lifetime:u64) {}

    fn get_spawn_time(&self) -> u64 {0}
    fn set_spawn_time(&mut self, _time:u64) {}

    fn draw(&mut self, g: &mut opengl_graphics::GlGraphics, c:graphics::Context) {
        graphics::polygon(self.color.into(), &self.path, c.transform, g);

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
