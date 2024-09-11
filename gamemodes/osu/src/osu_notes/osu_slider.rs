use crate::prelude::*;

const SLIDER_DOT_RADIUS:f32 = 8.0;
const BORDER_RADIUS:f32 = 6.0;
const BORDER_COLOR:Color = Color::WHITE;
const BEAT_SCALE: f32 = 1.4;

const OK_RADIUS_MULT: f32 = 4.0;
const OK_TICK_RADIUS_MULT: f32 = 2.0;

const USE_NEW_SLIDER_RENDERING:bool = false;

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
    pending_combo: Vec<(HitJudgment, Vector2)>,

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
    start_judgment: HitJudgment,

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
    sliderdot_hitsound: Hitsound,

    last_beat: f32,
    pulse_length: f32,
    beat_scale: f32,

    slider_body: SliderDrawable,
    skin: Arc<SkinSettings>,
}
impl OsuSlider {
    pub async fn new(def:SliderDef, curve:Curve, ar:f32, combo_num: u16, scaling_helper:Arc<ScalingHelper>, standard_settings:Arc<OsuSettings>, hitsound_fn: impl Fn(f32, u8, HitSamples)->Vec<Hitsound>, velocity: f32) -> Self {
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

        let approach_circle = ApproachCircle::new(def.pos, time, radius, time_preempt, scaling_helper.clone());
        let start_circle_image = HitCircleImageHelper::new(
            def.pos,
            scaling_helper.clone(),
            combo_num
        ).await;

        Self {
            def,
            curve,
            color: Color::WHITE,
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


            last_beat: -100.0,
            pulse_length: 0.0,
            beat_scale: 1.0,

            slider_body: SliderDrawable::default(),
            skin: Default::default(),
        }
    }

    fn use_render_targets(&self) -> bool {
        self.standard_settings.slider_render_targets || !USE_NEW_SLIDER_RENDERING
    }

    async fn make_body(&mut self) {
        // TODO: check if we should try again
        if self.slider_body_render_target_failed.is_some() {
            return
        }

        // let mut list:Vec<Box<dyn TatakuRenderable>> = Vec::new();
        let window_size = WindowSize::get().0;

        // info!("{:?}", skin.slider_track_override);
        let mut color = self.skin.slider_track_override.filter(|c|c != &Color::BLACK && self.standard_settings.use_skin_slider_body_color).unwrap_or_else(|| {
            let mut color = self.color;
            const DARKER:f32 = 2.0/3.0;
            color.r *= DARKER;
            color.g *= DARKER;
            color.b *= DARKER;
            color
        });

        color.a = self.standard_settings.slider_body_alpha;

        let border_color = BORDER_COLOR.alpha(self.standard_settings.slider_border_alpha); //self.skin.slider_border.unwrap_or(BORDER_COLOR);
        let border_radius = BORDER_RADIUS * self.scaling_helper.scaled_cs;

        let mut min_pos = window_size;
        let mut max_pos = Vector2::ZERO;
        let size;

        let mut drawables: Vec<Box<dyn TatakuRenderable>> = Vec::new();
        let mut offset = Vector2::ZERO;

        if USE_NEW_SLIDER_RENDERING {
            let mut line_segments: Vec<LineSegment> = self.curve.segments.iter().map(|segment| {
                let points = segment.all_points();

                if points.len() == 0 { return Vec::new(); }

                // Calculate bounds of first point
                let first = self.scaling_helper.scale_coords(*points.first().unwrap());
                min_pos.x = min_pos.x.min(first.x);
                min_pos.y = min_pos.y.min(first.y);
                max_pos.x = max_pos.x.max(first.x);
                max_pos.y = max_pos.y.max(first.y);

                points.windows(2).map(|points| {
                    let p1 = self.scaling_helper.scale_coords(points[0]);
                    let p2 = self.scaling_helper.scale_coords(points[1]);

                    // Calculate bounds of remaining points
                    min_pos.x = min_pos.x.min(p2.x);
                    min_pos.y = min_pos.y.min(p2.y);
                    max_pos.x = max_pos.x.max(p2.x);
                    max_pos.y = max_pos.y.max(p2.y);

                    LineSegment { p1: p1.into(), p2: p2.into() }
                }).collect::<Vec<_>>() // todo: avoid too many allocations here
            }).flatten().collect();

            min_pos -= self.radius;
            max_pos += self.radius;

            size = max_pos - min_pos;

            let border_width = border_radius;
            let circle_radius = self.radius - border_width;
            let cell_size = self.radius;

            let grid_size = size / cell_size;
            let grid_size = [1 + grid_size.x.floor() as u32, 1 + grid_size.y.floor() as u32];

            let mut grid_cells: Vec<Vec<u32>> = vec![Vec::new(); (grid_size[0] * grid_size[1]) as usize];

            line_segments.iter_mut().enumerate()
                .for_each(|(i, s)| {
                    let p1 = Vector2::from(s.p1) - min_pos;
                    let p2 = Vector2::from(s.p2) - min_pos;

                    s.p1 = p1.into();
                    s.p2 = p2.into();

                    let grid_index = |p: Vector2| {
                        let grid_coord = p / cell_size;
                        let (grid_x, grid_y) = (grid_coord.x.floor() as usize, grid_coord.y.floor() as usize);

                        // info!("adding ({grid_x}, {grid_y})");

                        grid_y * grid_size[0] as usize + grid_x
                    };

                    let p1_grid_index = grid_index(p1);
                    let p2_grid_index = grid_index(p2);

                    // todo: fix intermediate cells

                    grid_cells[p1_grid_index].push(i as u32);
                    if p1_grid_index != p2_grid_index {
                        grid_cells[p2_grid_index].push(i as u32);
                    }
                });

            let grid_cells_len = grid_cells.len();

            let (slider_grids, grid_cells) = grid_cells.into_iter()
                .fold((Vec::with_capacity(grid_cells_len), Vec::with_capacity(grid_cells_len)), |(mut indexes, mut cells), v| {
                    indexes.push(GridCell {
                        index: cells.len() as u32,
                        length: v.len() as u32,
                    });
                    cells.extend(v);

                    (indexes, cells)
                });


            let slider_data = SliderData {
                circle_radius,
                border_width,
                snake_percentage: 1.0,
                slider_velocity: self.velocity,
                grid_origin: min_pos.into(),
                grid_size,
                grid_index: 0,
                body_color: color.into(),
                border_color: border_color.into(),
            };

            self.slider_body.size = size;
            self.slider_body.slider_data = slider_data;
            self.slider_body.slider_grids = slider_grids;
            self.slider_body.grid_cells = grid_cells;
            self.slider_body.line_segments = line_segments;


            let mut slider_body = self.slider_body.clone();
            slider_body.slider_data.grid_origin = Vector2::ZERO; // reset grid origin when rendering to a target
            slider_body.alpha = 1.0;
            drawables.push(Box::new(slider_body));
        } else {
            // starting point
            let p: Vector2 = self.scaling_helper.scale_coords(self.curve.curve_lines[0].p1);

            let radius_with_border = self.radius - border_radius * 0.5;
            // circles with a border have extra radius because of the border (i think its 0.5x the border width)
    
            // both body and border use the same code with a few differences, so might as well for-loop them to simplify code
            // border is first, body is 2nd, since the body must be drawn on top of the border (which creates the border)
            for (radius, color, blend_mode) in [
                (self.radius - border_radius * 0.5, border_color, BlendMode::AlphaBlending), // border
                (self.radius - border_radius * 1.5, color, BlendMode::AlphaOverwrite) // fill
            ] {
                // add starting circle manually
                drawables.push(Box::new(Circle::new(
                    p,
                    radius,
                    color,
                    None
                ).with_blend_mode(blend_mode)));
    
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
    
                    // add a line to connect the points
                    drawables.push(Box::new(Line::new(
                        p1,
                        p2,
                        radius,
                        color
                    ).with_blend_mode(blend_mode)));
    
                    // add a circle to smooth out the corners
                    // border
                    drawables.push(Box::new(Circle::new(
                        p2,
                        radius,
                        color,
                        None
                    ).with_blend_mode(blend_mode)));
                }
            }

            size = max_pos - min_pos;
            offset = -min_pos;
        }


        // draw it to the render target
        #[cfg(feature="graphics")]
        if self.use_render_targets() {
            let options = DrawOptions::default();
            if let Some(target) = self.slider_body_render_target.clone() {
                GameWindow::update_render_target(target, Box::new(move |g: &mut dyn GraphicsEngine, mut transform: Matrix| {
                    transform = transform.trans(offset); 
                    drawables.into_iter().for_each(|d| d.draw(&options, transform, g))
                })).await;
            } else {
                let rt = RenderTarget::new(
                    size.x as u32,
                    size.y as u32, 
                    Box::new(move |g: &mut dyn GraphicsEngine, mut transform: Matrix| {
                        transform = transform.trans(offset); 
                        drawables.into_iter().for_each(|d| d.draw(&options, transform, g))
                    })
                ).await;

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
        self.start_circle_image.update(beatmap_time);

        self.beat_scale = f32::lerp(BEAT_SCALE, 1.0, (beatmap_time - self.last_beat) / self.pulse_length).clamp(1.0, BEAT_SCALE);

        // update shapes
        self.shapes.retain_mut(|shape| {
            shape.update(beatmap_time);
            shape.visible()
        });

        // check sliding ok
        self.slider_ball_pos = self.scaling_helper.scale_coords(self.curve.position_at_time(beatmap_time));
        let distance = self.slider_ball_pos.distance(self.mouse_pos); //((self.slider_ball_pos.x - self.mouse_pos.x).powi(2) + (self.slider_ball_pos.y - self.mouse_pos.y).powi(2)).sqrt();
        self.sliding_ok = self.holding && distance <= self.radius * OK_RADIUS_MULT;


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

        if alpha > 0.0 && self.slider_body_render_target.is_none() && (self.use_render_targets() || self.slider_body.slider_data.circle_radius == 0.0) {
            self.make_body().await;
        }

        if let Some(ball) = &mut self.sliderball_image {
            ball.update(beatmap_time)
        }

    }

    async fn draw(&mut self, _time: f32, list: &mut RenderableCollection) {
        // draw shapes
        for shape in self.shapes.iter_mut() {
            list.push(shape.clone())
        }

        // if its not time to draw anything else, leave
        if self.time - self.map_time > self.time_preempt || self.map_time > self.curve.end_time + self.hitwindow_miss { return }

        // color
        let alpha = self.get_alpha();
        let color = self.color.alpha(alpha);
        self.slider_body.alpha = alpha;

        // slider body
        if self.use_render_targets() {
            if let Some(rt) = &self.slider_body_render_target {
                let mut b = rt.image.clone();
                b.color.a = alpha;
                list.push(b);
            }
        } else {
            list.push(self.slider_body.clone());
        }

        // draw hit dots
        for dot in self.hit_dots.iter() {
            if dot.slide_layer == self.slides_complete {
                dot.draw(self.beat_scale, list)
            }
        }

        // start and end circles
        let slides_remaining = self.def.slides - self.slides_complete;
        let end_repeat = slides_remaining > self.def.slides % 2 + 1;
        let start_repeat = slides_remaining > 2 - self.def.slides % 2;


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
                    if end_repeat { Color::YELLOW } else { Color::WHITE }.alpha(alpha),
                    self.scaling_helper.border_scaled
                ))
            ));
        }

        if end_repeat {
            if let Some(mut reverse_arrow) = self.slider_reverse_image.clone() {
                reverse_arrow.pos = self.visual_end_pos;
                reverse_arrow.color.a = alpha;
                reverse_arrow.scale = Vector2::ONE * self.beat_scale * self.scaling_helper.scaled_cs;

                let l = self.curve.curve_lines.last().unwrap();
                reverse_arrow.rotation = (l.p1 - l.p2).atan2_wrong();

                list.push(reverse_arrow);
            }
        }


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
                        if start_repeat { Color::YELLOW } else { Color::WHITE }.alpha(alpha),
                        self.scaling_helper.border_scaled
                    ))
                ));
            }

            if start_repeat {
                if let Some(mut reverse_arrow) = self.slider_reverse_image.clone() {
                    reverse_arrow.pos = self.pos;
                    reverse_arrow.color.a = alpha;
                    reverse_arrow.scale = Vector2::ONE * self.beat_scale * self.scaling_helper.scaled_cs;

                    let l = self.curve.curve_lines.first().unwrap();
                    reverse_arrow.rotation = (l.p2 - l.p1).atan2_wrong();

                    list.push(reverse_arrow);
                }
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
                    self.radius * OK_TICK_RADIUS_MULT,
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

    #[cfg(feature="graphics")]
    async fn reload_skin(&mut self, source: &TextureSource, skin_manager: &mut dyn SkinProvider) {
        self.skin = skin_manager.skin().clone();
        self.start_circle_image.reload_skin(source, skin_manager).await;
        self.end_circle_image = skin_manager.get_texture("sliderendcircle", source, SkinUsage::Gamemode, false).await;
        self.slider_reverse_image = skin_manager.get_texture("reversearrow", source, SkinUsage::Gamemode, false).await;
        self.follow_circle_image = skin_manager.get_texture("sliderfollowcircle", source, SkinUsage::Gamemode, false).await;

        self.approach_circle.reload_texture(source, skin_manager).await;

        for dot in self.hit_dots.iter_mut() {
            dot.reload_skin(source, skin_manager).await;
        }

        // slider ball
        self.sliderball_under_image = skin_manager.get_texture("sliderb-nd", source, SkinUsage::Gamemode, false).await;

        let mut i = 0;
        let mut images = Vec::new();
        loop {
            let Some(image) = skin_manager.get_texture(&format!("sliderb{i}"), source, SkinUsage::Gamemode, false).await else { break };
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


    fn beat_happened(&mut self, pulse_length: f32) {
        self.last_beat = self.map_time;
        self.pulse_length = pulse_length;
    }
    fn kiai_changed(&mut self, _is_kiai: bool) {}
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


    fn new_combo(&self) -> bool { self.def.new_combo }
    fn set_combo_color(&mut self, color: Color) { 
        self.color = color;
        
        self.start_circle_image.set_color(color);
        if self.standard_settings.approach_combo_color { 
            self.approach_circle.set_color(color);
        }
     }

    fn set_judgment(&mut self, j: &HitJudgment) {
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

    fn check_release_points(&mut self, time: f32) -> HitJudgment {
        self.end_checked = true;
        self.sound_index = self.def.edge_sounds.len() - 1;
        let distance = self.mouse_pos.distance(self.time_end_pos); //((self.time_end_pos.x - self.mouse_pos.x).powi(2) + (self.time_end_pos.y - self.mouse_pos.y).powi(2)).sqrt();
        let ok_distance = self.radius * OK_RADIUS_MULT;

        if self.start_judgment == OsuHitJudgments::Miss {
            if self.dot_count == 0 {
                if distance > ok_distance || !self.holding {
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
        } else if self.dots_missed == 0 && self.holding && distance < ok_distance {
            self.add_end_ripple(time);
            OsuHitJudgments::X300
        } else {
            self.add_end_ripple(time);
            OsuHitJudgments::X100
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


    fn pending_combo(&mut self) -> Vec<(HitJudgment, Vector2)> {
        self.pending_combo.take()
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

        if self.slider_body_render_target.is_some() || (!self.standard_settings.slider_render_targets && USE_NEW_SLIDER_RENDERING) {
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

        if (self.slider_body_render_target.is_some() || (!self.standard_settings.slider_render_targets && USE_NEW_SLIDER_RENDERING)) && (self.standard_settings.slider_body_alpha != old_body_alpha || old_border_alpha != self.standard_settings.slider_border_alpha) {
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
    
    fn shake(&mut self, time: f32) { self.start_circle_image.shake(time) }
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
    dot_image: Option<Image>,

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
            dot_image: None, 
        }
    }
    /// returns true if the hitsound should play
    pub fn update(&mut self, beatmap_time:f32, mouse_down: bool, mouse_pos: Vector2, slider_radius:f32) -> Option<bool> {
        if beatmap_time >= self.time && !self.checked {
            self.checked = true;
            self.hit = mouse_down && mouse_pos.distance(self.pos) < slider_radius * OK_TICK_RADIUS_MULT;

            Some(self.hit)
        } else {
            None
        }
    }

    pub fn draw(&self, beat_scale: f32, list: &mut RenderableCollection) {
        if self.checked { return }

        if let Some(mut image) = self.dot_image.clone() {
            image.pos = self.pos;
            image.scale = Vector2::ONE * beat_scale * self.scale * 0.8;
            list.push(image);
        } else {
            list.push(Circle::new(
                self.pos,
                SLIDER_DOT_RADIUS * self.scale * beat_scale,
                Color::YELLOW,
                Some(Border::new(Color::WHITE, OSU_NOTE_BORDER_SIZE * self.scale))
            ));
        }
    }

    #[cfg(feature="graphics")]
    pub async fn reload_skin(&mut self, source: &TextureSource, skin_manager: &mut dyn SkinProvider) {
        self.dot_image = skin_manager.get_texture("sliderscorepoint", source, SkinUsage::Gamemode, false).await;
    }
}

