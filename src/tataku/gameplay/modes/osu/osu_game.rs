use std::ops::Range;

use crate::prelude::*;
use super::prelude::*;

const NOTE_DEPTH:Range<f32> = 100.0..200.0;
const SLIDER_DEPTH:Range<f32> = 200.0..300.0;

const STACK_LENIENCY:u32 = 3;
pub const PREEMPT_MIN:f32 = 450.0;


pub struct OsuGame {
    // lists
    pub notes: Vec<Box<dyn OsuHitObject>>,

    // hit timing bar stuff
    hit_windows: Vec<(OsuHitJudgments, Range<f32>)>,
    miss_window: f32,

    // draw_points: Vec<(f32, Vector2, ScoreHit)>,
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

    /// cached settings, saves on locking
    game_settings: Arc<StandardSettings>,

    /// autoplay helper
    auto_helper: StandardAutoHelper,

    /// list of note_indices which are new_combos
    new_combos: Vec<usize>,

    use_controller_cursor: bool,
    window_size: Arc<WindowSize>,
    end_time: f32,

    follow_point_image: Option<Image>,
    
    judgment_helper: JudgmentImageHelper,

    metadata: Arc<BeatmapMeta>,
    mods: Arc<ModManager>
}
impl OsuGame {
    async fn playfield_changed(&mut self) {
        let new_scale = Arc::new(ScalingHelper::new(self.cs, self.window_size.0, self.mods.has_mod(HardRock.name())).await);
        self.apply_playfield(new_scale).await
    }
    async fn apply_playfield(&mut self, playfield: Arc<ScalingHelper>) {
        self.scaling_helper = playfield.clone();

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

    
    fn add_judgement_indicator(pos: Vector2, time: f32, hit_value: &OsuHitJudgments, scaling_helper: &Arc<ScalingHelper>, judgment_helper: &JudgmentImageHelper, settings: &StandardSettings, manager: &mut IngameManager) {
        if !hit_value.should_draw() { return }

        let color = hit_value.color();
        let mut image = if settings.use_skin_judgments { judgment_helper.get_from_scorehit(hit_value) } else { None };
        if let Some(image) = &mut image {
            image.pos = pos;
            image.depth = -2.0;

            let scale = Vector2::ONE * scaling_helper.scaled_cs;
            image.scale = scale;
        }

        manager.add_judgement_indicator(BasicJudgementIndicator::new(
            pos, 
            time,
            -99999.99, // TODO: do this properly
            CIRCLE_RADIUS_BASE * scaling_helper.scaled_cs * (1.0/3.0),
            color,
            image
        ))
    }


    #[inline]
    fn scale_by_mods<V:std::ops::Mul<Output=V>>(val:V, ez_scale: V, hr_scale: V, mods: &ModManager) -> V {
        if mods.has_mod(Easy.name()) {
            val * ez_scale
        } else if mods.has_mod(HardRock.name()) {
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
        Self::scale_by_mods(meta.cs, 0.5, 1.3, &mods).clamp(1.0, 10.0)
    }
}

#[async_trait]
impl GameMode for OsuGame {
    async fn new(map:&Beatmap, diff_calc_only: bool) -> TatakuResult<Self> {
        let metadata = map.get_beatmap_meta();
        let mods = ModManager::get();
        let window_size = WindowSize::get();
        let effective_window_size = if diff_calc_only { super::diff_calc::WINDOW_SIZE } else { window_size.0 };
        
        let settings = get_settings!().standard_settings.clone();

        let cs = Self::get_cs(&metadata, &mods);
        let ar = Self::get_ar(&metadata, &mods);
        let od = Self::get_od(&metadata, &mods);
        let scaling_helper = Arc::new(ScalingHelper::new(cs, effective_window_size, mods.has_mod(HardRock.name())).await);

        let skin_combo_colors = &SkinManager::current_skin_config().await.combo_colors;
        let mut combo_colors = if skin_combo_colors.len() > 0 {
            skin_combo_colors.clone()
        } else {
            settings.combo_colors.iter().map(|c|Color::from_hex(c)).collect()
        };

        let judgment_helper = JudgmentImageHelper::new(OsuHitJudgments::Miss).await;
        let follow_point_image = SkinManager::get_texture("followpoint", true).await;

        let timing_points = map.get_timing_points();

        let mut s = match map {
            Beatmap::Osu(beatmap) => {
                let stack_leniency = beatmap.stack_leniency;
                let std_settings = Arc::new(settings);

                if std_settings.use_beatmap_combo_colors && beatmap.combo_colors.len() > 0 {
                    combo_colors = beatmap.combo_colors.clone();
                }
                
                let get_hitsounds = |time, hitsound, hitsamples| {
                    let tp = timing_points.timing_point_at(time);
                    Hitsound::from_hitsamples(hitsound, hitsamples, true, tp)
                };

                let mut s = Self {
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
                    new_combos: Vec::new(),
                    stack_leniency,
                    window_size,
                    follow_point_image,
                    judgment_helper,
                    metadata,
                    mods
                };
                
                // join notes and sliders into a single array
                // needed because of combo counts
                let mut all_items = Vec::new();
                for note in beatmap.notes.iter() {
                    all_items.push((Some(note), None, None));
                    s.end_time = s.end_time.max(note.time);
                }
                for slider in beatmap.sliders.iter() {
                    all_items.push((None, Some(slider), None));
        
                    // can this be improved somehow?
                    if slider.curve_points.len() == 0 || slider.length == 0.0 {
                        s.end_time = s.end_time.max(slider.time);
                    } else {
                        let curve = get_curve(slider, &map);
                        s.end_time = s.end_time.max(curve.end_time);
                    }
                }
                for spinner in beatmap.spinners.iter() {
                    all_items.push((None, None, Some(spinner)));
                    s.end_time = s.end_time.max(spinner.end_time);
                }
                // sort
                all_items.sort_by(|a, b| {
                    let a_time = match a {
                        (Some(note), None, None) => note.time,
                        (None, Some(slider), None) => slider.time,
                        (None, None, Some(spinner)) => spinner.time,
                        _ => 0.0
                    };
                    let b_time = match b {
                        (Some(note), None, None) => note.time,
                        (None, Some(slider), None) => slider.time,
                        (None, None, Some(spinner)) => spinner.time,
                        _ => 0.0
                    };
        
                    a_time.partial_cmp(&b_time).unwrap()
                });
        

                // add notes
                let mut combo_num = 0;
                let mut combo_change = 0;
        
                // used for the end time
                let end_time = s.end_time;
        
                let mut counter = 0;
        
                for (note, slider, spinner) in all_items {
                    // check for new combo
                    if let Some(note) = note { if note.new_combo { combo_num = 0 } }
                    if let Some(slider) = slider { if slider.new_combo { combo_num = 0 } }
                    if let Some(_spinner) = spinner { combo_num = 0 }
        
                    // if new combo, increment new combo counter
                    if combo_num == 0 {
                        combo_change += 1;
                        s.new_combos.push(counter);
                    }
                    // get color
                    let color = combo_colors[(combo_change - 1) % combo_colors.len()];
                    // update combo number
                    combo_num += 1;
        
                    if let Some(note) = note {
                        let depth = f32::lerp(NOTE_DEPTH.start, NOTE_DEPTH.end, note.time / end_time);
                        s.notes.push(Box::new(OsuNote::new(
                            note.clone(),
                            ar,
                            color,
                            combo_num as u16,
                            scaling_helper.clone(),
                            depth,
                            std_settings.clone(),
                            get_hitsounds(note.time, note.hitsound, note.hitsamples.clone())
                        ).await));
                    }
                    if let Some(slider) = slider {
                        // invisible note
                        if slider.curve_points.len() == 0 || slider.length == 0.0 {
                            let note = NoteDef {
                                pos: slider.pos,
                                time: slider.time,
                                hitsound: slider.hitsound,
                                hitsamples: slider.hitsamples.clone(),
                                new_combo: slider.new_combo,
                                color_skip: slider.color_skip,
                            };
        
                            let depth = f32::lerp(NOTE_DEPTH.start, NOTE_DEPTH.end, note.time / end_time);
                            let hitsounds = get_hitsounds(note.time, note.hitsound, note.hitsamples.clone());
                            s.notes.push(Box::new(OsuNote::new(
                                note,
                                ar,
                                Color::new(0.0, 0.0, 0.0, 1.0),
                                combo_num as u16,
                                scaling_helper.clone(),
                                depth,
                                std_settings.clone(),
                                hitsounds,
                            ).await));
                        } else {
                            // let slider_depth = SLIDER_DEPTH.start + (slider.time / end_time) * SLIDER_DEPTH.end;
                            // let depth = NOTE_DEPTH.start + (slider.time / end_time) * NOTE_DEPTH.end;

                            let slider_depth = f32::lerp(SLIDER_DEPTH.start, SLIDER_DEPTH.end, slider.time / end_time);
                            let depth = f32::lerp(NOTE_DEPTH.start, NOTE_DEPTH.end, slider.time / end_time);
        
                            let curve = get_curve(slider, &map);
                            s.notes.push(Box::new(OsuSlider::new(
                                slider.clone(),
                                curve,
                                ar,
                                color,
                                combo_num as u16,
                                scaling_helper.clone(),
                                slider_depth,
                                depth,
                                std_settings.clone(),
                                get_hitsounds,
                                beatmap.slider_velocity_at(slider.time)
                            ).await))
                        }
                        
                    }
                    if let Some(spinner) = spinner {
                        let duration = spinner.end_time - spinner.time;
                        let min_rps = map_difficulty(od, 2.0, 4.0, 6.0) * 0.6;
                        
                        s.notes.push(Box::new(OsuSpinner::new(
                            spinner.clone(),
                            scaling_helper.clone(),
                            (duration / 1000.0 * min_rps) as u16
                        ).await))
                    }
                    
                    counter += 1;
                }
        
                s
            }
            
            _ => return Err(BeatmapError::UnsupportedMode.into()),
        };

        // wait an extra sec
        s.end_time += 1000.0;

        s.setup_hitwindows();

        if !diff_calc_only {
            for n in s.notes.iter_mut() {
                n.reload_skin().await;
            }
        }

        Ok(s)
    }

    async fn handle_replay_frame(&mut self, frame:ReplayFrame, time:f32, manager:&mut IngameManager) {
        const ALLOWED_PRESSES:&[KeyPress] = &[
            KeyPress::Left, 
            KeyPress::Right,
            KeyPress::LeftMouse,
            KeyPress::RightMouse,
        ];

        match frame {
            ReplayFrame::Press(key) if ALLOWED_PRESSES.contains(&key) => {
                self.hold_count += 1;

                match key {
                    KeyPress::Left | KeyPress::LeftMouse => CursorManager::left_pressed(true, true),
                    KeyPress::Right | KeyPress::RightMouse => CursorManager::right_pressed(true, true),
                    _ => {}
                }

                let mut check_notes = Vec::new();
                for note in self.notes.iter_mut() {
                    note.press(time);
                    // check if note is in hitwindow, has not yet been hit, and is not a spinner
                    if (time - note.time()).abs() <= self.miss_window && !note.was_hit() && note.note_type() != NoteType::Spinner {
                        check_notes.push(note);
                    }
                }
                if check_notes.len() == 0 { return } // no notes to check
                check_notes.sort_by(|a, b| a.time().partial_cmp(&b.time()).unwrap());
                

                let note = &mut check_notes[0];
                let note_time = note.time();

                // spinners are a special case. hit windows don't affect them, or else you can miss for no reason lol
                // if note.note_type() == NoteType::Spinner {
                //     return;
                // }

                // check distance
                if note.check_distance(self.mouse_pos) {
                    if let Some(judge) = manager.check_judgment(&self.hit_windows, time, note_time).await {
                        note.set_judgment(judge);

                        if judge == &OsuHitJudgments::X300 && !self.game_settings.show_300s {
                            // dont show the judgment
                        } else {
                            Self::add_judgement_indicator(note.point_draw_pos(time), time, judge, &self.scaling_helper, &self.judgment_helper, &self.game_settings, manager);
                        }

                        if let OsuHitJudgments::Miss = judge {
                            // tell the note it was missed
                            note.miss();
                        } else {
                            // tell the note it was hit
                            note.hit(time);

                            // play the sound
                            let hitsound = note.get_hitsound();
                            manager.play_note_sound(&hitsound).await;
                        }

                    }
                }
            }
            // dont continue if no keys were being held (happens when leaving a menu)
            ReplayFrame::Release(key) if ALLOWED_PRESSES.contains(&key) && self.hold_count > 0 => {
                self.hold_count -= 1;

                match key {
                    KeyPress::Left | KeyPress::LeftMouse => CursorManager::left_pressed(false, true),
                    KeyPress::Right | KeyPress::RightMouse => CursorManager::right_pressed(false, true),
                    _ => {}
                }

                let mut check_notes = Vec::new();
                for note in self.notes.iter_mut() {
                    // if this is the last key to be released
                    if self.hold_count == 0 {
                        note.release(time)
                    }

                    // check if note is in hitwindow
                    if time >= note.end_time(self.miss_window) && !note.was_hit() && note.note_type() == NoteType::Slider {
                        check_notes.push(note);
                    }
                }
            }
            ReplayFrame::MousePos(x, y) => {
                // scale the coords from playfield to window
                let pos = self.scaling_helper.scale_coords(Vector2::new(x, y));
                self.mouse_pos = pos;

                for note in self.notes.iter_mut() {
                    note.mouse_move(pos);
                }
            }
            _ => {}
        }
    }


    async fn update(&mut self, manager:&mut IngameManager, time:f32) -> Vec<ReplayFrame> {
        let mut pending_frames = Vec::new();

        let has_autoplay = self.mods.has_autoplay();
        let has_relax = self.mods.has_mod(Relax.name());

        // do autoplay things
        if has_autoplay {

            self.auto_helper.update(time, &self.notes, &self.scaling_helper, &mut pending_frames);

            // // handle presses and mouse movements now, and releases later
            // for frame in pending_frames.iter() {
            //     self.handle_replay_frame(*frame, time, manager).await;
            // }
        }



        // if the map is over, say it is
        if time >= self.end_time {
            manager.completed = true;
            return Vec::new();
        }

        // update notes
        for note in self.notes.iter_mut() {
            note.update(time).await;
            let end_time = note.end_time(self.miss_window);

            // play queued sounds
            for hitsound in note.get_sound_queue() {
                manager.play_note_sound(&hitsound).await;
            }

            for (add_combo, pos) in note.pending_combo() {
                manager.add_judgment(&add_combo).await;
                Self::add_judgement_indicator(pos, time, &add_combo, &self.scaling_helper, &self.judgment_helper, &self.game_settings, manager);
            }


            if has_relax && !has_autoplay {
                // if its time to hit the note, the not hasnt been hit yet, and we're within the note's radius
                if time >= note.time() && time < end_time && !note.was_hit() && note.check_distance(self.mouse_pos) {
                    let key = KeyPress::LeftMouse;

                    match note.note_type() {
                        NoteType::Note => {
                            pending_frames.push(ReplayFrame::Press(key));
                            pending_frames.push(ReplayFrame::Release(key));
                        }
                        NoteType::Slider | NoteType::Spinner | NoteType::Hold => {
                            // make sure we're not already holding
                            if let Some(false) = manager.key_counter.keys.get(&key).map(|a|a.held) {
                                pending_frames.push(ReplayFrame::Press(key));
                            }
                        }
                    }
                }

                if time >= end_time && !note.was_hit() {
                    let key = KeyPress::LeftMouse;

                    match note.note_type() {
                        NoteType::Note => {}
                        NoteType::Slider | NoteType::Spinner | NoteType::Hold => {
                            // assume we're holding i guess?
                            pending_frames.push(ReplayFrame::Release(key));
                        }
                    }
                }
            }


            // check if note was missed
            
            // if the time is leading in, we dont want to check if any notes have been missed
            if time < 0.0 { continue }

            // check if note is in hitwindow
            if time >= end_time && !note.was_hit() {

                // check if we missed the current note
                match note.note_type() {
                    NoteType::Note => {
                        let j = OsuHitJudgments::Miss;
                        manager.add_judgment(&j).await;

                        Self::add_judgement_indicator(note.point_draw_pos(time), time, &j, &self.scaling_helper, &self.judgment_helper, &self.game_settings, manager);
                    }
                    NoteType::Slider => {
                        // check slider release points
                        // internally checks distance
                        let judge = &note.check_release_points(time);
                        manager.add_judgment(judge).await;
                        
                        if judge == &OsuHitJudgments::X300 && !self.game_settings.show_300s {
                            // dont show the judgment
                        } else {
                            Self::add_judgement_indicator(note.point_draw_pos(time), time, judge, &self.scaling_helper, &self.judgment_helper, &self.game_settings, manager);
                        }

                        if let OsuHitJudgments::Miss = judge {
                            // // tell the note it was missed
                            // unecessary bc its told it was missed later lol
                            // info!("missed slider");
                            // note.miss();
                        } else {
                            // tell the note it was hit
                            note.hit(time);

                            // play the sound
                            let hitsound = note.get_hitsound();
                            // let hitsamples = note.get_hitsamples().clone();
                            manager.play_note_sound(&hitsound).await;
                        }
                    }

                    NoteType::Spinner => {
                        let j = OsuHitJudgments::SpinnerMiss;
                        manager.add_judgment(&j).await;

                        Self::add_judgement_indicator(note.point_draw_pos(time), time, &j, &self.scaling_helper, &self.judgment_helper, &self.game_settings, manager);
                    }

                    _ => {},
                }
                
                // force the note to be misssed
                note.miss(); 
            }
        }

        // // handle note releases
        // // required because autoplay frames are checked after the frame is processed
        // // so if the key is released on the same frame its checked, it will count as not held
        // // which makes sense, but we dont want that
        if manager.current_mods.has_autoplay() {
            pending_frames.extend(self.auto_helper.get_release_queue());
            // for frame in self.auto_helper.get_release_queue() {
            //     self.handle_replay_frame(frame, time, manager).await;
            // }
        }

        pending_frames
    }
    
    async fn draw(&mut self, manager:&mut IngameManager, list: &mut RenderableCollection) {

        // draw the playfield
        if !manager.menu_background {
            let mut playfield = self.scaling_helper.playfield_scaled_with_cs_border.clone();

            if self.move_playfield.is_some() {
                let line_size = self.game_settings.playfield_movelines_thickness;
                // draw x and y center lines
                let px_line = Line::new(
                    playfield.pos + Vector2::new(0.0, playfield.size.y/2.0),
                    playfield.pos + Vector2::new(playfield.size.x, playfield.size.y/2.0),
                    line_size,
                    -100.0,
                    Color::WHITE
                );
                let py_line = Line::new(
                    playfield.pos + Vector2::new(playfield.size.x/2.0, 0.0),
                    playfield.pos + Vector2::new(playfield.size.x/2.0, playfield.size.y),
                    line_size, 
                    -100.0,
                    Color::WHITE
                );

                let wx_line = Line::new(
                    Vector2::new(0.0, self.window_size.y/2.0),
                    Vector2::new(self.window_size.x, self.window_size.y/2.0),
                    line_size,
                    -100.0,
                    Color::WHITE
                );
                let wy_line = Line::new(
                    Vector2::new(self.window_size.x/2.0, 0.0),
                    Vector2::new(self.window_size.x/2.0, self.window_size.y),
                    line_size, 
                    -100.0,
                    Color::WHITE
                );

                playfield.border = Some(Border::new(Color::WHITE, line_size));

                list.push(wx_line);
                list.push(wy_line);
                list.push(px_line);
                list.push(py_line);
            }

            if manager.current_timing_point().kiai {
                playfield.border = Some(Border::new(Color::YELLOW, 2.0));
            }
            list.push(playfield);
        }


        // if this is a replay, we need to draw the replay curser
        if manager.replaying || manager.current_mods.has_autoplay() || self.use_controller_cursor {
            CursorManager::set_pos(self.mouse_pos)
        }

        // draw notes
        for note in self.notes.iter_mut() {
            note.draw(list).await;
        }

        // draw follow points
        let time = manager.time();
        if self.game_settings.draw_follow_points {
            if self.notes.len() == 0 { return }

            let follow_dot_size = 3.0 * self.scaling_helper.scale;
            let follow_dot_distance = 20.0 * self.scaling_helper.scale;

            for i in 0..self.notes.len() - 1 {
                if !self.new_combos.contains(&(i + 1)) {
                    let n1 = &self.notes[i];
                    let n2 = &self.notes[i + 1];

                    // skip if either note is a spinner
                    if n1.note_type() == NoteType::Spinner { continue }
                    if n2.note_type() == NoteType::Spinner { continue }

                    let preempt = n2.get_preempt();
                    let n1_time = n1.time();
                    if time < n1_time - preempt { continue } //|| time > n2.end_time(0.0) {continue}
                    let n2_time = n2.time();
                    if time >= n2_time { continue }//|| time <= n1_time {continue}


                    // setup follow points and the time they should exist at
                    let n1_pos = n1.pos_at(n2_time);
                    let n2_pos = n2.pos_at(n2_time);
                    let distance = n1_pos.distance(n2_pos);
                    let direction = PI * 2.0 - Vector2::atan2(n2_pos - n1_pos);
                    
                    let follow_dot_count = distance / follow_dot_distance;
                    for i in 1..follow_dot_count as u64 {
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
                            const FOLLOW_DOT_TEX_SIZE: Vector2 = Vector2::new(128.0, 128.0);
                            i.pos = point;
                            i.rotation = direction;
                            // i.current_scale = Vector2::ONE * self.scaling_helper.scale;
                            list.push(i);
                        } else {
                            list.push(Circle::new(
                                Color::WHITE.alpha(alpha),
                                100_000.0,
                                point,
                                follow_dot_size,
                                None
                            ));
                        }

                    }
                }
            }
        }
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
    }

    fn skip_intro(&mut self, manager: &mut IngameManager) {
        if self.notes.len() == 0 {return}

        let time = self.notes[0].time() - self.notes[0].get_preempt();
        if time < manager.time() {return}

        if time < 0.0 { return }
        manager.song.set_position(time);
    }

    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.window_size = window_size;
        self.playfield_changed().await;
    }


    async fn fit_to_area(&mut self, pos: Vector2, size: Vector2) {
        let pos = pos;
        let size = size;
        let playfield = ScalingHelper::new_offset_scale(self.cs, size, pos, 0.5, self.mods.has_mod(HardRock.name()));
        self.apply_playfield(Arc::new(playfield)).await;
    }

    async fn time_jump(&mut self, new_time:f32) {
        for n in self.notes.iter_mut() {
            n.time_jump(new_time).await;
        }
    }
    
    async fn force_update_settings(&mut self, settings: &Settings) {
        let settings = settings.standard_settings.clone();
        let settings = Arc::new(settings);

        if self.game_settings == settings { return }

        self.game_settings = settings.clone();
        for n in self.notes.iter_mut() {
            n.set_settings(settings.clone());
        }
    }

    async fn reload_skin(&mut self) {
        self.judgment_helper.reload_skin().await;

        for n in self.notes.iter_mut() {
            n.reload_skin().await;
        }
    }

    async fn apply_mods(&mut self, mods: Arc<ModManager>) {
        let had_easy_or_hr = self.mods.has_mod(Easy.name()) || self.mods.has_mod(HardRock.name());

        let has_hr = mods.has_mod(HardRock.name());
        let has_easy_or_hr = mods.has_mod(Easy.name()) || has_hr;
        self.mods = mods;

        if has_easy_or_hr || had_easy_or_hr != has_easy_or_hr {
            let cs = Self::get_cs(&self.metadata, &self.mods);
            let ar = Self::get_ar(&self.metadata, &self.mods);
            
            // use existing settings, we only want to change the cs
            let pos = self.scaling_helper.settings_offset;
            let size = self.scaling_helper.window_size;
            let scale = self.scaling_helper.settings_scale;

            self.apply_playfield(Arc::new(ScalingHelper::new_offset_scale(cs, size, pos, scale, has_hr))).await;
            self.setup_hitwindows();
            
            for note in self.notes.iter_mut() {
                note.set_ar(ar);
            }
        }
    }

    
    fn unpause(&mut self, _manager:&mut IngameManager) {
        // info!("unpause");
        if self.use_controller_cursor {
            // info!("using to controller input");
            CursorManager::set_gamemode_override(true);
        } 
        // else {
        //     info!("using mouse input");
        // }
    }
}

#[async_trait]
impl GameModeInput for OsuGame {
    async fn key_down(&mut self, key:Key) -> Option<ReplayFrame> {
        // playfield adjustment
        if key == Key::LControl {
            let old = self.game_settings.get_playfield();
            self.move_playfield = Some((old.1, self.window_mouse_pos));
            return None;
        }

        // if relax is enabled, and the user doesn't want manual input, return
        if self.mods.has_mod(Relax.name()) && !self.game_settings.manual_input_with_relax { return None; }

        if key == self.game_settings.left_key {
            Some(ReplayFrame::Press(KeyPress::Left))
        } else if key == self.game_settings.right_key {
            Some(ReplayFrame::Press(KeyPress::Right))
        } else {
            None
        }
    }
    
    async fn key_up(&mut self, key:Key) -> Option<ReplayFrame> {
        // playfield adjustment
        if key == Key::LControl {
            self.move_playfield = None;
            return None;
        }

        // if relax is enabled, and the user doesn't want manual input, return
        if self.mods.has_mod(Relax.name()) && !self.game_settings.manual_input_with_relax { return None; }

        if key == self.game_settings.left_key {
            Some(ReplayFrame::Release(KeyPress::Left))
        } else if key == self.game_settings.right_key {
            Some(ReplayFrame::Release(KeyPress::Right))
        } else {
            None
        }
    }
    

    async fn mouse_move(&mut self, pos:Vector2) -> Option<ReplayFrame> {
        if self.use_controller_cursor {
            // info!("switched to mouse");
            CursorManager::set_gamemode_override(false);
            self.use_controller_cursor = false;
        }
        self.window_mouse_pos = pos;
        
        if let Some((original, mouse_start)) = self.move_playfield {
            {
                let settings = &mut get_settings_mut!().standard_settings;
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
            }

            self.playfield_changed().await;
            return None;
        }


        // convert window pos to playfield pos
        let pos = self.scaling_helper.descale_coords(pos);
        Some(ReplayFrame::MousePos(pos.x as f32, pos.y as f32))
    }
    
    async fn mouse_down(&mut self, btn:MouseButton) -> Option<ReplayFrame> {
        // if the user has mouse input disabled, return
        if self.game_settings.ignore_mouse_buttons { return None }

        // if relax is enabled, and the user doesn't want manual input, return
        if self.mods.has_mod(Relax.name()) && !self.game_settings.manual_input_with_relax { return None; }
        
        if btn == MouseButton::Left {
            Some(ReplayFrame::Press(KeyPress::LeftMouse))
        } else if btn == MouseButton::Right {
            Some(ReplayFrame::Press(KeyPress::RightMouse))
        } else {
            None
        }
    }
    
    async fn mouse_up(&mut self, btn:MouseButton) -> Option<ReplayFrame> {
        // if the user has mouse input disabled, return
        if self.game_settings.ignore_mouse_buttons { return None }

        // if relax is enabled, and the user doesn't want manual input, return
        if self.mods.has_mod(Relax.name()) && !self.game_settings.manual_input_with_relax { return None; }

        if btn == MouseButton::Left {
            Some(ReplayFrame::Release(KeyPress::LeftMouse))
        } else if btn == MouseButton::Right {
            Some(ReplayFrame::Release(KeyPress::RightMouse))
        } else {
            None
        }
    }

    async fn mouse_scroll(&mut self, delta:f32) -> Option<ReplayFrame> {
        if self.move_playfield.is_some() {
            {
                let settings = &mut get_settings_mut!().standard_settings;
                settings.playfield_scale += delta / 40.0;
            }

            self.playfield_changed().await;
        }

        None
    }


    async fn controller_press(&mut self, _:&GamepadInfo, btn:ControllerButton) -> Option<ReplayFrame> {
        // if relax is enabled, and the user doesn't want manual input, return
        if self.mods.has_mod(Relax.name()) && !self.game_settings.manual_input_with_relax { return None; }

        match btn {
            ControllerButton::LeftTrigger => Some(ReplayFrame::Press(KeyPress::Left)),
            ControllerButton::RightTrigger => Some(ReplayFrame::Press(KeyPress::Right)),
            _ => None
        }
    }
    
    async fn controller_release(&mut self, _:&GamepadInfo, btn:ControllerButton) -> Option<ReplayFrame> {
        // if relax is enabled, and the user doesn't want manual input, return
        if self.mods.has_mod(Relax.name()) && !self.game_settings.manual_input_with_relax { return None; }

        match btn {
            ControllerButton::LeftTrigger => Some(ReplayFrame::Release(KeyPress::Left)),
            ControllerButton::RightTrigger => Some(ReplayFrame::Release(KeyPress::Right)),
            _ => None
        }
    }
    
    async fn controller_axis(&mut self, _:&GamepadInfo, axis_data:HashMap<Axis, (bool, f32)>) -> Option<ReplayFrame> {
        if !self.use_controller_cursor {
            // info!("switched to controller input");
            CursorManager::set_gamemode_override(true);
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
        Some(ReplayFrame::MousePos(new_pos.x, new_pos.y))
    }

}

#[async_trait]
impl GameModeProperties for OsuGame {
    fn playmode(&self) -> PlayMode { "osu".to_owned() }
    fn end_time(&self) -> f32 { self.end_time }
    fn show_cursor(&self) -> bool { true }
    fn ripple_size(&self) -> Option<f32> {
        Some(self.scaling_helper.scaled_circle_size.x)
    }

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
            .map(|(j, w)| (w.end, j.color()))
            .collect()
    }

    async fn get_ui_elements(&self, window_size: Vector2, ui_elements: &mut Vec<UIElement>) {
        let playmode = self.playmode();
        let get_name = |name| {
            format!("{playmode}_{name}")
        };

        let size = Vector2::new(100.0, 30.0);
        let combo_bounds = Rectangle::bounds_only(
            Vector2::ZERO,
            size
        );
        
        // combo
        ui_elements.push(UIElement::new(
            &get_name("combo".to_owned()),
            Vector2::new(0.0, window_size.y - (size.y + DURATION_HEIGHT + 10.0)),
            ComboElement::new(combo_bounds).await
        ).await);

        // Leaderboard
        ui_elements.push(UIElement::new(
            &get_name("leaderboard".to_owned()),
            Vector2::with_y(window_size.y / 3.0),
            LeaderboardElement::new().await
        ).await);
        
    }
    
}
