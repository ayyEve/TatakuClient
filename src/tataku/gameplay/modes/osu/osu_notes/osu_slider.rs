use crate::prelude::*;
use super::super::prelude::*;

const SLIDER_DOT_RADIUS:f32 = 8.0;
const BORDER_RADIUS:f32 = 6.0;
const BORDER_COLOR:Color = Color::WHITE;

pub struct OsuSlider {
    /// slider definition for this slider
    def: SliderDef,
    /// curve that defines the slider
    curve: Curve,
    velocity: f32,

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
    /// note size
    radius: f32,
    
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

    /// when should the note start being drawn (specifically the )
    time_preempt: f32,
    hitwindow_miss: f32,

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
    standard_settings: Arc<OsuSettings>,
    /// list of shapes to be drawn
    shapes: Vec<TransformGroup>,


    start_circle_image: HitCircleImageHelper,
    end_circle_image: Option<Image>,
    slider_reverse_image: Option<Image>,
    sliderball_image: Option<Animation>,
    sliderball_under_image: Option<Image>,
    follow_circle_image: Option<Image>,

    approach_circle: ApproachCircle,
    slider_body_render_target: Option<RenderTarget>,
    slider_body_render_target_failed: Option<f32>,
    
    hitsounds: Vec<Vec<Hitsound>>,
    sliderdot_hitsound: Hitsound
}
impl OsuSlider {
    pub async fn new(def:SliderDef, curve:Curve, ar:f32, color:Color, combo_num: u16, scaling_helper:Arc<ScalingHelper>, standard_settings:Arc<OsuSettings>, hitsound_fn: impl Fn(f32, u8, HitSamples)->Vec<Hitsound>, velocity: f32) -> Self {
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

        let approach_circle = ApproachCircle::new(def.pos, time, radius, time_preempt, if standard_settings.approach_combo_color { color } else { Color::WHITE }, scaling_helper.clone());
        let start_circle_image = HitCircleImageHelper::new(
            def.pos,
            scaling_helper.clone(),
            color,
            combo_num
        ).await;

        Self {
            def,
            curve,
            color,
            time_preempt,
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
            mouse_pos: Vector2::ZERO,
            
            dots_missed: 0,
            dot_count: 0,
            start_judgment: OsuHitJudgments::Miss,

            sound_queue: Vec::new(),

            scaling_helper,
            sliding_ok: false,
            slider_ball_pos: Vector2::ZERO,


            standard_settings,
            shapes: Vec::new(),
            hitwindow_miss: 0.0,

            start_circle_image,
            end_circle_image: None,
            slider_reverse_image: None,
            slider_body_render_target: None,
            slider_body_render_target_failed: None,
            follow_circle_image: None,
            sliderball_image: None,
            sliderball_under_image: None,
            approach_circle,

            hitsounds,
            sliderdot_hitsound,
            velocity,
        }
    }

    async fn make_body(&mut self) {
        // TODO: check if we should try again
        if self.slider_body_render_target_failed.is_some() {
            return
        }
        let skin = SkinManager::current_skin_config().await;

        let mut list:Vec<Box<dyn TatakuRenderable>> = Vec::new();
        let window_size = WindowSize::get().0;
        
        // info!("{:?}", skin.slider_track_override);
        let mut color = skin.slider_track_override.filter(|c|c != &Color::BLACK && self.standard_settings.use_skin_slider_body_color).unwrap_or_else(|| {
            let mut color = self.color;
            const DARKER:f32 = 2.0/3.0;
            color.r *= DARKER;
            color.g *= DARKER;
            color.b *= DARKER;
            color
        });

        color.a = self.standard_settings.slider_body_alpha;

        let border_color = BORDER_COLOR.alpha(self.standard_settings.slider_border_alpha); //skin.slider_border.unwrap_or(BORDER_COLOR);
        let border_radius = BORDER_RADIUS * self.scaling_helper.scaled_cs;

        // starting point
        let p: Vector2 = self.scaling_helper.scale_coords(self.curve.curve_lines[0].p1);
        // p.y = window_size.y - p.y;

        let mut min_pos = window_size;
        let mut max_pos = Vector2::ZERO;

        let radius_with_border = self.radius - border_radius * 0.5;
        // circles with a border have extra radius because of the border (i think its 0.5x the border width)

        // both body and border use the same code with a few differences, so might as well for-loop them to simplify code
        // border is first, body is 2nd, since the body must be drawn on top of the border (which creates the border)
        for (radius, color) in [(self.radius - border_radius * 0.5, border_color), (self.radius - border_radius * 1.5, color)] {

            // add starting circle manually
            list.push(Box::new(Circle::new(
                p,
                radius,
                color,
                None
            )));

            // add all lines
            for line in self.curve.curve_lines.iter() {
                let p1 = self.scaling_helper.scale_coords(line.p1);
                let p2 = self.scaling_helper.scale_coords(line.p2);
                
                if p1.x - radius_with_border < min_pos.x { min_pos.x = p1.x - radius_with_border; }
                if p1.y - radius_with_border < min_pos.y { min_pos.y = p1.y - radius_with_border; }
                if p2.x - radius_with_border < min_pos.x { min_pos.x = p2.x - radius_with_border; }
                if p2.y - radius_with_border < min_pos.y { min_pos.y = p2.y - radius_with_border; }

                if p1.x + radius_with_border > max_pos.x { max_pos.x = p1.x + radius_with_border; }
                if p1.y + radius_with_border > max_pos.y { max_pos.y = p1.y + radius_with_border; }
                if p2.x + radius_with_border > max_pos.x { max_pos.x = p2.x + radius_with_border; }
                if p2.y + radius_with_border > max_pos.y { max_pos.y = p2.y + radius_with_border; }


                // p1.y = window_size.y - p1.y;
                // p2.y = window_size.y - p2.y;

                // add a line to connect the points
                list.push(Box::new(Line::new(
                    p1,
                    p2,
                    radius,
                    color
                )));

                // add a circle to smooth out the corners
                // border
                list.push(Box::new(Circle::new(
                    p2,
                    radius,
                    color,
                    None
                )));
            }
        }

        // draw it to the render texture
        if let Some(target) = self.slider_body_render_target.clone() {
            GameWindow::update_render_target(target, RenderPipeline::AlphaOverwrite, move |state, matrix| {
                let m = matrix.trans(-min_pos);
                for i in list { i.draw(m, state); }
            }).await;
        } else {
            let size = max_pos - min_pos;

            let rt = RenderTarget::new(size.x as u32, size.y as u32, RenderPipeline::AlphaOverwrite, move |state, matrix| {
                let m = matrix.trans(-min_pos);
                for i in list { i.draw(m, state); } 
            }).await;

            if let Ok(mut slider_body_render_target) = rt {
                slider_body_render_target.image.pos = min_pos; 
                slider_body_render_target.image.origin = Vector2::ZERO;
                self.slider_body_render_target = Some(slider_body_render_target);
            } else {
                warn!("failed to slider");
                self.slider_body_render_target_failed = Some(self.map_time);
            }
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
                self.scaling_helper.scale,
                slide_counter
            ).await;

            self.dot_count += 1;
            self.hit_dots.push(dot);
        }
    }

    fn add_ripple(&mut self, time: f32, pos: Vector2, is_tick: bool) {
        if self.standard_settings.hit_ripples {
            let mut group = TransformGroup::new(pos).alpha(0.0).border_alpha(1.0);
            group.alpha.current = 0.0;

            // border is white if ripple caused by slider tick
            let border_color = if is_tick { Color::WHITE } else { self.color };

            group.push(Circle::new(
                Vector2::ZERO,
                self.radius,
                Color::TRANSPARENT_WHITE,
                Some(Border::new(border_color, 2.0))
            ));

            let duration = 500.0;
            group.ripple(0.0, duration, time, self.standard_settings.ripple_scale, true,  None);

            self.shapes.push(group);
        }
    }

    fn get_alpha(&self) -> f32 {
        let mut alpha = ((1.0 - ((self.time - (self.time_preempt * (2.0/3.0))) - self.map_time) / (self.time_preempt * (1.0/3.0))) / 3.0).clamp(0.0, 1.0);
        if self.map_time >= self.curve.end_time {
            alpha = ((self.curve.end_time + self.hitwindow_miss) - self.map_time) / self.hitwindow_miss;
        }
        alpha
    }

    fn ripple_start(&mut self) {
        if !self.standard_settings.ripple_hitcircles { return }
        self.shapes.push(self.start_circle_image.ripple(self.map_time));
    }

    fn add_end_ripple(&mut self, time: f32) {
        self.add_ripple(time, self.time_end_pos, false);
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
        self.shapes.retain_mut(|shape| {
            shape.update(beatmap_time);
            shape.visible()
        });

        // check sliding ok
        self.slider_ball_pos = self.scaling_helper.scale_coords(self.curve.position_at_time(beatmap_time));
        let distance = self.slider_ball_pos.distance(self.mouse_pos); //((self.slider_ball_pos.x - self.mouse_pos.x).powi(2) + (self.slider_ball_pos.y - self.mouse_pos.y).powi(2)).sqrt();
        self.sliding_ok = self.holding && distance <= self.radius * 2.0;

        
        let alpha = self.get_alpha();
        self.approach_circle.set_alpha(alpha);
        self.approach_circle.update(beatmap_time);

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

            // if self.slides_complete == self.def.slides {
            //     self.sound_index = self.def.edge_sounds.len() - 1;
            // }

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

        if let Some(ball) = &mut self.sliderball_image {
            ball.update(beatmap_time)
        }

    }

    async fn draw(&mut self, list: &mut RenderableCollection) {
        // draw shapes
        for shape in self.shapes.iter_mut() {
            list.push(shape.clone())
        }

        // if its not time to draw anything else, leave
        if self.time - self.map_time > self.time_preempt || self.map_time > self.curve.end_time + self.hitwindow_miss { return }
        
        // color
        let alpha = self.get_alpha();
        let color = self.color.alpha(alpha);

        // slider body
        if let Some(rt) = &self.slider_body_render_target {
            let mut b = rt.image.clone();
            b.color.a = alpha;
            list.push(b);
        }

        // draw hit dots
        for dot in self.hit_dots.iter() {
            if dot.slide_layer == self.slides_complete {
                dot.draw(list)
            }
        }

        // start and end circles
        let slides_remaining = self.def.slides - self.slides_complete;
        let end_repeat = slides_remaining > self.def.slides % 2 + 1;
        let start_repeat = slides_remaining > 2 - self.def.slides % 2;


        // start pos
        if self.map_time < self.time {
            // draw the starting circle as a hitcircle
            self.start_circle_image.set_alpha(alpha);
            self.start_circle_image.draw(list);
        } else {
            // draw it as a slider end
            if let Some(end_circle) = &self.end_circle_image {
                let mut end_circle = end_circle.clone();
                end_circle.color.a = alpha;
                end_circle.pos = self.pos;
                list.push(end_circle);
                
            } else if self.start_circle_image.circle.is_none() {
                list.push(Circle::new(
                    self.pos,
                    self.radius,
                    self.color.alpha(alpha),
                    Some(Border::new(
                        if start_repeat { Color::RED } else { Color::BLACK }.alpha(alpha),
                        self.scaling_helper.border_scaled
                    ))
                ));
            }

            if start_repeat {
                if let Some(reverse_arrow) = &self.slider_reverse_image {
                    let mut im = reverse_arrow.clone();
                    im.pos = self.pos;
                    im.color.a = alpha;
                    im.scale = Vector2::ONE * self.scaling_helper.scaled_cs;

                    let l = self.curve.curve_lines[0];
                    im.rotation = Vector2::atan2_wrong(l.p2 - l.p1);

                    list.push(im);
                }
            }
        }


        // end pos
        if let Some(end_circle) = &self.end_circle_image {
            let mut im = end_circle.clone();
            im.color.a = alpha;
            list.push(im);
        } else if self.start_circle_image.circle.is_none() {
            list.push(Circle::new(
                self.visual_end_pos,
                self.radius,
                color,
                Some(Border::new(
                    if end_repeat { Color::RED } else { Color::BLACK }.alpha(alpha),
                    self.scaling_helper.border_scaled
                ))
            ));
        }

        if end_repeat {
            if let Some(reverse_arrow) = &self.slider_reverse_image {
                let mut im = reverse_arrow.clone();
                im.pos = self.visual_end_pos;
                im.color.a = alpha;
                im.scale = Vector2::ONE * self.scaling_helper.scaled_cs;

                let l = self.curve.curve_lines[self.curve.curve_lines.len() - 1];
                im.rotation = Vector2::atan2_wrong(l.p1 - l.p2);

                list.push(im);
            }
        }


        // slider ball
        if self.map_time < self.curve.end_time && self.map_time >= self.time {
            let rotation = PI * 2.0 - (self.pos_at(self.map_time + 0.1) - self.slider_ball_pos).atan2();

            let scale = Vector2::ONE * self.scaling_helper.scaled_cs;

            // under
            if let Some(mut ball) = self.sliderball_under_image.clone() {
                ball.pos = self.slider_ball_pos;
                ball.scale = scale;
                // ball.color = color;
                ball.color.a = alpha;

                list.push(ball);
            }

            // inner
            if let Some(mut ball) = self.sliderball_image.clone() {
                ball.pos = self.slider_ball_pos;
                ball.scale = scale;
                ball.color = color;
                ball.rotation = rotation;

                list.push(ball);
            } else {
                list.push(Circle::new(
                    self.slider_ball_pos,
                    self.radius,
                    color,
                    Some(Border::new(Color::WHITE.alpha(alpha), 2.0))
                ));
            }

            // radius thingy
            if let Some(mut circle) = self.follow_circle_image.clone() {
                circle.pos = self.slider_ball_pos;
                circle.scale = scale;
                circle.color = color;
                circle.rotation = rotation;

                list.push(circle);
            } else {
                list.push(Circle::new(
                    self.slider_ball_pos,
                    self.radius * 2.0,
                    Color::TRANSPARENT_WHITE,
                    Some(Border::new(if self.sliding_ok {Color::LIME} else {Color::RED}.alpha(alpha), 2.0)
                )));
            }
        }


        // approach circle
        if self.map_time < self.time {
            self.approach_circle.draw(list);
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
        self.start_circle_image.reload_skin().await;
        self.end_circle_image = SkinManager::get_texture("sliderendcircle", true).await;
        self.slider_reverse_image = SkinManager::get_texture("reversearrow", true).await;
        self.follow_circle_image = SkinManager::get_texture("sliderfollowcircle", true).await;

        self.approach_circle.reload_texture().await;

        for dot in self.hit_dots.iter_mut() {
            dot.reload_skin().await;
        }

        // slider ball
        self.sliderball_under_image = SkinManager::get_texture("sliderb-nd", true).await;
        
        let mut i = 0;
        let mut images = Vec::new();
        loop {
            let Some(image) = SkinManager::get_texture(format!("sliderb{i}"), true).await else { break };
            images.push(image);
            i += 1;
        }

        if images.len() > 0 {
            let size = images[0].tex_size();
            let base_scale = images[0].base_scale;

            let images = images.into_iter().map(|i|i.tex).collect::<Vec<_>>();

            // stolen from peppy, i'll figure it out later lol
            let frametime = 1000.0 / 60.0;
            let velocity = self.velocity;
            let frametime = ((150.0 / velocity) * frametime).max(frametime);
            let frametimes = vec![frametime; images.len()];

            let mut animation = Animation::new(Vector2::ZERO, size, images, frametimes, base_scale);
            animation.scale = Vector2::ONE;

            self.sliderball_image = Some(animation);
        } else {
            self.sliderball_image = None;
        }

    }

}

#[async_trait]
impl OsuHitObject for OsuSlider {
    fn miss(&mut self) { self.end_checked = true }
    fn was_hit(&self) -> bool { self.end_checked }
    fn point_draw_pos(&self, time: f32) -> Vector2 { self.pos_at(time) }

    fn get_preempt(&self) -> f32 { self.time_preempt }
    fn press(&mut self, _:f32) { self.holding = true }
    fn release(&mut self, _:f32) { self.holding = false }
    fn mouse_move(&mut self, pos:Vector2) { self.mouse_pos = pos }
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
                    self.add_end_ripple(time);
                    OsuHitJudgments::X100
                } else {
                    self.add_end_ripple(time);
                    OsuHitJudgments::X50
                }
            }

            _ => {
                if self.dots_missed == 0 && self.holding && distance < self.radius * 2.0 {
                    self.add_end_ripple(time);
                    OsuHitJudgments::X300
                } else {
                    self.add_end_ripple(time);
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

        self.approach_circle.scale_changed(new_scale, self.radius);
        self.start_circle_image.playfield_changed(&self.scaling_helper);

        if let Some(image) = &mut self.end_circle_image {
           image.pos = self.scaling_helper.scale_coords(self.visual_end_pos);
           image.scale = Vector2::ONE * self.scaling_helper.scaled_cs;
        }

        if self.slider_body_render_target.is_some() {
            // if the playfield was resized, if we dont set this to none it will use the old size and then be wrong
            self.slider_body_render_target = None;
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

    async fn set_settings(&mut self, settings: Arc<OsuSettings>) {
        let old_body_alpha = self.standard_settings.slider_body_alpha;
        let old_border_alpha = self.standard_settings.slider_border_alpha;
        //TODO: cache these and only update if the difference is above some threshhold so we dont absolutely spam render targets\
        self.standard_settings = settings;

        if self.slider_body_render_target.is_some() && (self.standard_settings.slider_body_alpha != old_body_alpha || old_border_alpha != self.standard_settings.slider_border_alpha) {
            self.make_body().await;
        }
    }


    fn set_ar(&mut self, ar: f32) {
        self.time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);
    }

    fn set_approach_easing(&mut self, easing: Easing) {
        self.approach_circle.easing_type = easing;
    }
    
    fn get_hitsound(&self) -> Vec<Hitsound> {
        // println!("playing hitsound index {}/{}", self.sound_index+1, self.def.edge_sets.len());
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
    scale: f32,

    /// which slide "layer" is this on?
    slide_layer: u64,
    dot_image: Option<Image>
}
impl SliderDot {
    pub async fn new(time:f32, pos:Vector2, scale: f32, slide_layer: u64) -> SliderDot {
        SliderDot {
            time,
            pos,
            scale,
            slide_layer,

            hit: false,
            checked: false,
            dot_image: SkinManager::get_texture("sliderscorepoint", true).await
        }
    }
    /// returns true if the hitsound should play
    pub fn update(&mut self, beatmap_time:f32, mouse_down: bool, mouse_pos: Vector2, slider_radius:f32) -> Option<bool> {
        if beatmap_time >= self.time && !self.checked {
            self.checked = true;
            self.hit = mouse_down && mouse_pos.distance(self.pos) < slider_radius * 2.0;
            Some(self.hit)
        } else {
            None
        }
    }
    
    pub fn draw(&self, list: &mut RenderableCollection) {
        if self.checked{ return }

        if let Some(mut image) = self.dot_image.clone() {
            image.pos = self.pos;
            image.scale = Vector2::ONE * self.scale * 0.8;
            list.push(image);
        } else {
            list.push(Circle::new(
                self.pos,
                SLIDER_DOT_RADIUS * self.scale,
                Color::WHITE,
                Some(Border::new(Color::BLACK, OSU_NOTE_BORDER_SIZE * self.scale))
            ));
        }

    }

    pub async fn reload_skin(&mut self) {
        self.dot_image = SkinManager::get_texture("sliderscorepoint", true).await;
    }
}
