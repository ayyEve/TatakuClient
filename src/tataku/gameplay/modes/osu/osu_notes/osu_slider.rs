use crate::prelude::*;
use super::super::prelude::*;


const SLIDER_DOT_RADIUS:f64 = 8.0;

pub struct OsuSlider {
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
    hitwindow_miss: f32,

    /// combo text cache, probably not needed but whatever
    combo_text: Option<Box<Text>>,
    combo_image: Option<SkinnedNumber>,

    /// list of sounds waiting to be played (used by repeat and slider dot sounds)
    /// (time, hitsound)
    sound_queue: Vec<Vec<Hitsound>>,

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
    approach_circle: ApproachCircle,
    slider_body_render_target: Option<RenderTarget>,
    slider_body_render_target_failed: Option<f32>,
    
    hitsounds: Vec<Vec<Hitsound>>,
    sliderdot_hitsound: Hitsound
}
impl OsuSlider {
    pub async fn new(def:SliderDef, curve:Curve, ar:f32, color:Color, combo_num: u16, scaling_helper:Arc<ScalingHelper>, slider_depth:f64, circle_depth:f64, standard_settings:Arc<StandardSettings>, hitsound_fn: impl Fn(f32, u8, HitSamples)->Vec<Hitsound>) -> Self {
        let time = def.time;
        let time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);
        
        let pos = scaling_helper.scale_coords(def.pos);
        let visual_end_pos = scaling_helper.scale_coords(curve.curve_lines.last().unwrap().p2);
        let time_end_pos = if def.slides % 2 == 1 {visual_end_pos} else {pos};
        let radius = CIRCLE_RADIUS_BASE * scaling_helper.scaled_cs;

        const SAMPLE_SETS:[&str; 4] = ["normal", "normal", "soft", "drum"];
        let sliderdot_hitsound = Hitsound::new_simple(format!("{}-slidertick", SAMPLE_SETS[def.hitsamples.addition_set as usize]));
        // sliderdot_hitsound.volume = def.hitsamples.volume as f32 / 100.0;

        let hitsounds = def.edge_sets.iter().enumerate().map(|(n, &[normal_set, addition_set])| {
            let mut samples = def.hitsamples.clone();
            samples.normal_set = normal_set;
            samples.addition_set = addition_set;

            let hitsound = def.edge_sounds[n.min(def.edge_sounds.len() - 1)];
            
            hitsound_fn(def.time, hitsound, samples)
        }).collect();

        let approach_circle = ApproachCircle::new(def.pos, time, radius, time_preempt, circle_depth, if standard_settings.approach_combo_color { color } else { Color::WHITE }, scaling_helper.clone());

        Self {
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

            combo_text: None,
            combo_image: None,
            sound_queue: Vec::new(),

            scaling_helper,
            sliding_ok: false,
            slider_ball_pos: Vector2::zero(),


            standard_settings,
            shapes: Vec::new(),
            hitwindow_miss: 0.0,

            start_circle_image: None,
            end_circle_image: None,
            slider_reverse_image: None,
            slider_body_render_target: None,
            slider_body_render_target_failed: None,
            approach_circle,

            hitsounds,
            sliderdot_hitsound,
        }
    }

    async fn make_body(&mut self) {
        // TODO: check if we should try again
        if self.slider_body_render_target_failed.is_some() {
            return
        }

        let mut list:Vec<Box<dyn Renderable>> = Vec::new();
        let window_size = WindowSize::get().0;
        
        let mut color = self.color;
        const DARKER:f32 = 2.0/3.0;
        color.r *= DARKER;
        color.g *= DARKER;
        color.b *= DARKER;

        const BORDER_RADIUS: f64 = 6.0;
        const BORDER_COLOR:Color = Color::WHITE;

        // starting point
        let mut p = self.scaling_helper.scale_coords(self.curve.curve_lines[0].p1);
        p.y = window_size.y - p.y;

        // both body and border use the same code with a few differences, so might as well for-loop them to simplify code
        // border is first, body is 2nd, since the body must be drawn on top of the border (which created the border)
        for (radius, color) in [(self.radius, BORDER_COLOR), (self.radius - BORDER_RADIUS, color)] {

            // add starting circle manually
            list.push(Box::new(Circle::new(
                color,
                0.0,
                p,
                radius,
                None
            )));

            // add all lines
            for line in self.curve.curve_lines.iter() {
                let mut p1 = self.scaling_helper.scale_coords(line.p1);
                let mut p2 = self.scaling_helper.scale_coords(line.p2);

                p1.y = window_size.y - p1.y;
                p2.y = window_size.y - p2.y;

                // add a line to connect the points
                list.push(Box::new(Line::new(
                    p1,
                    p2,
                    radius,
                    0.0,
                    color
                )));

                // add a circle to smooth out the corners
                // border
                list.push(Box::new(Circle::new(
                    color,
                    0.0,
                    p2,
                    radius,
                    None
                )));
            }
            
        }
        
        // draw it to the render texture
        if let Ok(mut slider_body_render_target) = RenderTarget::new(window_size.x, window_size.y, |rt, g| {
            use graphics::Graphics;
            let c = g.draw_begin(rt.viewport());
            g.clear_color(Color::TRANSPARENT_WHITE.into());
            for i in list { i.draw(g, c); }
            g.draw_end();
        }).await {
            slider_body_render_target.image.origin = Vector2::zero();
            slider_body_render_target.image.depth = self.slider_depth;
            self.slider_body_render_target = Some(slider_body_render_target);
        } else {
            self.slider_body_render_target_failed = Some(self.map_time);
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
            let border_color = if is_tick { Color::WHITE } else { self.color };
            let depth = if is_tick && self.standard_settings.slider_tick_ripples_above { self.slider_depth - 0.000001 } else { self.slider_depth };

            group.items.push(DrawItem::Circle(Circle::new(
                Color::TRANSPARENT_WHITE,
                depth,
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

    fn ripple_start(&mut self) {
        if !self.standard_settings.ripple_hitcircles { return }

        let scale = 0.0..1.3;

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
        let mut circle_group = TransformGroup::new();
        if let Some(start_circle) = &self.start_circle_image {
            circle_group.items.push(DrawItem::Image(start_circle.circle.clone()));
            circle_group.items.push(DrawItem::Image(start_circle.overlay.clone()));
        } else {
            circle_group.items.push(DrawItem::Circle(Circle::new(
                self.color,
                self.circle_depth, // should be above curves but below slider ball
                self.pos,
                self.radius,
                Some(Border::new(
                    Color::BLACK,
                    self.scaling_helper.border_scaled
                ))
            )));
        }

        // make it ripple and add it to the list
        circle_group.ripple_scale_range(0.0, 500.0, self.map_time as f64, scale, None, Some(0.5));
        self.shapes.push(circle_group);
    }

}

#[async_trait]
impl HitObject for OsuSlider {
    fn note_type(&self) -> NoteType { NoteType::Slider }
    fn time(&self) -> f32 { self.time }
    fn end_time(&self,_:f32) -> f32 { self.curve.end_time }

    async fn update(&mut self, beatmap_time: f32) {
        self.map_time = beatmap_time;

        // update shapes
        let time = beatmap_time as f64;
        self.shapes.retain_mut(|shape| {
            shape.update(time);
            shape.items.iter().find(|di|di.visible()).is_some()
        });

        // check sliding ok
        self.slider_ball_pos = self.scaling_helper.scale_coords(self.curve.position_at_time(beatmap_time));
        let distance = self.slider_ball_pos.distance(self.mouse_pos); //((self.slider_ball_pos.x - self.mouse_pos.x).powi(2) + (self.slider_ball_pos.y - self.mouse_pos.y).powi(2)).sqrt();
        self.sliding_ok = self.holding && distance <= self.radius * 2.0;

        
        let alpha = self.get_alpha();
        self.approach_circle.update(beatmap_time, alpha);

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
            self.ripple_start();
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
                self.sound_queue.push(self.get_hitsound());
                self.add_ripple(beatmap_time, pos, false);
            } else {
                // we broke combo
                self.pending_combo.push((OsuHitJudgments::SliderEndMiss, pos));
            }
        }


        let mut dots = std::mem::take(&mut self.hit_dots);
        for dot in dots.iter_mut() {
            if let Some(was_hit) = dot.update(beatmap_time, self.holding, self.mouse_pos, self.radius) {
                if was_hit {
                    self.add_ripple(beatmap_time, dot.pos, true);

                    
                    self.pending_combo.push((OsuHitJudgments::SliderDot, dot.pos));
                    self.sound_queue.push(vec![self.sliderdot_hitsound.clone()]);
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

    async fn draw(&mut self, _args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        // draw shapes
        for shape in self.shapes.iter_mut() {
            shape.draw(list)
        }

        // if its not time to draw anything else, leave
        if self.time - self.map_time > self.time_preempt || self.map_time > self.curve.end_time + self.hitwindow_miss { return }
        
        // color
        let alpha = self.get_alpha();
        let color = self.color.alpha(alpha);

        if self.map_time < self.time {
            // timing circle
            self.approach_circle.draw(list);
            
            // list.push(approach_circle(self.pos, self.radius, self.time - self.map_time, self.time_preempt, self.circle_depth, self.scaling_helper.scaled_cs, alpha, approach_circle_color).await);

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

        // slider body
        if let Some(rt) = &self.slider_body_render_target {
            let mut b = rt.image.clone();
            b.current_color.a = alpha;
            list.push(Box::new(b));
        }

        
        // start and end circles
        let slides_remaining = self.def.slides - self.slides_complete;
        let end_repeat = slides_remaining > self.def.slides % 2 + 1;
        let start_repeat = slides_remaining > 2 - self.def.slides % 2;


        // start pos
        if self.map_time < self.time {

            // draw the starting circle as a hitcircle
            if let Some(start_circle) = &mut self.start_circle_image {
                start_circle.set_alpha(alpha);
                start_circle.draw(list);
            } else {
                list.push(Box::new(Circle::new(
                    self.color.alpha(alpha),
                    self.circle_depth, // should be above curves but below slider ball
                    self.pos,
                    self.radius,
                    Some(Border::new(
                        Color::BLACK.alpha(alpha),
                        self.scaling_helper.border_scaled
                    ))
                )));
            }

        } else {
            // draw it as a slider end
            if let Some(end_circle) = &self.end_circle_image {
                let mut end_circle = end_circle.clone();
                end_circle.current_color.a = alpha;
                end_circle.current_pos = self.pos;
                list.push(Box::new(end_circle));
                
                if start_repeat {
                    if let Some(reverse_arrow) = &self.slider_reverse_image {
                        let mut im = reverse_arrow.clone();
                        im.current_pos = self.pos;
                        im.depth = self.circle_depth;
                        im.current_color.a = alpha;
                        im.current_scale = Vector2::one() * self.scaling_helper.scaled_cs;

                        let l = self.curve.curve_lines[0];
                        im.current_rotation = Vector2::atan2_wrong(l.p2 - l.p1);

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
                        if start_repeat { Color::RED } else { Color::BLACK }.alpha(alpha),
                        self.scaling_helper.border_scaled
                    ))
                )));
            }

        }


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
                    im.current_rotation = Vector2::atan2_wrong(l.p1 - l.p2);

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
                    if end_repeat { Color::RED } else { Color::BLACK }.alpha(alpha),
                    self.scaling_helper.border_scaled
                ))
            )));
        }


        // draw hit dots
        for dot in self.hit_dots.iter() {
            if dot.slide_layer == self.slides_complete {
                dot.draw(list)
            }
        }

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

    async fn time_jump(&mut self, new_time: f32) {
        if new_time > self.time {
            self.start_checked = true;
            if new_time > self.end_time(0.0) {
                self.end_checked = true;
            }
        } else {
            self.start_checked = false;
            self.end_checked = false;
        }
    }

    async fn reload_skin(&mut self) {
        self.start_circle_image = HitCircleImageHelper::new(self.pos, &self.scaling_helper, self.circle_depth, self.color).await;
        self.end_circle_image = SkinManager::get_texture("sliderendcircle", true).await;
        self.slider_reverse_image = SkinManager::get_texture("reversearrow", true).await;

        self.approach_circle.reload_texture().await;
        
        
        let mut combo_text =  Box::new(Text::new(
            Color::BLACK,
            self.circle_depth - 0.0000001,
            self.pos,
            self.radius as u32,
            self.combo_num.to_string(),
            get_font()
        ));
        combo_text.center_text(&Rectangle::bounds_only(
            self.pos - Vector2::one() * self.radius / 2.0,
            Vector2::one() * self.radius,
        ));
        self.combo_text = Some(combo_text);

        let mut combo_image = SkinnedNumber::new(
            Color::WHITE, // TODO: setting: colored same as note or just white?
            self.circle_depth - 0.0000001, 
            Vector2::zero(), 
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
        };
        self.combo_image = combo_image;

        for dot in self.hit_dots.iter_mut() {
            dot.reload_skin().await;
        }

        
    }
}

#[async_trait]
impl OsuHitObject for OsuSlider {
    fn miss(&mut self) { self.end_checked = true }
    fn was_hit(&self) -> bool { self.end_checked }
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
        
        self.ripple_start();
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
        let distance = self.mouse_pos.distance(self.time_end_pos); //((self.time_end_pos.x - self.mouse_pos.x).powi(2) + (self.time_end_pos.y - self.mouse_pos.y).powi(2)).sqrt();

        match self.start_judgment {
            OsuHitJudgments::Miss => {
                if self.dot_count == 0 {
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
                if self.dots_missed == 0 && self.holding && distance < self.radius * 2.0 {
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

    
    fn pending_combo(&mut self) -> Vec<(OsuHitJudgments, Vector2)> {
        std::mem::take(&mut self.pending_combo)
    }


    async fn playfield_changed(&mut self, new_scale: Arc<ScalingHelper>) {
        self.scaling_helper = new_scale.clone();
        self.pos = self.scaling_helper.scale_coords(self.def.pos);
        self.radius = CIRCLE_RADIUS_BASE * self.scaling_helper.scaled_cs;
        self.visual_end_pos =  self.scaling_helper.scale_coords(self.curve.position_at_length(self.curve.length()));
        self.time_end_pos = if self.def.slides % 2 == 1 {self.visual_end_pos} else {self.pos};

        self.approach_circle.scale_changed(new_scale);
        
        let mut combo_text =  Box::new(Text::new(
            Color::BLACK,
            self.circle_depth - 0.0000001,
            self.pos,
            self.radius as u32,
            format!("{}", self.combo_num),
            get_font()
        ));
        combo_text.center_text(&Rectangle::bounds_only(
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

    fn set_settings(&mut self, settings: Arc<StandardSettings>) {
        self.standard_settings = settings;
    }

    

    fn set_ar(&mut self, ar: f32) {
        self.time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);
    }


    // fn get_hitsamples(&self) -> HitSamples {
    //     let mut samples = self.def.hitsamples.clone();
    //     let [normal_set, addition_set] = self.def.edge_sets[self.sound_index.min(self.def.edge_sets.len() - 1)];
    //     samples.normal_set = normal_set;
    //     samples.addition_set = addition_set;

    //     samples
    // }
    // fn get_hitsound(&self) -> u8 {
    //     // trace!("{}: getting hitsound at index {}/{}", self.time, self.sound_index, self.def.edge_sounds.len() - 1);
    //     self.def.edge_sounds[self.sound_index.min(self.def.edge_sounds.len() - 1)]
    // }
    fn get_hitsound(&self) -> Vec<Hitsound> {
        let index = self.sound_index.min(self.def.edge_sets.len() - 1);
        self.hitsounds[index].clone()
    }
    fn get_sound_queue(&mut self) -> Vec<Vec<Hitsound>> {
        std::mem::take(&mut self.sound_queue)
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
    pub fn update(&mut self, beatmap_time:f32, mouse_down: bool, mouse_pos: Vector2, slider_radius:f64) -> Option<bool> {
        if beatmap_time >= self.time && !self.checked {
            self.checked = true;
            self.hit = mouse_down && mouse_pos.distance(self.pos) < slider_radius * 2.0;
            Some(self.hit)
        } else {
            None
        }
    }
    
    pub fn draw(&self, list:&mut Vec<Box<dyn Renderable>>) {
        if self.checked{ return }

        if let Some(mut image) = self.dot_image.clone() {
            image.depth = self.depth;
            image.current_pos = self.pos;
            image.current_scale = Vector2::one() * self.scale * 0.8;
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

    pub async fn reload_skin(&mut self) {
        self.dot_image = SkinManager::get_texture("sliderscorepoint", true).await;
    }
}
