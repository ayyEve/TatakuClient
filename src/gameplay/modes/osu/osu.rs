use std::ops::Range;

use crate::prelude::*;
use super::osu_notes::*;


const NOTE_DEPTH:Range<f64> = 100.0..200.0;
pub const SLIDER_DEPTH:Range<f64> = 200.0..300.0;

const STACK_LENIENCY:u32 = 3;


/// calculate the standard acc for `score`
pub fn calc_acc(score: &Score) -> f64 {
    let x50 = score.x50 as f64;
    let x100 = score.x100 as f64;
    let x300 = score.x300 as f64;
    let geki = score.xgeki as f64;
    let katu = score.xkatu as f64;
    let miss = score.xmiss as f64;

    (50.0 * x50 + 100.0 * (x100 + katu) + 300.0 * (x300 + geki)) 
    / (300.0 * (miss + x50 + x100 + x300 + katu + geki))
}

pub struct StandardGame {
    // lists
    pub notes: Vec<Box<dyn StandardHitObject>>,

    // hit timing bar stuff
    hitwindow_50: f32,
    hitwindow_100: f32,
    hitwindow_300: f32,
    hitwindow_miss: f32,
    end_time: f32,

    // draw_points: Vec<(f32, Vector2, ScoreHit)>,
    mouse_pos: Vector2,
    window_mouse_pos: Vector2,

    key_counter: KeyCounter,

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

    use_controller_cursor: bool
}
impl StandardGame {
    fn playfield_changed(&mut self) {
        let new_scale = Arc::new(ScalingHelper::new(self.cs, "osu".to_owned()));
        self.scaling_helper = new_scale.clone();

        // update playfield for notes
        for note in self.notes.iter_mut() {
            note.playfield_changed(new_scale.clone());
        }
    }

    // TODO: finish this
    #[allow(dead_code, unused_variables)]
    fn apply_stacking(&mut self) {
        let stack_offset = self.scaling_helper.scaled_cs / 10.0;

        let stack_vector = Vector2::one() * stack_offset;

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
                if obj.note_type() == NoteType::Spinner {break}

                let obj_n = &self.notes[n];
                if obj_n.note_type() == NoteType::Spinner {break}

                let stack_threshhold = obj_n.get_preempt() * self.stack_leniency;

                if obj_n.time() - obj.time() > stack_threshhold {
                    // outside stack threshhold
                    break;
                }

                let obj_pos = obj.pos_at(obj.time());
                let obj_n_pos = obj.pos_at(obj.time());
                let obj_is_slider = obj.note_type() == NoteType::Slider;
                let obj_end_pos = obj.pos_at(obj.end_time(0.0));

                if obj_pos.distance(obj_n_pos) < STACK_LENIENCY as f64 || (obj_is_slider && obj_end_pos.distance(obj_n_pos) < STACK_LENIENCY as f64) {
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

}

impl GameMode for StandardGame {
    fn playmode(&self) -> PlayMode {"osu".to_owned()}
    fn end_time(&self) -> f32 {self.end_time}
    fn show_cursor(&self) -> bool {true}

    fn new(map:&Beatmap, diff_calc_only: bool) -> Result<Self, crate::errors::TatakuError> {
        let metadata = map.get_beatmap_meta();
        let mods = ModManager::get().clone();
        
        let settings = get_settings!().standard_settings.clone();
        let scaling_helper = Arc::new(ScalingHelper::new(metadata.get_cs(&mods), "osu".to_owned()));
        let ar = metadata.get_ar(&mods);

        let skin_combo_colors = &SKIN_MANAGER.read().current_skin_config().combo_colors;
        let mut combo_colors = if skin_combo_colors.len() > 0 {
            skin_combo_colors.clone()
        } else {
            settings.combo_colors.iter().map(|c|Color::from_hex(c)).collect()
        };

        match map {
            Beatmap::Osu(beatmap) => {
                let stack_leniency = beatmap.stack_leniency;
                let std_settings = Arc::new(settings);

                if std_settings.use_beatmap_combo_colors && beatmap.combo_colors.len() > 0 {
                    combo_colors = beatmap.combo_colors.clone();
                }

                let mut s = Self {
                    notes: Vec::new(),
                    mouse_pos:Vector2::zero(),
                    window_mouse_pos:Vector2::zero(),
        
                    hold_count: 0,
                    // note_index: 0,
                    end_time: 0.0,
        
                    hitwindow_50: 0.0,
                    hitwindow_100: 0.0,
                    hitwindow_300: 0.0,
                    hitwindow_miss: 0.0,
        
                    move_playfield: None,
                    scaling_helper: scaling_helper.clone(),
                    cs: beatmap.metadata.get_cs(&mods),
        
                    key_counter: if diff_calc_only {KeyCounter::default()} else {
                            KeyCounter::new(
                            vec![
                                (KeyPress::Left, "L".to_owned()),
                                (KeyPress::Right, "R".to_owned()),
                                (KeyPress::LeftMouse, "M1".to_owned()),
                                (KeyPress::RightMouse, "M2".to_owned()),
                            ],
                            Vector2::zero()
                        )
                    },
                    use_controller_cursor: false,
        
                    game_settings: std_settings.clone(),
                    auto_helper: StandardAutoHelper::new(),
                    new_combos: Vec::new(),
                    stack_leniency
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
                let end_time = s.end_time as f64;
        
                let mut counter = 0;
        
                for (note, slider, spinner) in all_items {
                    // check for new combo
                    if let Some(note) = note {if note.new_combo {combo_num = 0}}
                    if let Some(slider) = slider {if slider.new_combo {combo_num = 0}}
                    if let Some(_spinner) = spinner {combo_num = 0}
        
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
                        let depth = NOTE_DEPTH.start + (note.time as f64 / end_time) * NOTE_DEPTH.end;
                        s.notes.push(Box::new(StandardNote::new(
                            note.clone(),
                            ar,
                            color,
                            combo_num as u16,
                            scaling_helper.clone(),
                            depth,
                            std_settings.clone(),
                            diff_calc_only,
                        )));
                    }
                    if let Some(slider) = slider {
                        // invisible note
                        if slider.curve_points.len() == 0 || slider.length == 0.0 {
                            let note = &NoteDef {
                                pos: slider.pos,
                                time: slider.time,
                                hitsound: slider.hitsound,
                                hitsamples: slider.hitsamples.clone(),
                                new_combo: slider.new_combo,
                                color_skip: slider.color_skip,
                            };
        
                            let depth = NOTE_DEPTH.start + (note.time as f64 / end_time) * NOTE_DEPTH.end;
                            s.notes.push(Box::new(StandardNote::new(
                                note.clone(),
                                ar,

                                Color::new(0.0, 0.0, 0.0, 1.0),
                                combo_num as u16,
                                scaling_helper.clone(),
                                depth,
                                std_settings.clone(),
                                diff_calc_only,
                            )));
                        } else {
                            let slider_depth = SLIDER_DEPTH.start + (slider.time as f64 / end_time) * SLIDER_DEPTH.end;
                            let depth = NOTE_DEPTH.start + (slider.time as f64 / end_time) * NOTE_DEPTH.end;
        
                            let curve = get_curve(slider, &map);
                            s.notes.push(Box::new(StandardSlider::new(
                                slider.clone(),
                                curve,
                                ar,
                                color,
                                combo_num as u16,
                                scaling_helper.clone(),
                                slider_depth,
                                depth,
                                std_settings.clone(),
                                diff_calc_only,
                            )))
                        }
                        
                    }
                    if let Some(spinner) = spinner {
                        s.notes.push(Box::new(StandardSpinner::new(
                            spinner.clone(),
                            scaling_helper.clone(),
                            diff_calc_only,
                        )))
                    }
                    
                    counter += 1;
                }
        
                // wait an extra sec
                s.end_time += 1000.0;
        
                Ok(s)
            },
            
            _ => Err(crate::errors::BeatmapError::UnsupportedMode.into()),
        }
    }

    fn handle_replay_frame(&mut self, frame:ReplayFrame, time:f32, manager:&mut IngameManager) {
        if !manager.replaying {
            manager.replay.frames.push((time, frame.clone()));
            manager.outgoing_spectator_frame((time, SpectatorFrameData::ReplayFrame{frame}));
        }

        const ALLOWED_PRESSES:&[KeyPress] = &[
            KeyPress::Left, 
            KeyPress::Right,
            KeyPress::LeftMouse,
            KeyPress::RightMouse,
        ];

        match frame {
            ReplayFrame::Press(key) if ALLOWED_PRESSES.contains(&key) => {
                self.key_counter.key_down(key);
                self.hold_count += 1;

                match key {
                    KeyPress::Left | KeyPress::LeftMouse => CursorManager::left_pressed(true, true),
                    KeyPress::Right | KeyPress::RightMouse => CursorManager::right_pressed(true, true),
                    _ => {}
                }

                let mut check_notes = Vec::new();
                let w = self.hitwindow_miss;
                for note in self.notes.iter_mut() {
                    note.press(time);
                    // check if note is in hitwindow
                    if (time - note.time()).abs() <= w && !note.was_hit() {
                        check_notes.push(note);
                    }
                }
                if check_notes.len() == 0 {return} // no notes to check
                check_notes.sort_by(|a, b| a.time().partial_cmp(&b.time()).unwrap());
                

                let note = &mut check_notes[0];
                let note_time = note.time();
                let pts = note.get_points(true, time, (self.hitwindow_miss, self.hitwindow_50, self.hitwindow_100, self.hitwindow_300));
                
                
                let is_300 = match pts {ScoreHit::X300 | ScoreHit::Xgeki => true, _ => false};
                if !is_300 || (is_300 && self.game_settings.show_300s) {
                    add_judgement_indicator(note.point_draw_pos(time), time, &pts, &self.scaling_helper, manager);
                }

                match &pts {
                    ScoreHit::None | ScoreHit::Other(_,_) => {}
                    ScoreHit::Miss => {
                        manager.combo_break();
                        manager.score.hit_miss(time, note_time);
                        manager.hitbar_timings.push((time, time - note_time));

                        manager.health.take_damage();
                        if manager.health.is_dead() {
                            manager.fail()
                        }
                    }

                    pts => {
                        let hitsound = note.get_hitsound();
                        let hitsamples = note.get_hitsamples().clone();
                        manager.play_note_sound(note_time, hitsound, hitsamples);

                        match pts {
                            ScoreHit::X50 => manager.score.hit50(time, note_time),
                            ScoreHit::X100 | ScoreHit::Xkatu => manager.score.hit100(time, note_time),
                            ScoreHit::X300 | ScoreHit::Xgeki => manager.score.hit300(time, note_time),
                            _ => {}
                        }

                        manager.health.give_life();

                        manager.hitbar_timings.push((time, time - note_time));
                    }
                }
            }
            // dont continue if no keys were being held (happens when leaving a menu)
            ReplayFrame::Release(key) if ALLOWED_PRESSES.contains(&key) && self.hold_count > 0 => {
                self.key_counter.key_up(key);
                self.hold_count -= 1;

                match key {
                    KeyPress::Left | KeyPress::LeftMouse => CursorManager::left_pressed(false, true),
                    KeyPress::Right | KeyPress::RightMouse => CursorManager::right_pressed(false, true),
                    _ => {}
                }

                let mut check_notes = Vec::new();
                let w = self.hitwindow_miss;
                for note in self.notes.iter_mut() {

                    // if this is the last key to be released
                    if self.hold_count == 0 {
                        note.release(time)
                    }

                    // check if note is in hitwindow
                    if (time - note.time()).abs() <= w && !note.was_hit() {
                        check_notes.push(note);
                    }
                }
            }
            ReplayFrame::MousePos(x, y) => {
                // scale the coords from playfield to window
                let pos = self.scaling_helper.scale_coords(Vector2::new(x as f64, y as f64));
                self.mouse_pos = pos;

                for note in self.notes.iter_mut() {
                    note.mouse_move(pos);
                }
            }
            _ => {}
        }
    }


    fn update(&mut self, manager:&mut IngameManager, time:f32) {

        // do autoplay things
        if manager.current_mods.autoplay {
            let mut pending_frames = Vec::new();

            self.auto_helper.update(time, &mut self.notes, &self.scaling_helper, &mut pending_frames);

            // handle presses and mouse movements now, and releases later
            for frame in pending_frames.iter() {
                self.handle_replay_frame(*frame, time, manager);
            }
        }


        // if the map is over, say it is
        if time >= self.end_time {
            manager.completed = true;
            return;
        }

        // update notes
        for note in self.notes.iter_mut() {
            note.update(time);

            // play queued sounds
            for (time, hitsound, samples) in note.get_sound_queue() {
                manager.play_note_sound(time, hitsound, samples);
            }

            for add_combo in note.pending_combo() {
                if add_combo < 0 {
                    manager.combo_break();
                    manager.health.take_damage();
                } else if add_combo > 0 {
                    for _ in 0..add_combo {
                        manager.score.hit300(0.0, 0.0);
                        manager.health.give_life();
                    }
                }
            }

            // check if note was missed
            
            // if the time is leading in, we dont want to check if any notes have been missed
            if time < 0.0 {continue}

            let end_time = note.end_time(self.hitwindow_miss);

            // check if note is in hitwindow
            if time - end_time >= 0.0 && !note.was_hit() {

                // check if we missed the current note
                match note.note_type() {
                    NoteType::Note if end_time < time => {
                        trace!("note missed: {}-{}", time, end_time);
                        manager.combo_break();
                        manager.score.hit_miss(time, end_time);
                        add_judgement_indicator(note.point_draw_pos(time), time, &ScoreHit::Miss, &self.scaling_helper, manager);

                        manager.health.take_damage();
                        if manager.health.is_dead() {
                            manager.fail()
                        }
                    }
                    NoteType::Slider if end_time <= time => {
                        let note_time = note.end_time(0.0);
                        // check slider release points
                        // -1.0 for miss hitwindow to indidate it was held to the end (ie, no hitwindow to check)
                        let pts = note.get_points(false, time, (-1.0, self.hitwindow_50, self.hitwindow_100, self.hitwindow_300));
                        
                        let is_300 = match pts {ScoreHit::X300 | ScoreHit::Xgeki => true, _ => false};
                        if !is_300 || (is_300 && self.game_settings.show_300s) {
                            add_judgement_indicator(note.point_draw_pos(time), time, &pts, &self.scaling_helper, manager);
                        }
                        
                        match pts {
                            ScoreHit::Other(_, _) => {}
                            ScoreHit::None | ScoreHit::Miss => {
                                manager.combo_break();
                                manager.score.hit_miss(time, note_time);
                                manager.hitbar_timings.push((time, time - note_time));
                                
                                manager.health.take_damage();
                                if manager.health.is_dead() {
                                    manager.fail()
                                }
                            }
                            pts => {
                                match pts {
                                    ScoreHit::X300 | ScoreHit::Xgeki => manager.score.hit300(time, note_time),
                                    ScoreHit::X100 | ScoreHit::Xkatu => {
                                        manager.score.hit100(time, note_time);
                                        // manager.combo_break();
                                    },
                                    ScoreHit::X50 => manager.score.hit50(time, note_time),
                                    _ => {}
                                }

                                // play hitsound
                                let hitsound = note.get_hitsound();
                                let hitsamples = note.get_hitsamples().clone();
                                manager.play_note_sound(note_time, hitsound, hitsamples);
                                manager.hitbar_timings.push((time, time - note_time));

                                manager.health.give_life();
                            }
                        }
                    }

                    NoteType::Spinner if end_time <= time => {}

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
        if manager.current_mods.autoplay {
            for frame in self.auto_helper.get_release_queue().iter() {
                self.handle_replay_frame(*frame, time, manager);
            }
        }

    }
    fn draw(&mut self, args:RenderArgs, manager:&mut IngameManager, list:&mut Vec<Box<dyn Renderable>>) {

        // draw the playfield
        if !manager.menu_background {
            let mut playfield = self.scaling_helper.playfield_scaled_with_cs_border.clone();

            if self.move_playfield.is_some() {
                let line_size = self.game_settings.playfield_movelines_thickness;
                // draw x and y center lines
                let px_line = Line::new(
                    playfield.current_pos + Vector2::new(0.0, playfield.size.y/2.0),
                    playfield.current_pos + Vector2::new(playfield.size.x, playfield.size.y/2.0),
                    line_size,
                    -100.0,
                    Color::WHITE
                );
                let py_line = Line::new(
                    playfield.current_pos + Vector2::new(playfield.size.x/2.0, 0.0),
                    playfield.current_pos + Vector2::new(playfield.size.x/2.0, playfield.size.y),
                    line_size, 
                    -100.0,
                    Color::WHITE
                );

                let window_size = Settings::window_size();
                let wx_line = Line::new(
                    Vector2::new(0.0, window_size.y/2.0),
                    Vector2::new(window_size.x, window_size.y/2.0),
                    line_size,
                    -100.0,
                    Color::WHITE
                );
                let wy_line = Line::new(
                    Vector2::new(window_size.x/2.0, 0.0),
                    Vector2::new(window_size.x/2.0, window_size.y),
                    line_size, 
                    -100.0,
                    Color::WHITE
                );

                playfield.border = Some(Border::new(Color::WHITE, line_size));

                list.push(Box::new(wx_line));
                list.push(Box::new(wy_line));
                list.push(Box::new(px_line));
                list.push(Box::new(py_line));
            }

            if manager.current_timing_point().kiai {
                playfield.border = Some(Border::new(Color::YELLOW, 2.0));
            }
            list.push(Box::new(playfield));


            // draw key counter
            self.key_counter.draw(args, list);
        }


        // if this is a replay, we need to draw the replay curser
        if manager.replaying || manager.current_mods.autoplay || self.use_controller_cursor {
            CursorManager::set_pos(self.mouse_pos, true)
        }

        // draw notes
        for note in self.notes.iter_mut() {
            note.draw(args, list);
        }

        // draw follow points
        let time = manager.time();
        if self.game_settings.draw_follow_points {
            for i in 0..self.notes.len() - 1 {
                if !self.new_combos.contains(&(i + 1)) {
                    let n1 = &self.notes[i];
                    let n2 = &self.notes[i + 1];

                    if n1.note_type() == NoteType::Spinner {continue}
                    if n2.note_type() == NoteType::Spinner {continue}

                    let preempt = n2.get_preempt();
                    let n1_time = n1.time();
                    if time < n1_time - preempt {continue} //|| time > n2.end_time(0.0) {continue}
                    let n2_time = n2.time();
                    if time >= n2_time {continue}//|| time <= n1_time {continue}

                    // let bar_size = Vector2::new(7.0, 3.0) * self.scaling_helper.scale;
                    let follow_dot_size = 3.0 * self.scaling_helper.scale;
                    let follow_dot_distance = 20.0 * self.scaling_helper.scale;

                    // setup follow points and the time they should exist at

                    let n1_pos = n1.pos_at(n2_time);
                    let n2_pos = n2.pos_at(n2_time);

                    let distance = n1_pos.distance(n2_pos);
                    let follow_dot_count = distance/follow_dot_distance;
                    for i in 0..follow_dot_count as u64 {
                        let lerp_amount = i as f64 / follow_dot_count as f64;
                        let time_at_this_point = f64::lerp(n1_time as f64, n2_time as f64, lerp_amount) as f32;
                        let point = Vector2::lerp(n1_pos, n2_pos, lerp_amount);
                        
                        let alpha_lerp_amount = (time_at_this_point - time) / (n2_time - n1_time);
                        let mut alpha = 1.0 - f64::easeinout_sine(0.0, 1.0, alpha_lerp_amount as f64) as f32;
                        if time < n1_time {
                            // TODO!
                            alpha = 0.1;
                        }
                        if alpha == 0.0 {continue}

                        list.push(Box::new(Circle::new(
                            Color::WHITE.alpha(alpha),
                            100_000.0,
                            point,
                            follow_dot_size,
                            None
                        )));
                    }
                }
            }
        }
    }

    
    fn key_down(&mut self, key:piston::Key, manager:&mut IngameManager) {
        if key == piston::Key::LCtrl {
            let old = get_settings!().standard_settings.get_playfield();
            self.move_playfield = Some((old.1, self.window_mouse_pos));
            return;
        }
        
        // dont accept key input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.autoplay || manager.replaying {
            return;
        }

        let time = manager.time();
        if key == self.game_settings.left_key {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Left), time, manager);
        }
        if key == self.game_settings.right_key {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Right), time, manager);
        }
    }
    fn key_up(&mut self, key:piston::Key, manager:&mut IngameManager) {
        if key == piston::Key::LCtrl {
            self.move_playfield = None;
            return;
        }

        
        // dont accept key input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.autoplay || manager.replaying {
            return;
        }

        let time = manager.time();
        if key == self.game_settings.left_key {
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::Left), time, manager);
        }
        if key == self.game_settings.right_key {
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::Right), time, manager);
        }
    }
    

    fn mouse_move(&mut self, pos:Vector2, manager:&mut IngameManager) {
        self.window_mouse_pos = pos;
        
        if let Some((original, mouse_start)) = self.move_playfield {
            {
                let settings = &mut get_settings_mut!().standard_settings;
                let mut change = original + (pos - mouse_start);

                // check playfield snapping
                // TODO: can this be simplified?
                let window_size = Settings::window_size();
                let playfield_size = self.scaling_helper.playfield_scaled_with_cs_border.size;

                // what the offset should be if playfield is centered
                let center_offset = (window_size - FIELD_SIZE * self.scaling_helper.scale) / 2.0 - (window_size - playfield_size) / 2.0;

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

            self.playfield_changed();
            return;
        }

        // dont accept mouse input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.autoplay || manager.replaying {
            return;
        }

        // convert window pos to playfield pos
        let time = manager.time();
        let pos = self.scaling_helper.descale_coords(pos);
        self.handle_replay_frame(ReplayFrame::MousePos(pos.x as f32, pos.y as f32), time, manager);
    }
    fn mouse_down(&mut self, btn:piston::MouseButton, manager:&mut IngameManager) {
        if self.game_settings.ignore_mouse_buttons {return}
        
        // dont accept mouse input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.autoplay || manager.replaying {
            return;
        }

        let time = manager.time();
        if btn == MouseButton::Left {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftMouse), time, manager);
        }
        if btn == MouseButton::Right {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::RightMouse), time, manager);
        }
    }
    fn mouse_up(&mut self, btn:piston::MouseButton, manager:&mut IngameManager) {
        if self.game_settings.ignore_mouse_buttons {return}

        // dont accept mouse input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.autoplay || manager.replaying {
            return;
        }

        let time = manager.time();
        if btn == MouseButton::Left {
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::LeftMouse), time, manager);
        }
        if btn == MouseButton::Right {
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::RightMouse), time, manager);
        }
    }

    fn mouse_scroll(&mut self, delta:f64, _manager:&mut IngameManager) {
        if self.move_playfield.is_some() {
            {
                let settings = &mut get_settings_mut!().standard_settings;
                settings.playfield_scale += delta / 40.0;
            }

            self.playfield_changed();
        }
    }



    fn controller_press(&mut self, c: &Box<dyn Controller>, btn: u8, manager:&mut IngameManager) {
        // dont accept controller input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.autoplay || manager.replaying {
            return;
        }

        if Some(ControllerButton::Left_Bumper) == c.map_button(btn) {
            let time = manager.time();
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Left), time, manager);
        }

        if Some(ControllerButton::Right_Bumper) == c.map_button(btn) {
            let time = manager.time();
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Right), time, manager);
        }
    }
    fn controller_release(&mut self, c: &Box<dyn Controller>, btn: u8, manager:&mut IngameManager) {
        // dont accept controller input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.autoplay || manager.replaying {
            return;
        }

        if Some(ControllerButton::Left_Bumper) == c.map_button(btn) {
            let time = manager.time();
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::Left), time, manager);
        }

        if Some(ControllerButton::Right_Bumper) == c.map_button(btn) {
            let time = manager.time();
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::Right), time, manager);
        }
    }
    fn controller_axis(&mut self, c: &Box<dyn Controller>, axis_data:HashMap<u8, (bool, f64)>, manager:&mut IngameManager) {
        // dont accept controller input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.autoplay || manager.replaying {
            return;
        }

        self.use_controller_cursor = true;

        let mut new_pos = self.mouse_pos;
        let scaling_helper = self.scaling_helper.clone();
        let playfield = scaling_helper.playfield_scaled_with_cs_border;

        for (axis, &(_new, value)) in axis_data.iter() {
            match c.map_axis(*axis) {
                Some(ControllerAxis::Left_X) => {
                    // -1.0 to 1.0
                    // where -1 is 0, and 1 is scaling_helper.playfield_scaled_with_cs_border.whatever
                    let normalized = (value + 1.0) / 2.0;
                    new_pos.x = playfield.current_pos.x + f64::lerp(0.0, playfield.size.x, normalized);
                },
                Some(ControllerAxis::Left_Y) => {
                    
                    let normalized = (value + 1.0) / 2.0;
                    new_pos.y = playfield.current_pos.y + f64::lerp(0.0, playfield.size.y, normalized);
                },
                _ => {},
            }
        }

        let time = manager.time();
        let new_pos = scaling_helper.descale_coords(new_pos);
        self.handle_replay_frame(ReplayFrame::MousePos(new_pos.x as f32, new_pos.y as f32), time, manager);
    }


    fn reset(&mut self, beatmap:&Beatmap) {
        
        // setup hitwindows
        let od = beatmap.get_beatmap_meta().get_od(& ModManager::get());
        self.hitwindow_miss = map_difficulty(od, 225.0, 175.0, 125.0); // idk
        self.hitwindow_50   = map_difficulty(od, 200.0, 150.0, 100.0);
        self.hitwindow_100  = map_difficulty(od, 140.0, 100.0, 60.0);
        self.hitwindow_300  = map_difficulty(od, 80.0, 50.0, 20.0);

        // reset notes
        let hwm = self.hitwindow_miss;
        for note in self.notes.iter_mut() {
            note.reset();
            note.set_hitwindow_miss(hwm);
        }

        self.key_counter.reset();
    }

    fn skip_intro(&mut self, manager: &mut IngameManager) {
        if self.notes.len() == 0 {return}

        let time = self.notes[0].time() - self.notes[0].get_preempt();
        if time < manager.time() {return}

        if time < 0.0 {return}
        #[cfg(feature="bass_audio")]
        manager.song.set_position(time as f64).unwrap();
        #[cfg(feature="neb_audio")]
        manager.song.upgrade().unwrap().set_position(time);
    }

    fn timing_bar_things(&self) -> (Vec<(f32,Color)>, (f32,Color)) {
        (vec![
            (self.hitwindow_50, [0.8549, 0.6823, 0.2745, 1.0].into()),
            (self.hitwindow_100, [0.3411, 0.8901, 0.0745, 1.0].into()),
            (self.hitwindow_300, [0.0, 0.7647, 1.0, 1.0].into()),
        ], (self.hitwindow_miss, [0.9, 0.05, 0.05, 1.0].into()))
    }

    fn combo_bounds(&self) -> Rectangle {
        let size = Vector2::new(100.0, 30.0);
        Rectangle::bounds_only(
            Vector2::new(0.0, Settings::window_size().y - (size.y + DURATION_HEIGHT + 10.0)),
            size
        )
    }

    
    fn apply_auto(&mut self, _settings: &crate::game::BackgroundGameSettings) {
        // for note in self.notes.iter_mut() {
        //     note.set_alpha(settings.opacity)
        // }
    }


    fn score_hit_string(hit:&ScoreHit) -> String where Self: Sized {
        match hit {
            ScoreHit::Miss  => "Miss".to_owned(),
            ScoreHit::X50   => "x50".to_owned(),
            ScoreHit::X100  => "x100".to_owned(),
            ScoreHit::X300  => "x300".to_owned(),
            ScoreHit::Xgeki => "Geki".to_owned(),
            ScoreHit::Xkatu => "Katu".to_owned(),

            ScoreHit::None  => String::new(),
            ScoreHit::Other(_, _) => String::new(),
        }
    }
}



fn add_judgement_indicator(pos: Vector2, time: f32, hit_value: &ScoreHit, scaling_helper: &Arc<ScalingHelper>, manager: &mut IngameManager) {
    let (color, image) = match hit_value {
        ScoreHit::Miss => (Color::RED, None),
        ScoreHit::X50  => (Color::YELLOW, None),
        ScoreHit::X100 | ScoreHit::Xkatu => (Color::GREEN, None),
        ScoreHit::X300 | ScoreHit::Xgeki => (Color::new(0.0, 0.7647, 1.0, 1.0), None),
        ScoreHit::None | ScoreHit::Other(_, _) => return,
    };

    manager.add_judgement_indicator(BasicJudgementIndicator::new(
        pos, 
        time,
        -99999.99, // TODO: do this properly
        CIRCLE_RADIUS_BASE * scaling_helper.scaled_cs * (1.0/3.0),
        color,
        image
    ))
}


// impl Drop for StandardGame {
//     fn drop(&mut self) {
//         trace!("dropping std")
//     }
// }
struct StandardAutoHelper {
    // point_trail_angle: Vector2,
    point_trail_start_time: f32,
    point_trail_end_time: f32,
    point_trail_start_pos: Vector2,
    point_trail_end_pos: Vector2,

    /// list of notes currently being held
    holding: Vec<usize>,

    release_queue:Vec<ReplayFrame>
}
impl StandardAutoHelper {
    fn new() -> Self {
        Self {
            // point_trail_angle: Vector2::zero(),
            point_trail_start_time: 0.0,
            point_trail_end_time: 0.0,
            point_trail_start_pos: Vector2::zero(),
            point_trail_end_pos: Vector2::zero(),

            holding: Vec::new(),

            release_queue: Vec::new()
        }
    }
    fn get_release_queue(&mut self) -> Vec<ReplayFrame> {
        std::mem::take(&mut self.release_queue)
    }

    fn update(&mut self, time:f32, notes: &mut Vec<Box<dyn StandardHitObject>>, scaling_helper: &Arc<ScalingHelper>, frames: &mut Vec<ReplayFrame>) {
        let mut any_checked = false;

        for i in 0..notes.len() {
            let note = &notes[i];
            if note.was_hit() {continue}

            if let Ok(ind) = self.holding.binary_search(&i) {
                if time >= note.end_time(0.0) {
                    self.release_queue.push(ReplayFrame::Release(KeyPress::LeftMouse));

                    let pos = scaling_helper.descale_coords(note.pos_at(time));

                    self.holding.remove(ind);
                    if i+1 >= notes.len() {
                        self.point_trail_start_pos = pos;
                        self.point_trail_end_pos = pos;
                        continue;
                    }
                    
                    let next_note = &notes[i + 1];

                    self.point_trail_start_pos = pos;
                    self.point_trail_end_pos = scaling_helper.descale_coords(next_note.pos_at(self.point_trail_end_time));
                    
                    self.point_trail_start_time = time;
                    self.point_trail_end_time = next_note.time();
                } else {
                    let pos = scaling_helper.descale_coords(note.pos_at(time));
                    // move the mouse to the pos
                    frames.push(ReplayFrame::MousePos(
                        pos.x as f32,
                        pos.y as f32
                    ));
                }
                
                any_checked = true;
                continue;
            }
            
            if time >= note.time() {
                let pos = scaling_helper.descale_coords(note.pos_at(time));
                // move the mouse to the pos
                frames.push(ReplayFrame::MousePos(
                    pos.x as f32,
                    pos.y as f32
                ));
                
                frames.push(ReplayFrame::Press(KeyPress::LeftMouse));
                if note.note_type() == NoteType::Note {
                    self.release_queue.push(ReplayFrame::Release(KeyPress::LeftMouse));
                } else {
                    self.holding.push(i)
                }

                // if this was the last note
                if i + 1 >= notes.len() {
                    self.point_trail_start_pos = pos;
                    self.point_trail_end_pos = scaling_helper.descale_coords(scaling_helper.window_size / 2.0);
                    
                    self.point_trail_start_time = 0.0;
                    self.point_trail_end_time = 1.0;
                    continue;
                }

                // draw a line to the next note
                let next_note = &notes[i + 1];

                self.point_trail_start_pos = pos;
                self.point_trail_end_pos = scaling_helper.descale_coords(next_note.pos_at(self.point_trail_end_time));
                
                self.point_trail_start_time = time;
                self.point_trail_end_time = next_note.time();

                any_checked = true;
            }
        }
        if any_checked {return}

        // if we got here no notes were updated
        // follow the point_trail
        let duration = self.point_trail_end_time - self.point_trail_start_time;
        let current = time - self.point_trail_start_time;
        let len = current / duration;
        
        let new_pos = Vector2::lerp(self.point_trail_start_pos, self.point_trail_end_pos, len as f64);
        frames.push(ReplayFrame::MousePos(
            new_pos.x as f32,
            new_pos.y as f32
        ));
    }
}
