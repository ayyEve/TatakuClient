use std::ops::Range;

use crate::prelude::*;

const STACK_LENIENCY:u32 = 3;
pub const PREEMPT_MIN:f32 = 450.0;

pub struct OsuGame {
    // lists
    pub notes: Vec<Box<dyn OsuHitObject>>,

    actions: ActionQueue,

    // hit timing bar stuff
    hit_windows: Vec<(HitJudgment, Range<f32>)>,
    miss_window: f32,

    mouse_pos: Vector2,
    window_mouse_pos: Vector2,

    /// original, mouse_start
    move_playfield: Option<(Vector2, Vector2)>,

    /// how many keys are being held?
    hold_count: u16,

    /// scaling helper to help with scaling
    scaling_helper: Arc<ScalingHelper>,
    /// needed for scaling recalc
    cs: f32,
    stack_leniency: f32,

    /// cached settings
    game_settings: Arc<OsuSettings>,

    /// autoplay helper
    auto_helper: StandardAutoHelper,
    relax_manager: RelaxManager,

    /// list of note_indices which are new_combos
    new_combos: Vec<usize>,
    beatmap_combo_colors: Vec<Color>,

    use_controller_cursor: bool,
    window_size: Arc<WindowSize>,
    end_time: f32,

    follow_point_image: Option<Image>,
    judgment_helper: JudgmentImageHelper,

    metadata: Arc<BeatmapMeta>,
    mods: Arc<ModManager>,
    timing_points: Vec<TimingPoint>,

    smoke_emitter: Option<Emitter>,

    cursor: OsuCursor,
}
impl OsuGame {
    async fn recalculate_playfield(&mut self) {
        let new_scale = Arc::new(ScalingHelper::new_with_settings(
            &self.game_settings, 
            self.cs, 
            self.window_size.0, 
            self.mods.has_mod(HardRock)
        ));

        self.apply_playfield(new_scale).await
    }
    async fn apply_playfield(&mut self, playfield: Arc<ScalingHelper>) {
        self.scaling_helper = playfield.clone();
        self.cursor.note_radius = self.scaling_helper.scaled_circle_size.x / 2.0;

        // update playfield for notes
        for note in self.notes.iter_mut() {
            note.playfield_changed(playfield.clone()).await;
        }
    }

    // TODO: finish this
    #[allow(dead_code, unused_variables)]
    fn apply_stacking(&mut self) {
        let stack_offset = self.scaling_helper.scaled_cs / 10.0;

        let stack_vector = Vector2::ONE * stack_offset;

        // let stack_threshhold = self.preempt * self.beatmap.stack_leniency

        // // reset stack counters
        // for note in self.notes.iter_mut() {
        //     note.set_stack_count(0)
        // }

        
        // Extend the end index to include objects they are stacked on
        let mut extended_end_index = self.notes.len();

        let mut stack_base_index = self.notes.len();
        loop {
            for n in (stack_base_index + 1)..self.notes.len() {
                let obj = &self.notes[stack_base_index];
                if obj.note_type() == NoteType::Spinner { break }

                let obj_n = &self.notes[n];
                if obj_n.note_type() == NoteType::Spinner { break }

                let stack_threshhold = obj_n.get_preempt() * self.stack_leniency;

                if obj_n.time() - obj.time() > stack_threshhold {
                    // outside stack threshhold
                    break;
                }

                let obj_pos = obj.pos_at(obj.time());
                let obj_n_pos = obj.pos_at(obj.time());
                let obj_is_slider = obj.note_type() == NoteType::Slider;
                let obj_end_pos = obj.pos_at(obj.end_time(0.0));

                if obj_pos.distance(obj_n_pos) < STACK_LENIENCY as f32 || (obj_is_slider && obj_end_pos.distance(obj_n_pos) < STACK_LENIENCY as f32) {
                    stack_base_index = n;

                    // self.notes[n].set_stack_count(0)
                }
            }


            if stack_base_index > extended_end_index {
                extended_end_index = stack_base_index;
                if extended_end_index == self.notes.len() - 1 {break}
            }

            // check loop
            // if stack_base_index == 0 {
            //     stack_base_index -= 1
            // } else {
            //     break
            // }
        }


        
        // Reverse pass for stack calculation.
        let extended_start_index = self.notes.len() - 1;

    }

    fn setup_hitwindows(&mut self) {
        // windows
        let od = Self::get_od(&self.metadata, &self.mods);
        let w_miss = map_difficulty(od, 225.0, 175.0, 125.0); // idk
        let w_50   = map_difficulty(od, 200.0, 150.0, 100.0);
        let w_100  = map_difficulty(od, 140.0, 100.0, 60.0);
        let w_300  = map_difficulty(od, 80.0, 50.0, 20.0);
        self.miss_window = w_miss;

        self.hit_windows = vec![
            (OsuHitJudgments::X300, 0.0..w_300),
            (OsuHitJudgments::X100, w_300..w_100),
            (OsuHitJudgments::X50, w_100..w_50),
            (OsuHitJudgments::Miss, w_50..w_miss),
        ];
    }

    
    fn add_judgement_indicator(
        pos: Vector2, 
        hit_value: &HitJudgment, 
        scaling_helper: &Arc<ScalingHelper>, 
        judgment_helper: &JudgmentImageHelper, 
        settings: &OsuSettings, 
        state: &mut GameplayStateForUpdate<'_>
    ) {
        if hit_value.tex_name.is_empty() { return }

        let color = hit_value.color;
        let mut image = if settings.use_skin_judgments { judgment_helper.get_from_scorehit(hit_value) } else { None };
        if let Some(image) = &mut image {
            image.pos = pos;
            let scale = Vector2::ONE * scaling_helper.scaled_cs;
            image.scale = scale;
        }

        state.add_indicator(BasicJudgementIndicator::new(
            pos, 
            state.time,
            CIRCLE_RADIUS_BASE * scaling_helper.scaled_cs * (1.0/3.0),
            color,
            image
        ))
    }


    #[inline]
    fn scale_by_mods<V:std::ops::Mul<Output=V>>(val:V, ez_scale: V, hr_scale: V, mods: &ModManager) -> V {
        if mods.has_mod(Easy) {
            val * ez_scale
        } else if mods.has_mod(HardRock) {
            val * hr_scale
        } else {
            val
        }
    }

    #[inline]
    pub fn get_ar(meta: &BeatmapMeta, mods: &ModManager) -> f32 {
        Self::scale_by_mods(meta.ar, 0.5, 1.4, mods).clamp(1.0, 11.0)
    }

    #[inline]
    pub fn get_od(meta: &BeatmapMeta, mods: &ModManager) -> f32 {
        Self::scale_by_mods(meta.od, 0.5, 1.4, mods).clamp(1.0, 10.0)
    }

    #[inline]
    pub fn get_cs(meta: &BeatmapMeta, mods: &ModManager) -> f32 {
        Self::scale_by_mods(meta.cs, 0.5, 1.3, mods).clamp(1.0, 10.0)
    }

    fn draw_follow_points(&mut self, time: f32, list: &mut RenderableCollection) {
        if !self.game_settings.draw_follow_points { return; }
        if self.notes.is_empty() { return }

        let follow_dot_size = 3.0 * self.scaling_helper.scale;
        let follow_dot_distance = 20.0 * self.scaling_helper.scale;

        for i in 0..self.notes.len() - 1 {
            if self.new_combos.contains(&(i + 1)) { continue }

            let n1 = &self.notes[i];
            let n2 = &self.notes[i + 1];

            // skip if either note is a spinner
            if n1.note_type() == NoteType::Spinner { continue }
            if n2.note_type() == NoteType::Spinner { continue }

            // old code as backup
            // let preempt = n2.get_preempt();
            // let n1_time = n1.time();
            // if time < n1_time - preempt { continue } //|| time > n2.end_time(0.0) {continue}
            // let n2_time = n2.time();
            // if time >= n2_time { continue }//|| time <= n1_time {continue}

            let preempt = n2.get_preempt();
            let n1_time = n1.time();
            if time < n1_time - preempt { continue } //|| time > n2.end_time(0.0) {continue}
            let n2_time = n2.end_time(0.0);
            if time >= n2_time { continue }//|| time <= n1_time {continue}

            // setup follow points and the time they should exist at
            let n1_pos = n1.pos_at(n2_time);
            let n2_pos = n2.pos_at(n1_time);
            let distance = n1_pos.distance(n2_pos);
            if distance < follow_dot_distance { continue }

            let direction = PI * 2.0 - Vector2::atan2(n2_pos - n1_pos);
            let follow_dot_count = distance / follow_dot_distance;
            for i in 1..(follow_dot_count as u64 - 1) {
                let lerp_amount = i as f32 / follow_dot_count;
                let time_at_this_point = f32::lerp(n1_time, n2_time, lerp_amount);
                let point = Vector2::lerp(n1_pos, n2_pos, lerp_amount);
                
                // get the alpha
                let alpha_lerp_amount = (time_at_this_point - time) / (n2_time - n1_time);
                let alpha = if alpha_lerp_amount > 2.0 || alpha_lerp_amount < 0.0 {
                    0.0
                } else if alpha_lerp_amount > 1.0 {
                    f32::easeout_sine(1.0, 0.0, alpha_lerp_amount - 1.0)
                } else {
                    f32::easein_sine(0.0, 1.0, alpha_lerp_amount)
                };

                if alpha == 0.0 { continue }

                // add point
                if let Some(mut i) = self.follow_point_image.clone() {
                    i.pos = point;
                    i.rotation = direction;
                    // i.current_scale = Vector2::ONE * self.scaling_helper.scale;
                    list.push(i);
                } else {
                    list.push(Circle::new(
                        point,
                        follow_dot_size,
                        Color::WHITE.alpha(alpha),
                        None
                    ));
                }

            }
            
        }
        

    }

    fn apply_combo_colors(&mut self, colors: Vec<Color>) {
        let mut combo_num = 0;
        let mut combo_change = 0;

        for note in self.notes.iter_mut() {
            if note.new_combo() { combo_num = 0 }

            // if new combo, increment new combo counter
            if combo_num == 0 {
                combo_change += 1;
            }

            // get color
            let color = colors[(combo_change - 1) % colors.len()];
            note.set_combo_color(color);

            // update combo number
            combo_num += 1;
        }
    }

    fn map_key(&self, key: &Key) -> Option<KeyPress> {
        if key == &self.game_settings.left_key {
            Some(KeyPress::Left)
        } else if key == &self.game_settings.right_key {
            Some(KeyPress::Right)
        } else if key == &self.game_settings.smoke_key {
            Some(KeyPress::Dash)
        } else {
            None
        }
    }
    #[cfg(feature = "graphics")]
    fn map_btn(&self, btn: &MouseButton) -> Option<KeyPress> {
        if btn == &MouseButton::Left {
            Some(KeyPress::LeftMouse)
        } else if btn == &MouseButton::Right {
            Some(KeyPress::RightMouse)
        } else {
            None
        }
    }
}

#[async_trait]
impl GameMode for OsuGame {
    async fn new(
        map: &Beatmap, 
        diff_calc_only: bool,
        settings: &Settings,
    ) -> TatakuResult<Self> {
        let metadata = map.get_beatmap_meta();
        let mods = Arc::new(Default::default());
        let window_size = WindowSize::get();
        let effective_window_size = if diff_calc_only { super::diff_calc::WINDOW_SIZE } else { window_size.0 };
        
        let game_settings = settings.osu_settings.clone();

        let cs = Self::get_cs(&metadata, &mods);
        let ar = Self::get_ar(&metadata, &mods);
        let od = Self::get_od(&metadata, &mods);
        let scaling_helper = Arc::new(ScalingHelper::new_with_settings(&game_settings, cs, effective_window_size, mods.has_mod(HardRock)));

        let judgment_helper = JudgmentImageHelper::new(OsuHitJudgments::variants().to_vec()).await;

        let timing_points = TimingPointHelper::new(map.get_timing_points(), map.slider_velocity());

        let parent_dir = map.get_parent_dir().unwrap_or_default().to_string_lossy().to_string();
        let cursor = OsuCursor::new(scaling_helper.scaled_circle_size.x / 2.0, SkinSettings::default(), parent_dir, settings).await;
        let mut actions = ActionQueue::new();
        cursor.init(&mut actions);

        let mut s = match map {
            Beatmap::Osu(beatmap) => {
                let stack_leniency = beatmap.stack_leniency;
                let std_settings = Arc::new(game_settings);

                let get_hitsounds = |time, hitsound, hitsamples| {
                    let tp = timing_points.timing_point_at(time, true);
                    Hitsound::from_hitsamples(hitsound, hitsamples, true, tp)
                };

                let mut s = Self {
                    actions,
                    notes: Vec::new(),
                    mouse_pos: Vector2::ZERO,
                    window_mouse_pos: Vector2::ZERO,
                    hit_windows: Vec::new(),
                    miss_window: 0.0,
        
                    hold_count: 0,
                    end_time: 0.0,
        
                    move_playfield: None,
                    scaling_helper: scaling_helper.clone(),
                    cs,

                    use_controller_cursor: false,
        
                    game_settings: std_settings.clone(),
                    auto_helper: StandardAutoHelper::new(),
                    relax_manager: RelaxManager::new(),
                    
                    new_combos: Vec::new(),
                    stack_leniency,
                    window_size,
                    follow_point_image: None,
                    judgment_helper,
                    metadata,
                    mods,
                    timing_points: map.get_timing_points(),
                    smoke_emitter: None,
                    cursor,

                    beatmap_combo_colors: beatmap.combo_colors.clone()
                };


                enum Thing<'a> {
                    Note(&'a NoteDef),
                    Slider(&'a SliderDef, Option<Box<Curve>>),
                    Spinner(&'a SpinnerDef),
                }
                impl Thing<'_> {
                    fn time(&self) -> f32 {
                        match self {
                            Self::Note(n) => n.time,
                            Self::Slider(s, _) => s.time,
                            Self::Spinner(s) => s.time,
                        }
                    }
                    fn end_time(&self) -> f32 {
                        match self {
                            Self::Note(n) => n.time,
                            Self::Slider(s, None) => s.time, // invisible note
                            Self::Slider(_, Some(c)) => c.end_time,
                            Self::Spinner(s) => s.end_time,
                        }
                    }
                    fn new_combo(&self) -> bool {
                        match self {
                            Self::Note(n) => n.new_combo,
                            Self::Slider(s, _) => s.new_combo,
                            Self::Spinner(_) => true,
                        }
                    }
                    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                        self
                            .time()
                            .partial_cmp(&other.time())
                            .unwrap_or(std::cmp::Ordering::Equal)
                    }
                }

                let mut all_items = 
                    beatmap.notes.iter().map(Thing::Note)
                    .chain(beatmap.sliders.iter().map(|s| Thing::Slider(s, if s.curve_points.is_empty() || s.length == 0.0 { None } else { Some(Box::new(get_curve(s, map, &timing_points))) } )))
                    .chain(beatmap.spinners.iter().map(Thing::Spinner))
                    .collect::<Vec<_>>()
                ;

                // sort all the notes
                all_items.sort_by(|a, b| a.cmp(b));

                // add notes
                let mut combo_num = 0;
                for (counter, i) in all_items.into_iter().enumerate() {

                    // update end time
                    s.end_time = s.end_time.max(i.end_time());

                    // reset combo if hitobject says so
                    if i.new_combo() { combo_num = 0 }

                    // if new combo, add counter to combo
                    if combo_num == 0 {
                        s.new_combos.push(counter);
                    }

                    // update combo number
                    combo_num += 1;

                    // add the hitobject
                    match i {
                        Thing::Note(note) => {
                            s.notes.push(Box::new(OsuNote::new(
                                note.clone(),
                                ar,
                                combo_num as u16,
                                scaling_helper.clone(),
                                std_settings.clone(),
                                get_hitsounds(note.time, note.hitsound, note.hitsamples.clone())
                            ).await));
                        }

                        Thing::Slider(slider, None) => {
                            let note = NoteDef {
                                pos: slider.pos,
                                time: slider.time,
                                hitsound: slider.hitsound,
                                hitsamples: slider.hitsamples.clone(),
                                new_combo: slider.new_combo,
                                color_skip: slider.color_skip,
                            };
        
                            let hitsounds = get_hitsounds(note.time, note.hitsound, note.hitsamples.clone());
                            s.notes.push(Box::new(OsuNote::new(
                                note,
                                ar,
                                combo_num as u16,
                                scaling_helper.clone(),
                                std_settings.clone(),
                                hitsounds,
                            ).await));
                        }

                        Thing::Slider(slider, Some(curve)) => {
                            s.notes.push(Box::new(OsuSlider::new(
                                slider.clone(),
                                *curve,
                                ar,
                                combo_num as u16,
                                scaling_helper.clone(),
                                std_settings.clone(),
                                get_hitsounds,
                                timing_points.slider_velocity_at(slider.time)
                            ).await));
                        }


                        Thing::Spinner(spinner) => {
                            let duration = spinner.end_time - spinner.time;
                            let min_rps = map_difficulty(od, 2.0, 4.0, 6.0) * 0.6;

                            let mut spins_required = (duration / 1000.0 * min_rps) as u16;
                            // fudge until we can properly calculate
                            if spins_required < 10 {
                                if spins_required > 2 {
                                    spins_required = 2;
                                } else {
                                    spins_required = 0;
                                }
                            }
                            
                            s.notes.push(Box::new(OsuSpinner::new(
                                spinner.clone(),
                                scaling_helper.clone(),
                                spins_required
                            ).await))
                        }
                    }
                }



                
                // // join notes and sliders into a single array
                // // needed because of combo counts
                // let mut all_items = Vec::new();
                // for note in beatmap.notes.iter() {
                //     all_items.push((Some(note), None, None));
                //     s.end_time = s.end_time.max(note.time);
                // }
                // for slider in beatmap.sliders.iter() {
                //     all_items.push((None, Some(slider), None));
        
                //     // can this be improved somehow?
                //     if slider.curve_points.is_empty() || slider.length == 0.0 {
                //         s.end_time = s.end_time.max(slider.time);
                //     } else {
                //         let curve = get_curve(slider, map, &timing_points);
                //         s.end_time = s.end_time.max(curve.end_time);
                //     }
                // }
                // for spinner in beatmap.spinners.iter() {
                //     all_items.push((None, None, Some(spinner)));
                //     s.end_time = s.end_time.max(spinner.end_time);
                // }

                // // sort
                // all_items.sort_by(|a, b| {
                //     let a_time = match a {
                //         (Some(note), None, None) => note.time,
                //         (None, Some(slider), None) => slider.time,
                //         (None, None, Some(spinner)) => spinner.time,
                //         _ => 0.0
                //     };
                //     let b_time = match b {
                //         (Some(note), None, None) => note.time,
                //         (None, Some(slider), None) => slider.time,
                //         (None, None, Some(spinner)) => spinner.time,
                //         _ => 0.0
                //     };
        
                //     a_time.partial_cmp(&b_time).unwrap()
                // });
        

                // // add notes
                // let mut combo_num = 0;
        
                // for (counter, (note, slider, spinner)) in all_items.into_iter().enumerate() {
                //     // check for new combo
                //     if let Some(note) = note { if note.new_combo { combo_num = 0 } }
                //     if let Some(slider) = slider { if slider.new_combo { combo_num = 0 } }
                //     if let Some(_spinner) = spinner { combo_num = 0 }
        
                //     // if new combo, increment new combo counter
                //     if combo_num == 0 {
                //         s.new_combos.push(counter);
                //     }
                //     // get color
                //     // update combo number
                //     combo_num += 1;
        
                //     if let Some(note) = note {
                //         s.notes.push(Box::new(OsuNote::new(
                //             note.clone(),
                //             ar,
                //             combo_num as u16,
                //             scaling_helper.clone(),
                //             std_settings.clone(),
                //             get_hitsounds(note.time, note.hitsound, note.hitsamples.clone())
                //         ).await));
                //     }
                //     if let Some(slider) = slider {
                //         // invisible note
                //         if slider.curve_points.is_empty() || slider.length == 0.0 {
                //             let note = NoteDef {
                //                 pos: slider.pos,
                //                 time: slider.time,
                //                 hitsound: slider.hitsound,
                //                 hitsamples: slider.hitsamples.clone(),
                //                 new_combo: slider.new_combo,
                //                 color_skip: slider.color_skip,
                //             };
        
                //             let hitsounds = get_hitsounds(note.time, note.hitsound, note.hitsamples.clone());
                //             s.notes.push(Box::new(OsuNote::new(
                //                 note,
                //                 ar,
                //                 combo_num as u16,
                //                 scaling_helper.clone(),
                //                 std_settings.clone(),
                //                 hitsounds,
                //             ).await));
                //         } else {
                //             let curve = get_curve(slider, map, &timing_points);
                //             s.notes.push(Box::new(OsuSlider::new(
                //                 slider.clone(),
                //                 curve,
                //                 ar,
                //                 combo_num as u16,
                //                 scaling_helper.clone(),
                //                 std_settings.clone(),
                //                 get_hitsounds,
                //                 timing_points.slider_velocity_at(slider.time)
                //             ).await))
                //         }
                        
                //     }
                //     if let Some(spinner) = spinner {
                //         let duration = spinner.end_time - spinner.time;
                //         let min_rps = map_difficulty(od, 2.0, 4.0, 6.0) * 0.6;

                //         let mut spins_required = (duration / 1000.0 * min_rps) as u16;
                //         // fudge until we can properly calculate
                //         if spins_required < 10 {
                //             if spins_required > 2 {
                //                 spins_required = 2;
                //             } else {
                //                 spins_required = 0;
                //             }
                //         }
                        
                //         s.notes.push(Box::new(OsuSpinner::new(
                //             spinner.clone(),
                //             scaling_helper.clone(),
                //             spins_required
                //         ).await))
                //     }
                // }
        
                s
            }
            
            _ => return Err(BeatmapError::UnsupportedMode.into()),
        };

        // wait an extra sec
        s.end_time += 1000.0;

        s.setup_hitwindows();

        Ok(s)
    }

    async fn handle_replay_frame<'a>(
        &mut self, 
        frame: ReplayFrame, 
        state: &mut GameplayStateForUpdate<'a>
    ) {
        const ALLOWED_PRESSES:&[KeyPress] = &[
            KeyPress::Left, 
            KeyPress::Right,
            KeyPress::Dash,
            KeyPress::LeftMouse,
            KeyPress::RightMouse,
        ];

        match frame.action {
            ReplayAction::Press(key) if ALLOWED_PRESSES.contains(&key) => {
                self.hold_count += 1;

                match key {
                    KeyPress::Left | KeyPress::LeftMouse => self.cursor.left_pressed(true),
                    KeyPress::Right | KeyPress::RightMouse => self.cursor.right_pressed(true),
                    KeyPress::Dash => {
                        for i in self.smoke_emitter.iter_mut() {
                            i.should_emit = true
                        }
                        return;
                    }
                    _ => {}
                }

                let mut hittable_notes = Vec::new();
                let mut visible_notes = Vec::new();

                for note in self.notes.iter_mut() {
                    note.press(frame.time);
                    // check if note is in hitwindow, has not yet been hit, and is not a spinner
                    let note_time = note.time();
                    let in_hitwindow = (frame.time - note_time).abs() <= self.miss_window;
                    let is_visible = frame.time > note_time - note.get_preempt() && frame.time < note_time;

                    if (in_hitwindow || is_visible) && !note.was_hit() && note.note_type() != NoteType::Spinner {
                        if in_hitwindow {
                            hittable_notes.push(note)
                        } else { 
                            visible_notes.push(note) 
                        }
                    }
                }

                if hittable_notes.is_empty() && visible_notes.is_empty() { return } // no notes to check
                hittable_notes.sort_by(|a, b| a.time().partial_cmp(&b.time()).unwrap());
                
                for note in hittable_notes {
                    if !note.check_distance(self.mouse_pos) { continue }
                    let note_time = note.time();
                    
                    if let Some(judge) = state.check_judgment(&self.hit_windows, frame.time, note_time).await {
                        note.set_judgment(judge);

                        if judge == &OsuHitJudgments::X300 && !self.game_settings.show_300s {
                            // dont show the judgment
                        } else {
                            Self::add_judgement_indicator(
                                note.point_draw_pos(frame.time), 
                                judge, 
                                &self.scaling_helper, 
                                &self.judgment_helper, 
                                &self.game_settings, 
                                state
                            );
                        }

                        if judge == &OsuHitJudgments::Miss {
                            // tell the note it was missed
                            note.miss();
                        } else {
                            // tell the note it was hit
                            note.hit(frame.time);

                            // play the sound
                            state.play_note_sound(note.get_hitsound());
                        }

                        return;
                    }
                }

                // no notes were hit, check visible notes
                visible_notes.sort_by(|a, b| a.time().partial_cmp(&b.time()).unwrap());
                for note in visible_notes {
                    if !note.check_distance(self.mouse_pos) { continue }

                    note.shake(frame.time);
                    break
                }
            }
            // dont continue if no keys were being held (happens when leaving a menu)
            ReplayAction::Release(key) if ALLOWED_PRESSES.contains(&key) && self.hold_count > 0 => {
                self.hold_count -= 1;

                match key {
                    KeyPress::Left | KeyPress::LeftMouse => self.cursor.left_pressed(false),
                    KeyPress::Right | KeyPress::RightMouse => self.cursor.right_pressed(false),
                    KeyPress::Dash => {
                        if let Some(i) = self.smoke_emitter.as_mut() { i.should_emit = false; }
                        return;
                    }
                    _ => {}
                }

                // let mut check_notes = Vec::new();
                for note in self.notes.iter_mut() {
                    // if this is the last key to be released
                    if self.hold_count == 0 {
                        note.release(frame.time)
                    }

                    // // check if note is in hitwindow
                    // if time >= note.end_time(self.miss_window) && !note.was_hit() && note.note_type() == NoteType::Slider {
                    //     check_notes.push(note);
                    // }
                }
            }
            ReplayAction::MousePos(x, y) => {
                // scale the coords from playfield to window
                let pos = self.scaling_helper.scale_coords(Vector2::new(x, y));
                self.mouse_pos = pos;
                self.smoke_emitter.as_mut().map(|i| i.position = pos);
                self.cursor.cursor_pos(pos);

                for note in self.notes.iter_mut() {
                    note.mouse_move(pos);
                }
            }
            _ => {}
        }
    }


    async fn update<'a>(
        &mut self, 
        state: &mut GameplayStateForUpdate<'a>,
    ) {
        state.action_queue.extend(self.actions.take());

        // let mut pending_frames = Vec::new();

        // disable the cursor particle emitter if this is a menu game
        // the emitter nukes perf so its best to keep it off unless needed
        if state.gameplay_mode.is_preview() && self.cursor.emitter_enabled {
            self.cursor.emitter_enabled = false;
        }
        self.cursor.update(state.time, state.settings).await;

        let has_autoplay = state.mods.has_autoplay();
        let has_relax = state.mods.has_mod(Relax);

        // do autoplay things
        if has_autoplay {
            let mut pending_frames = Vec::new();
            self.auto_helper.update(state.time, &self.notes, &self.scaling_helper, &mut pending_frames);

            // // handle presses and mouse movements now, and releases later
            for action in pending_frames {
                state.add_replay_action(action);
            }
            // for frame in pending_frames.iter() {
            //     self.handle_replay_frame(*frame, time, manager).await;
            // }
        }
        
        if has_relax {
            self.relax_manager.update(state.time);
        }

        // update emitter
        if let Some(e) = self.smoke_emitter.as_mut() { e.update(state.time) }

        // if the map is over, say it is
        if state.time >= self.end_time {
            if !state.complete() {
                state.add_action(GamemodeAction::MapComplete);
                // manager.completed = true;
            }
            return;
        }

        // update notes
        for (note_index, note) in self.notes.iter_mut().enumerate() {
            note.update(state.time).await;
            let end_time = note.end_time(self.miss_window);

            // play queued sounds
            for hitsound in note.get_sound_queue() {
                state.play_note_sound(hitsound);
            }

            for (judgment, pos) in note.pending_combo() {
                state.add_judgment(judgment);
                Self::add_judgement_indicator(
                    pos, 
                    &judgment, 
                    &self.scaling_helper, 
                    &self.judgment_helper, 
                    &self.game_settings, 
                    state
                );
            }

            // check relax stuff
            if has_relax && !has_autoplay {
                self.relax_manager.check_note(
                    self.mouse_pos,
                    end_time,
                    note,
                    note_index,
                    state,
                );

                // // if its time to hit the note, the not hasnt been hit yet, and we're within the note's radius
                // if state.time >= note.time() && state.time < end_time && !note.was_hit() && note.check_distance(self.mouse_pos) {
                //     let key = KeyPress::LeftMouse;

                //     match note.note_type() {
                //         NoteType::Note => {
                //             pending_frames.push(ReplayAction::Press(key));
                //             pending_frames.push(ReplayAction::Release(key));
                //         }
                //         NoteType::Slider | NoteType::Spinner | NoteType::Hold => {
                //             // make sure we're not already holding
                //             if let Some(false) = state.key_counter.keys.get(&key).map(|a| a.held) {
                //                 state.add_replay_action(ReplayAction::Press(key));
                //                 // pending_frames.push(ReplayAction::Press(key));
                //             }
                //         }
                //     }
                // }

                // if state.time >= end_time && !note.was_hit() {
                //     let key = KeyPress::LeftMouse;

                //     match note.note_type() {
                //         NoteType::Note => {}
                //         NoteType::Slider | NoteType::Spinner | NoteType::Hold => {
                //             // assume we're holding i guess?
                //             state.add_replay_action(ReplayAction::Release(key));
                //             // pending_frames.push(ReplayAction::Release(key));
                //         }
                //     }
                // }
            }

            // check if note was missed
            
            // if the time is leading in, we dont want to check if any notes have been missed
            if state.time < 0.0 { continue }

            // check if note is in hitwindow
            if state.time >= end_time && !note.was_hit() {

                // check if we missed the current note
                match note.note_type() {
                    NoteType::Note => {
                        let j = OsuHitJudgments::Miss;
                        state.add_judgment(j);

                        Self::add_judgement_indicator(
                            note.point_draw_pos(state.time), 
                            &j, 
                            &self.scaling_helper, 
                            &self.judgment_helper, 
                            &self.game_settings, 
                            state
                        );
                    }
                    NoteType::Slider => {
                        // check slider release points
                        // internally checks distance
                        let judge = note.check_release_points(state.time);
                        state.add_judgment(judge);
                        
                        if judge != OsuHitJudgments::X300 || self.game_settings.show_300s {
                            Self::add_judgement_indicator(
                                note.point_draw_pos(state.time), 
                                &judge, 
                                &self.scaling_helper, 
                                &self.judgment_helper, 
                                &self.game_settings, 
                                state
                            );
                        }

                        if judge == OsuHitJudgments::Miss {
                            // // tell the note it was missed
                            // unecessary bc its told it was missed later lol
                            // info!("missed slider");
                            // note.miss();
                        } else {
                            // tell the note it was hit
                            note.hit(state.time);

                            // play the sound
                            // let hitsamples = note.get_hitsamples().clone();
                            state.play_note_sound(note.get_hitsound());
                        }
                    }

                    NoteType::Spinner => {
                        let j = OsuHitJudgments::SpinnerMiss;
                        state.add_judgment(j);

                        Self::add_judgement_indicator(
                            note.point_draw_pos(state.time), 
                            &j,
                            &self.scaling_helper, 
                            &self.judgment_helper, 
                            &self.game_settings, 
                            state
                        );
                    }

                    _ => {},
                }
                
                // force the note to be misssed
                note.miss(); 
            }
        }

        // handle note releases
        // required because autoplay frames are checked after the frame is processed
        // so if the key is released on the same frame its checked, it will count as not held
        // which makes sense, but we dont want that
        if has_autoplay {
            for action in self.auto_helper.get_release_queue() {
                state.add_replay_action(action);
            }
            // pending_frames.extend();
            // for frame in self.auto_helper.get_release_queue() {
            //     self.handle_replay_frame(frame, time, manager).await;
            // }
        }

    }
    
    async fn draw<'a>(
        &mut self, 
        state: GameplayStateForDraw<'a>, 
        list: &mut RenderableCollection
    ) {
        // draw the playfield
        if !state.gameplay_mode.is_preview() {
            let alpha = self.game_settings.playfield_alpha;
            let mut playfield = self.scaling_helper.playfield_scaled_with_cs_border;
            playfield.color.a = alpha;

            if self.move_playfield.is_some() {
                let line_size = self.game_settings.playfield_movelines_thickness;
                // draw x and y center lines
                let px_line = Line::new(
                    playfield.pos + Vector2::new(0.0, playfield.size.y/2.0),
                    playfield.pos + Vector2::new(playfield.size.x, playfield.size.y/2.0),
                    line_size,
                    Color::WHITE
                );
                let py_line = Line::new(
                    playfield.pos + Vector2::new(playfield.size.x/2.0, 0.0),
                    playfield.pos + Vector2::new(playfield.size.x/2.0, playfield.size.y),
                    line_size, 
                    Color::WHITE
                );

                let wx_line = Line::new(
                    Vector2::new(0.0, self.window_size.y/2.0),
                    Vector2::new(self.window_size.x, self.window_size.y/2.0),
                    line_size,
                    Color::WHITE
                );
                let wy_line = Line::new(
                    Vector2::new(self.window_size.x/2.0, 0.0),
                    Vector2::new(self.window_size.x/2.0, self.window_size.y),
                    line_size, 
                    Color::WHITE
                );

                playfield.border = Some(Border::new(Color::WHITE, line_size));

                list.push(wx_line);
                list.push(wy_line);
                list.push(px_line);
                list.push(py_line);
            }

            if state.current_timing_point.kiai {
                playfield.border = Some(Border::new(Color::YELLOW.alpha(alpha), 2.0));
            }
            list.push(playfield);
        }

        let has_flashlight = self.mods.has_mod(Flashlight);
        // if flashlight is enabled, we want to scissor all items by the playfield
        // this prevents things like approach circles and ripples from showing up outside the flashlight radius
        if has_flashlight {
            list.push_scissor(self.scaling_helper.playfield_scaled_with_cs_border.into_scissor())
        }

        // draw cursor ripples
        self.cursor.draw_below(list).await;

        // draw follow points
        self.draw_follow_points(state.time, list);

        // draw notes
        let mut spinners = Vec::new();
        for note in self.notes.iter_mut().rev() {
            match note.note_type() {
                NoteType::Spinner => spinners.push(note),
                _ => note.draw(state.time, list).await,
            }
        }

        // draw flashlight
        if has_flashlight {
            list.pop_scissor();

            let radius = match state.score.combo {
                0..=99 => 125.0,
                100..=199 => 100.0,
                _ => 75.0
            } * self.scaling_helper.scale;
            let fade_radius = radius / 5.0;

            list.push(FlashlightDrawable::new(
                self.mouse_pos,
                radius - fade_radius,
                fade_radius,
                *self.scaling_helper.playfield_scaled_with_cs_border,
                Color::BLACK
            ));
        }

        // spinners should be drawn last since they should be on top of everything
        // (we dont want notes or sliders drawn on top of the spinners)
        for i in spinners {
            i.draw(state.time, list).await
        }

        // need to draw the smoke particles on top of everything
        if let Some(e) = self.smoke_emitter.as_ref() { 
            e.draw(list) 
        }

        // draw the cursor on top of smoke tho
        self.cursor.draw_above(list).await;
    }

    
    async fn reset(&mut self, _beatmap:&Beatmap) {
        // let ar = scale_by_mods(self.metadata.ar, 0.5, 1.4, &self.mods).clamp(1.0, 11.0);

        // reset notes
        let hwm = self.miss_window;
        for note in self.notes.iter_mut() {
            note.reset().await;
            note.set_hitwindow_miss(hwm);
            // note.set_ar(ar)
        }

        // reset the smoke particles
        if let Some(e) = self.smoke_emitter.as_mut() { e.reset(0.0) }

        self.cursor.reset()
    }

    fn skip_intro(&mut self, game_time: f32) -> Option<f32> {
        if self.notes.is_empty() { return None }

        let time = self.notes[0].time() - self.notes[0].get_preempt();
        if time < game_time || time < 0.0 { return None }
        
        Some(time)
    }

    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.window_size = window_size;
        self.recalculate_playfield().await;
    }


    async fn fit_to_area(&mut self, bounds: Bounds) {
        self.apply_playfield(Arc::new(ScalingHelper::new_offset_scale(
            self.cs, 
            bounds.size, 
            bounds.pos, 
            0.5, 
            self.mods.has_mod(HardRock)
        ))).await;
    }

    async fn time_jump<'a>(
        &mut self, 
        new_time: f32,
        state: &mut GameplayStateForUpdate<'a>
    ) {
        for n in self.notes.iter_mut() {
            n.time_jump(new_time).await;
        }

        let mut pending_frames = Vec::new();
        self.auto_helper.time_skip(
            new_time, 
            &self.notes, 
            &self.scaling_helper, 
            &mut pending_frames
        );

        for i in pending_frames {
            state.add_replay_action(i);
        }
    }
    
    async fn force_update_settings(&mut self, settings: &Settings) {
        let settings = settings.osu_settings.clone();
        let settings = Arc::new(settings);

        if self.game_settings == settings { return }

        self.game_settings = settings.clone();
        for n in self.notes.iter_mut() {
            n.set_settings(settings.clone()).await;
        }
    }

    #[cfg(feature="graphics")]
    async fn reload_skin(&mut self, beatmap_path: &str, skin_manager: &mut dyn SkinProvider) -> TextureSource {
        let source = if self.game_settings.beatmap_skin { TextureSource::Beatmap(beatmap_path.to_owned()) } else { TextureSource::Skin };

        self.cursor.reload_skin(skin_manager).await;
        self.judgment_helper.reload_skin(skin_manager).await;
        self.follow_point_image = skin_manager.get_texture("followpoint", &source, SkinUsage::Gamemode, false).await;

        let combo_colors = if self.game_settings.use_beatmap_combo_colors && !self.beatmap_combo_colors.is_empty() {
            self.beatmap_combo_colors.clone()
        } else if !skin_manager.skin().combo_colors.is_empty() {
            skin_manager.skin().combo_colors.clone()
        } else {
            self.game_settings.combo_colors.iter().map(Color::from_hex).collect()
        };

        self.apply_combo_colors(combo_colors);

        for n in self.notes.iter_mut() {
            n.reload_skin(&source, skin_manager).await;
        }

        let smoke = skin_manager.get_texture("cursor-smoke", &source, SkinUsage::Gamemode, false).await.map(|i| i.tex).unwrap_or_default();
        if let Some(emitter) = &mut self.smoke_emitter {
            emitter.image = smoke;
        } else {
            // create the emitter
            let emitter = EmitterBuilder::new()
                .should_emit(false)
                .spawn_delay(10.0)
                .life(500.0..2000.0)
                .image(smoke)
                .scale(EmitterVal::init_only(0.8..1.5))
                .opacity(EmitterVal::new(1.0..1.0, 1.0..0.0))
                .rotation(EmitterVal::init_only(0.0..PI*2.0))
                .color(Color::WHITE)
                .build(0.0);

            self.smoke_emitter = Some(emitter);
        }

        source
    }

    async fn apply_mods(&mut self, mods: Arc<ModManager>) {
        let had_easy_or_hr = self.mods.has_mod(Easy) || self.mods.has_mod(HardRock);

        let has_hr = mods.has_mod(HardRock);
        let has_easy_or_hr = mods.has_mod(Easy) || has_hr;

        let had_otb = self.mods.has_mod(OnTheBeat);
        let has_otb = mods.has_mod(OnTheBeat);

        // check easing type
        let easing_type_names = ["in", "out", "inout"];
        let mut last_easing_type = "";
        let mut new_easing_type = "";
        for i in easing_type_names {
            if self.mods.has_mod(i) { last_easing_type = i }
            if mods.has_mod(i) { new_easing_type = i }
        }

        // check easing
        let easing_names = ["sine", "quad", "cube", "quart", "quint", "exp", "circ", "back"];
        let mut last_easing = "";
        let mut new_easing = "";
        for i in easing_names {
            if self.mods.has_mod(i) { last_easing = i }
            if mods.has_mod(i) { new_easing = i }
        }

        self.mods = mods;

        let mut set_ar = None;
        let mut set_easing = None;

        if has_easy_or_hr || had_easy_or_hr != has_easy_or_hr {
            let cs = Self::get_cs(&self.metadata, &self.mods);
            let ar = Self::get_ar(&self.metadata, &self.mods);
            
            // use existing settings, we only want to change the cs
            let pos = self.scaling_helper.settings_offset;
            let size = self.scaling_helper.window_size;
            let scale = self.scaling_helper.settings_scale;

            self.apply_playfield(Arc::new(ScalingHelper::new_offset_scale(cs, size, pos, scale, has_hr))).await;
            self.setup_hitwindows();

            set_ar = Some(ar);
        }
    
        if last_easing != new_easing || last_easing_type != new_easing_type {
            // use out as default easing type
            if new_easing_type.is_empty() && !new_easing.is_empty() {
                new_easing_type = "out"
            }

            let easing = match (new_easing_type, new_easing) {
                // sine
                ("in", "sine") => Easing::EaseInSine,
                ("out", "sine") => Easing::EaseOutSine,
                ("inout", "sine") => Easing::EaseInOutSine,
                // quadratic
                ("in", "quad") => Easing::EaseInQuadratic,
                ("out", "quad") => Easing::EaseOutQuadratic,
                ("inout", "quad") => Easing::EaseInOutQuadratic,
                // cubic
                ("in", "cube") => Easing::EaseInCubic,
                ("out", "cube") => Easing::EaseOutCubic,
                ("inout", "cube") => Easing::EaseInOutCubic,
                // quartic
                ("in", "quart") => Easing::EaseInQuartic,
                ("out", "quart") => Easing::EaseOutQuartic,
                ("inout", "quart") => Easing::EaseInOutQuartic,
                // quintic
                ("in", "quint") => Easing::EaseInQuintic,
                ("out", "quint") => Easing::EaseOutQuintic,
                ("inout", "quint") => Easing::EaseInOutQuintic,
                // exponential
                ("in", "exp") => Easing::EaseInExponential,
                ("out", "exp") => Easing::EaseOutExponential,
                ("inout", "exp") => Easing::EaseInOutExponential,
                // // circular
                // ("in", "circ") => Easing::EaseInCircular,
                // ("out", "circ") => Easing::EaseOutCircular,
                // // back
                // ("in", "back") => Easing::EaseInBack      (1.7, 1.7 * 1.525),
                // ("out", "back") => Easing::EaseOutBack    (1.7, 1.7 * 1.525),
                // ("inout", "back") => Easing::EaseInOutBack(1.7, 1.7 * 1.525),
                _ => Easing::Linear
            };

            set_easing = Some(easing);
        }
        
        if has_otb != had_otb {
            if has_otb {
                let timing_points = self.timing_points.iter().filter(|t| !t.is_inherited()).cloned().collect::<Vec<_>>();
                let mut index = 0;
                // info!("tp: {} -> {}", timing_points[index].time, timing_points[index].beat_length);
                
                for note in self.notes.iter_mut() {
                    // check next timing point
                    if let Some(next) = timing_points.get(index + 1) {
                        if next.time <= note.time() { 
                            index += 1; 
                            // info!("tp: {} -> {}", timing_points[index].time, timing_points[index].beat_length);
                        }
                    }

                    // get the beat length of the current timing point
                    let beat_length = timing_points[index].beat_length;

                    // normalize the note time to "align" with the control point time offset
                    let normalized_time = note.time() - timing_points[index].time; // beat lengths with decimal points
                    let m = beat_length - (normalized_time % beat_length); // beat lengths without a decimal point
                    let m2 = normalized_time % beat_length;
                    // info!("{normalized_time}, {m}, {m2}");

                    // if this note lands on a beat, or within 10ms of a beat, make it ~funky~
                    if m < 10.0 || m2 < 10.0 {
                        note.set_approach_easing(Easing::EaseOutExponential)
                    } else {
                        note.set_approach_easing(Easing::Linear)
                    }
                    
                }

                set_easing = None;
            } else {
                set_easing = Some(Easing::Linear);
            }

        }

        if set_ar.is_some() || set_easing.is_some() {
            for note in self.notes.iter_mut() {
                if let Some(easing) = set_easing {
                    note.set_approach_easing(easing)
                }
                if let Some(ar) = set_ar {
                    note.set_ar(ar);
                }
            }
        }

    }

    
    fn unpause(&mut self) {
        // info!("unpause");
        if self.use_controller_cursor {
            // info!("using to controller input");
            // CursorManager::set_gamemode_override(true);
        } 
        // else {
        //     info!("using mouse input");
        // }
    }

    
    async fn beat_happened(&mut self, pulse_length: f32) {
        self.notes.iter_mut().for_each(|n|n.beat_happened(pulse_length));
    }
    async fn kiai_changed(&mut self, is_kiai: bool) {
        self.notes.iter_mut().for_each(|n|n.kiai_changed(is_kiai));
    }
}

#[async_trait]
#[cfg(feature="graphics")]
impl GameModeInput for OsuGame {
    async fn key_down(&mut self, key: Key) -> Option<ReplayAction> {
        // playfield adjustment
        if key == Key::LControl {
            let old = self.game_settings.get_playfield();
            self.move_playfield = Some((old.1, self.window_mouse_pos));
            return None;
        }

        let key = self.map_key(&key)?;

        // if relax is enabled, and the user doesn't want manual input, return
        if self.mods.has_mod(Relax) {
            if !self.game_settings.manual_input_with_relax { return None; }
            self.relax_manager.key_pressed(key);
        }

        Some(ReplayAction::Press(key))
    }
    
    async fn key_up(&mut self, key: Key) -> Option<ReplayAction> {
        // playfield adjustment
        if key == Key::LControl {
            self.move_playfield = None;
            return None;
        }

        let key = self.map_key(&key)?;

        // if relax is enabled, and the user doesn't want manual input, return
        if self.mods.has_mod(Relax) {
            if !self.game_settings.manual_input_with_relax { return None; }
            self.relax_manager.key_released(key);
        }

        Some(ReplayAction::Release(key))
    }
    

    async fn mouse_move(
        &mut self, 
        pos: Vector2
    ) -> Option<ReplayAction> {
        if self.use_controller_cursor {
            // info!("switched to mouse");
            self.use_controller_cursor = false;
        }
        self.window_mouse_pos = pos;
        
        if let Some((original, mouse_start)) = self.move_playfield {
            
            let mut settings = (*self.game_settings).clone();
            let mut change = original + (pos - mouse_start);

            // check playfield snapping
            // TODO: can this be simplified?
            let playfield_size = self.scaling_helper.playfield_scaled_with_cs_border.size;

            // what the offset should be if playfield is centered
            let center_offset = (self.window_size.0 - FIELD_SIZE * self.scaling_helper.scale) / 2.0 - (self.window_size.0 - playfield_size) / 2.0;

            let snap_threshold = settings.playfield_snap;
            if (center_offset.x - change.x).abs() < snap_threshold {
                change.x = center_offset.x;
            }
            if (center_offset.y - change.y).abs() < snap_threshold {
                change.y = center_offset.y;
            }

            settings.playfield_x_offset = change.x;
            settings.playfield_y_offset = change.y;
            
            
            let settings2 = settings.clone();
            self.actions.push(GameAction::UpdateSettings(Box::new(move |settings| settings.osu_settings = settings2 )));

            self.game_settings = Arc::new(settings);
            self.recalculate_playfield().await;
            return None;
        }
        

        // convert window pos to playfield pos
        let pos = self.scaling_helper.descale_coords(pos);
        Some(ReplayAction::MousePos(pos.x, pos.y))
    }
    
    async fn mouse_down(&mut self, btn: MouseButton) -> Option<ReplayAction> {
        // if the user has mouse input disabled, return
        if self.game_settings.ignore_mouse_buttons { return None }
        
        let button = self.map_btn(&btn)?;

        // if relax is enabled, and the user doesn't want manual input, return
        if self.mods.has_mod(Relax) {
            if !self.game_settings.manual_input_with_relax { return None; }
            self.relax_manager.key_pressed(button);
        }

        Some(ReplayAction::Press(button))
    }
    
    async fn mouse_up(&mut self, btn: MouseButton) -> Option<ReplayAction> {
        // if the user has mouse input disabled, return
        if self.game_settings.ignore_mouse_buttons { return None }

        let button = self.map_btn(&btn)?;

        // if relax is enabled, and the user doesn't want manual input, return
        if self.mods.has_mod(Relax) {
            if !self.game_settings.manual_input_with_relax { return None; }
            self.relax_manager.key_released(button);
        }

        Some(ReplayAction::Release(button))
    }

    async fn mouse_scroll(&mut self, delta: f32) -> Option<ReplayAction> {
        if self.move_playfield.is_some() {
            let delta = delta / 40.0;
            let mut a = (*self.game_settings).clone();
            a.playfield_scale += delta;
            self.game_settings = Arc::new(a);

            self.actions.push(GameAction::UpdateSettings(Box::new(move |settings| settings.osu_settings.playfield_scale += delta )));

            self.recalculate_playfield().await;
        }

        None
    }


    async fn controller_press(&mut self, _: &GamepadInfo, btn: ControllerButton) -> Option<ReplayAction> {
        // if relax is enabled, and the user doesn't want manual input, return
        if self.mods.has_mod(Relax) && !self.game_settings.manual_input_with_relax { return None; }

        match btn {
            ControllerButton::LeftTrigger => Some(ReplayAction::Press(KeyPress::Left)),
            ControllerButton::RightTrigger => Some(ReplayAction::Press(KeyPress::Right)),
            _ => None
        }
    }
    
    async fn controller_release(&mut self, _: &GamepadInfo, btn: ControllerButton) -> Option<ReplayAction> {
        // if relax is enabled, and the user doesn't want manual input, return
        if self.mods.has_mod(Relax) && !self.game_settings.manual_input_with_relax { return None; }

        match btn {
            ControllerButton::LeftTrigger => Some(ReplayAction::Release(KeyPress::Left)),
            ControllerButton::RightTrigger => Some(ReplayAction::Release(KeyPress::Right)),
            _ => None
        }
    }
    
    async fn controller_axis(&mut self, _: &GamepadInfo, axis_data: HashMap<Axis, (bool, f32)>) -> Option<ReplayAction> {
        if !self.use_controller_cursor {
            // info!("switched to controller input");
            // CursorManager::set_gamemode_override(true);
            self.use_controller_cursor = true;
        }

        let mut new_pos = self.mouse_pos;
        let scaling_helper = self.scaling_helper.clone();
        let playfield = scaling_helper.playfield_scaled_with_cs_border;

        for (axis, &(new, value)) in axis_data.iter() {
            if new {
                match *axis {
                    Axis::LeftStickX => {
                        // -1.0 to 1.0
                        // where -1 is 0, and 1 is scaling_helper.playfield_scaled_with_cs_border.whatever
                        let normalized = (value + 1.0) / 2.0;
                        new_pos.x = playfield.pos.x + f32::lerp(0.0, playfield.size.x, normalized);
                    }
                    Axis::LeftStickY => {
                        // y is upside down in gilrs i guess?
                        let normalized = (value + 1.0) / 2.0;
                        new_pos.y = playfield.pos.y + f32::lerp(playfield.size.y, 0.0, normalized);
                    }
                    _ => {},
                }
            }
        }

        let new_pos = scaling_helper.descale_coords(new_pos);
        Some(ReplayAction::MousePos(new_pos.x, new_pos.y))
    }

}


#[cfg(not(feature="graphics"))]
impl GameModeInput for OsuGame {}

#[async_trait]
impl GameModeProperties for OsuGame {
    fn playmode(&self) -> Cow<'static, str> { Cow::Borrowed("osu") }
    fn end_time(&self) -> f32 { self.end_time }
    fn show_cursor(&self) -> bool { false } // we have our own cursor

    fn get_info(&self) -> GameModeInfo { crate::GAME_INFO }

    fn get_possible_keys(&self) -> Vec<(KeyPress, &str)> {
        vec![
            (KeyPress::Left, "L"),
            (KeyPress::Right, "R"),
            (KeyPress::LeftMouse, "M1"),
            (KeyPress::RightMouse, "M2"),
        ]
    }

    fn timing_bar_things(&self) -> Vec<(f32, Color)> {
        self.hit_windows
            .iter()
            .map(|(j, w)| (w.end, j.color))
            .collect()
    }

    async fn get_ui_elements(
        &self, 
        window_size: Vector2, 
        ui_elements: &mut Vec<UIElement>,
        loader: &mut dyn UiElementLoader
    ) {
        let playmode = self.playmode();
        let get_name = |name| {
            format!("{playmode}_{name}")
        };

        let size = Vector2::new(100.0, 30.0);
        let combo_bounds = Bounds::new(
            Vector2::ZERO,
            size
        );
        
        // combo
        ui_elements.push(loader.load(
            &get_name("combo".to_owned()),
            Vector2::new(0.0, window_size.y - (size.y + DURATION_HEIGHT + 10.0)),
            Box::new(ComboElement::new(combo_bounds).await)
        ).await);

        // Leaderboard
        ui_elements.push(loader.load(
            &get_name("leaderboard".to_owned()),
            Vector2::with_y(window_size.y / 3.0),
            Box::new(LeaderboardElement::new(crate::GAME_INFO).await)
        ).await);
        
    }
    
}
