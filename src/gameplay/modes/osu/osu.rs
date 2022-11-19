use std::ops::Range;

use crate::prelude::*;
use super::osu_notes::*;
use super::OsuHitJudgments;


const NOTE_DEPTH:Range<f64> = 100.0..200.0;
pub const SLIDER_DEPTH:Range<f64> = 200.0..300.0;

const STACK_LENIENCY:u32 = 3;


/// calculate the standard acc for `score`
pub fn calc_acc(score: &Score) -> f64 {
    let x50  = score.judgments.get("x50").copy_or_default()  as f64;
    let x100 = score.judgments.get("x100").copy_or_default() as f64;
    let x300 = score.judgments.get("x300").copy_or_default() as f64;
    let geki = score.judgments.get("geki").copy_or_default() as f64;
    let katu = score.judgments.get("katu").copy_or_default() as f64;
    let miss = score.judgments.get("xmiss").copy_or_default() as f64;

    (50.0 * x50 + 100.0 * (x100 + katu) + 300.0 * (x300 + geki)) 
    / (300.0 * (miss + x50 + x100 + x300 + katu + geki))
}

pub struct StandardGame {
    // lists
    pub notes: Vec<Box<dyn StandardHitObject>>,

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

    slider_dot_image: Option<Image>,
    
    judgment_helper: JudgmentImageHelper,
}
impl StandardGame {
    async fn playfield_changed(&mut self) {
        let new_scale = Arc::new(ScalingHelper::new(self.cs, self.window_size.0).await);
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

#[async_trait]
impl GameMode for StandardGame {
    async fn new(map:&Beatmap, diff_calc_only: bool) -> Result<Self, crate::errors::TatakuError> {
        let metadata = map.get_beatmap_meta();
        let mods = ModManager::get().await.clone();
        let window_size = WindowSize::get();
        let effective_window_size = if diff_calc_only { Vector2::new(1280.0, 720.0) } else { window_size.0 };
        
        let settings = get_settings!().standard_settings.clone();
        let scaling_helper = Arc::new(ScalingHelper::new(metadata.get_cs(&mods), effective_window_size).await);
        let ar = metadata.get_ar(&mods);

        // TODO: beatmap combo colors
        let skin_combo_colors = &SkinManager::current_skin_config().await.combo_colors;
        let mut combo_colors = if skin_combo_colors.len() > 0 {
            skin_combo_colors.clone()
        } else {
            settings.combo_colors.iter().map(|c|Color::from_hex(c)).collect()
        };

        
        // windows
        let od = map.get_beatmap_meta().get_od(&*ModManager::get().await);
        let w_miss = map_difficulty(od, 225.0, 175.0, 125.0); // idk
        let w_50   = map_difficulty(od, 200.0, 150.0, 100.0);
        let w_100  = map_difficulty(od, 140.0, 100.0, 60.0);
        let w_300  = map_difficulty(od, 80.0, 50.0, 20.0);

        let miss_window = w_miss;
        let hit_windows = vec![
            (OsuHitJudgments::X300, 0.0..w_300),
            (OsuHitJudgments::X100, w_300..w_100),
            (OsuHitJudgments::X50, w_100..w_50),
            (OsuHitJudgments::Miss, w_50..w_miss),
        ];

        let judgment_helper = JudgmentImageHelper::new(OsuHitJudgments::Miss).await;
        let slider_dot_image = SkinManager::get_texture("followpoint", true).await;

        let mut s = match map {
            Beatmap::Osu(beatmap) => {
                let stack_leniency = beatmap.stack_leniency;
                let std_settings = Arc::new(settings);

                if std_settings.use_beatmap_combo_colors && beatmap.combo_colors.len() > 0 {
                    combo_colors = beatmap.combo_colors.clone();
                }

                let mut s = Self {
                    notes: Vec::new(),
                    mouse_pos: Vector2::zero(),
                    window_mouse_pos: Vector2::zero(),
                    hit_windows,
                    miss_window,
        
                    hold_count: 0,
                    end_time: 0.0,
        
                    move_playfield: None,
                    scaling_helper: scaling_helper.clone(),
                    cs: beatmap.metadata.get_cs(&mods),

                    use_controller_cursor: false,
        
                    game_settings: std_settings.clone(),
                    auto_helper: StandardAutoHelper::new(),
                    new_combos: Vec::new(),
                    stack_leniency,
                    window_size,
                    slider_dot_image,
                    judgment_helper,
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
        
                            let depth = NOTE_DEPTH.start + (note.time as f64 / end_time) * NOTE_DEPTH.end;
                            s.notes.push(Box::new(StandardNote::new(
                                note,
                                ar,

                                Color::new(0.0, 0.0, 0.0, 1.0),
                                combo_num as u16,
                                scaling_helper.clone(),
                                depth,
                                std_settings.clone(),
                                diff_calc_only,
                            ).await));
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
                            ).await))
                        }
                        
                    }
                    if let Some(spinner) = spinner {
                        s.notes.push(Box::new(StandardSpinner::new(
                            spinner.clone(),
                            scaling_helper.clone(),
                            diff_calc_only,
                        ).await))
                    }
                    
                    counter += 1;
                }
        
                s
            }
            
            _ => return Err(crate::errors::BeatmapError::UnsupportedMode.into()),
        };

        // wait an extra sec
        s.end_time += 1000.0;

        if !diff_calc_only {
            for n in s.notes.iter_mut() {
                n.reload_skin().await;
            }
        }

        Ok(s)
    }

    async fn handle_replay_frame(&mut self, frame:ReplayFrame, time:f32, manager:&mut IngameManager) {
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
                manager.key_counter.key_down(key);
                self.hold_count += 1;

                match key {
                    KeyPress::Left | KeyPress::LeftMouse => CursorManager::left_pressed(true, true),
                    KeyPress::Right | KeyPress::RightMouse => CursorManager::right_pressed(true, true),
                    _ => {}
                }

                let mut check_notes = Vec::new();
                for note in self.notes.iter_mut() {
                    note.press(time);
                    // check if note is in hitwindow
                    if (time - note.time()).abs() <= self.miss_window && !note.was_hit() {
                        check_notes.push(note);
                    }
                }
                if check_notes.len() == 0 { return } // no notes to check
                check_notes.sort_by(|a, b| a.time().partial_cmp(&b.time()).unwrap());
                

                let note = &mut check_notes[0];
                let note_time = note.time();

                // check distance
                if note.check_distance(self.mouse_pos) {
                    if let Some(judge) = manager.check_judgment(&self.hit_windows, time, note_time).await {
                        note.set_judgment(judge);

                        if judge == &OsuHitJudgments::X300 && !self.game_settings.show_300s {
                            // dont show the judgment
                        } else {
                            add_judgement_indicator(note.point_draw_pos(time), time, judge, &self.scaling_helper, &self.judgment_helper, manager);
                        }

                        if let OsuHitJudgments::Miss = judge {
                            // tell the note it was missed
                            note.miss();
                        } else {
                            // tell the note it was hit
                            note.hit(time);

                            // play the sound
                            let hitsound = note.get_hitsound();
                            let hitsamples = note.get_hitsamples().clone();
                            manager.play_note_sound(note_time, hitsound, hitsamples, true).await;
                        }

                    }
                }


                // let pts = note.get_points(true, time, (self.hitwindow_miss, self.hitwindow_50, self.hitwindow_100, self.hitwindow_300));
                
                
                // let is_300 = match pts {ScoreHit::X300 | ScoreHit::Xgeki => true, _ => false};
                // if !is_300 || (is_300 && self.game_settings.show_300s) {
                // }

                // match &pts {
                //     ScoreHit::None | ScoreHit::Other(_,_) => {}
                //     ScoreHit::Miss => {
                //         manager.combo_break().await;
                //         manager.score.hit_miss(time, note_time);
                //         manager.hitbar_timings.push((time, time - note_time));

                //         manager.health.take_damage();
                //         if manager.health.is_dead() {
                //             manager.fail()
                //         }
                //     }

                //     pts => {

                //         match pts {
                //             ScoreHit::X50 => manager.score.hit50(time, note_time),
                //             ScoreHit::X100 | ScoreHit::Xkatu => manager.score.hit100(time, note_time),
                //             ScoreHit::X300 | ScoreHit::Xgeki => manager.score.hit300(time, note_time),
                //             _ => {}
                //         }

                //         manager.health.give_life();

                //         manager.hitbar_timings.push((time, time - note_time));
                //     }
                // }
            }
            // dont continue if no keys were being held (happens when leaving a menu)
            ReplayFrame::Release(key) if ALLOWED_PRESSES.contains(&key) && self.hold_count > 0 => {
                manager.key_counter.key_up(key);
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
                    if time >= note.end_time(self.miss_window) && !note.was_hit() && note.note_type() != NoteType::Note {
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


    async fn update(&mut self, manager:&mut IngameManager, time:f32) {
        // do autoplay things
        if manager.current_mods.has_autoplay() {
            let mut pending_frames = Vec::new();

            self.auto_helper.update(time, &mut self.notes, &self.scaling_helper, &mut pending_frames);

            // handle presses and mouse movements now, and releases later
            for frame in pending_frames.iter() {
                self.handle_replay_frame(*frame, time, manager).await;
            }
        }


        // if the map is over, say it is
        if time >= self.end_time {
            manager.completed = true;
            return;
        }

        // update notes
        for note in self.notes.iter_mut() {
            note.update(time).await;

            // play queued sounds
            for (time, hitsound, samples) in note.get_sound_queue() {
                manager.play_note_sound(time, hitsound, samples, true).await;
            }

            for (add_combo, pos) in note.pending_combo() {
                manager.add_judgment(&add_combo).await;
                add_judgement_indicator(pos, time, &add_combo, &self.scaling_helper, &self.judgment_helper, manager);
            }

            // check if note was missed
            
            // if the time is leading in, we dont want to check if any notes have been missed
            if time < 0.0 { continue }

            let end_time = note.end_time(self.miss_window);

            // check if note is in hitwindow
            if time >= end_time && !note.was_hit() {

                // check if we missed the current note
                match note.note_type() {
                    NoteType::Note => {
                        trace!("note missed: {}-{}", time, end_time);

                        let j = OsuHitJudgments::Miss;
                        manager.add_judgment(&j).await;

                        add_judgement_indicator(note.point_draw_pos(time), time, &j, &self.scaling_helper, &self.judgment_helper, manager);
                    }
                    NoteType::Slider => {
                        let note_time = note.end_time(0.0);

                        // check slider release points
                        // internally checks distance
                        let judge = &note.check_release_points(time);

                        // info!("slider end check: {:?}", judge);
                        manager.add_judgment(judge).await;
                        
                        if judge == &OsuHitJudgments::X300 && !self.game_settings.show_300s {
                            // dont show the judgment
                        } else {
                            add_judgement_indicator(note.point_draw_pos(time), time, judge, &self.scaling_helper, &self.judgment_helper, manager);
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
                            let hitsamples = note.get_hitsamples().clone();
                            manager.play_note_sound(note_time, hitsound, hitsamples, true).await;
                        }
                    }

                    NoteType::Spinner => {}

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
        if manager.current_mods.has_autoplay() {
            for frame in self.auto_helper.get_release_queue().iter() {
                self.handle_replay_frame(*frame, time, manager).await;
            }
        }

    }
    async fn draw(&mut self, args:RenderArgs, manager:&mut IngameManager, list:&mut Vec<Box<dyn Renderable>>) {

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

                list.push(Box::new(wx_line));
                list.push(Box::new(wy_line));
                list.push(Box::new(px_line));
                list.push(Box::new(py_line));
            }

            if manager.current_timing_point().kiai {
                playfield.border = Some(Border::new(Color::YELLOW, 2.0));
            }
            list.push(Box::new(playfield));
        }


        // if this is a replay, we need to draw the replay curser
        if manager.replaying || manager.current_mods.has_autoplay() || self.use_controller_cursor {
            CursorManager::set_pos(self.mouse_pos, true)
        }

        // draw notes
        for note in self.notes.iter_mut() {
            list.extend(note.draw(args).await);
        }

        // draw follow points
        let time = manager.time();
        if self.game_settings.draw_follow_points {
            if self.notes.len() == 0 { return }
            for i in 0..self.notes.len() - 1 {
                if !self.new_combos.contains(&(i + 1)) {
                    let n1 = &self.notes[i];
                    let n2 = &self.notes[i + 1];

                    if n1.note_type() == NoteType::Spinner { continue }
                    if n2.note_type() == NoteType::Spinner { continue }

                    let preempt = n2.get_preempt();
                    let n1_time = n1.time();
                    if time < n1_time - preempt { continue } //|| time > n2.end_time(0.0) {continue}
                    let n2_time = n2.time();
                    if time >= n2_time { continue }//|| time <= n1_time {continue}

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
                        
                        // get the alpha
                        let alpha_lerp_amount = (time_at_this_point - time) / (n2_time - n1_time);
                        let mut alpha = 1.0 - f64::easeinout_sine(0.0, 1.0, alpha_lerp_amount as f64) as f32;
                        if time < n1_time {
                            // TODO!
                            alpha = 0.1;
                        }
                        if alpha == 0.0 { continue }

                        // add point
                        if let Some(mut i) = self.slider_dot_image.clone() {
                            const FOLLOW_DOT_TEX_SIZE: Vector2 = Vector2::new(128.0, 128.0);
                            i.current_pos = point;
                            i.current_scale = Vector2::one() * (follow_dot_size * 2.0) / FOLLOW_DOT_TEX_SIZE;
                            list.push(Box::new(i));
                        } else {
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
    }

    
    async fn reset(&mut self, _beatmap:&Beatmap) {
        // reset notes
        let hwm = self.miss_window;
        for note in self.notes.iter_mut() {
            note.reset().await;
            note.set_hitwindow_miss(hwm);
        }
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

    fn apply_auto(&mut self, _settings: &crate::game::BackgroundGameSettings) {
        // for note in self.notes.iter_mut() {
        //     note.set_alpha(settings.opacity)
        // }
    }


    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.window_size = window_size;
        self.playfield_changed().await;
    }


    async fn fit_to_area(&mut self, pos: Vector2, size: Vector2) {
        let pos = pos;
        let size = size;
        let playfield = ScalingHelper::new_offset_scale(self.cs, size, pos, 0.5);
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
}

#[async_trait]
impl GameModeInput for StandardGame {

    async fn key_down(&mut self, key:piston::Key, manager:&mut IngameManager) {
        if key == piston::Key::LCtrl {
            let old = get_settings!().standard_settings.get_playfield();
            self.move_playfield = Some((old.1, self.window_mouse_pos));
            return;
        }
        
        // dont accept key input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.has_autoplay() || manager.replaying {
            return;
        }

        let time = manager.time();
        if key == self.game_settings.left_key {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Left), time, manager).await;
        }
        if key == self.game_settings.right_key {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Right), time, manager).await;
        }
    }
    
    async fn key_up(&mut self, key:piston::Key, manager:&mut IngameManager) {
        if key == piston::Key::LCtrl {
            self.move_playfield = None;
            return;
        }

        
        // dont accept key input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.has_autoplay() || manager.replaying {
            return;
        }

        let time = manager.time();
        if key == self.game_settings.left_key {
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::Left), time, manager).await;
        }
        if key == self.game_settings.right_key {
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::Right), time, manager).await;
        }
    }
    

    async fn mouse_move(&mut self, pos:Vector2, manager:&mut IngameManager) {
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
            return;
        }

        // dont accept mouse input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.has_autoplay() || manager.replaying {
            return;
        }

        // convert window pos to playfield pos
        let time = manager.time();
        let pos = self.scaling_helper.descale_coords(pos);
        self.handle_replay_frame(ReplayFrame::MousePos(pos.x as f32, pos.y as f32), time, manager).await;
    }
    
    async fn mouse_down(&mut self, btn:piston::MouseButton, manager:&mut IngameManager) {
        if self.game_settings.ignore_mouse_buttons {return}
        
        // dont accept mouse input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.has_autoplay() || manager.replaying {
            return;
        }

        let time = manager.time();
        if btn == MouseButton::Left {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftMouse), time, manager).await;
        }
        if btn == MouseButton::Right {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::RightMouse), time, manager).await;
        }
    }
    
    async fn mouse_up(&mut self, btn:piston::MouseButton, manager:&mut IngameManager) {
        if self.game_settings.ignore_mouse_buttons {return}

        // dont accept mouse input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.has_autoplay() || manager.replaying {
            return;
        }

        let time = manager.time();
        if btn == MouseButton::Left {
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::LeftMouse), time, manager).await;
        }
        if btn == MouseButton::Right {
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::RightMouse), time, manager).await;
        }
    }

    async fn mouse_scroll(&mut self, delta:f64, _manager:&mut IngameManager) {
        if self.move_playfield.is_some() {
            {
                let settings = &mut get_settings_mut!().standard_settings;
                settings.playfield_scale += delta / 40.0;
            }

            self.playfield_changed().await;
        }
    }


    async fn controller_press(&mut self, c: &Box<dyn Controller>, btn: u8, manager:&mut IngameManager) {
        // dont accept controller input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.has_autoplay() || manager.replaying {
            return;
        }

        if Some(ControllerButton::Left_Bumper) == c.map_button(btn) {
            let time = manager.time();
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Left), time, manager).await;
        }

        if Some(ControllerButton::Right_Bumper) == c.map_button(btn) {
            let time = manager.time();
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Right), time, manager).await;
        }
    }
    
    async fn controller_release(&mut self, c: &Box<dyn Controller>, btn: u8, manager:&mut IngameManager) {
        // dont accept controller input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.has_autoplay() || manager.replaying {
            return;
        }

        if Some(ControllerButton::Left_Bumper) == c.map_button(btn) {
            let time = manager.time();
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::Left), time, manager).await;
        }

        if Some(ControllerButton::Right_Bumper) == c.map_button(btn) {
            let time = manager.time();
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::Right), time, manager).await;
        }
    }
    
    async fn controller_axis(&mut self, c: &Box<dyn Controller>, axis_data:HashMap<u8, (bool, f64)>, manager:&mut IngameManager) {
        // dont accept controller input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.has_autoplay() || manager.replaying {
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
        self.handle_replay_frame(ReplayFrame::MousePos(new_pos.x as f32, new_pos.y as f32), time, manager).await;
    }

}

#[async_trait]
impl GameModeInfo for StandardGame {
    fn playmode(&self) -> PlayMode {"osu".to_owned()}
    fn end_time(&self) -> f32 {self.end_time}
    fn show_cursor(&self) -> bool {true}
    fn ripple_size(&self) -> Option<f64> {
        Some(self.scaling_helper.scaled_circle_size.x * 1.5)
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
            .map(|(j, w) | {
                (w.end, j.color())
            })
            .collect()
    }

    async fn get_ui_elements(&self, window_size: Vector2, ui_elements: &mut Vec<UIElement>) {
        let playmode = self.playmode();
        let get_name = |name| {
            format!("{playmode}_{name}")
        };

        let size = Vector2::new(100.0, 30.0);
        let combo_bounds = Rectangle::bounds_only(
            Vector2::zero(),
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
            Vector2::y_only(window_size.y / 3.0),
            LeaderboardElement::new()
        ).await);
        
    }

    // fn score_hit_string(hit:&ScoreHit) -> String where Self: Sized {
    //     match hit {
    //         ScoreHit::Miss  => "Miss".to_owned(),
    //         ScoreHit::X50   => "x50".to_owned(),
    //         ScoreHit::X100  => "x100".to_owned(),
    //         ScoreHit::X300  => "x300".to_owned(),
    //         ScoreHit::Xgeki => "Geki".to_owned(),
    //         ScoreHit::Xkatu => "Katu".to_owned(),

    //         ScoreHit::None  => String::new(),
    //         ScoreHit::Other(_, _) => String::new(),
    //     }
    // }

    fn judgment_type(&self) -> Box<dyn HitJudgments> {
        Box::new(OsuHitJudgments::Miss)
    }
}

fn add_judgement_indicator(pos: Vector2, time: f32, hit_value: &OsuHitJudgments, scaling_helper: &Arc<ScalingHelper>, judgment_helper: &JudgmentImageHelper, manager: &mut IngameManager) {
    if !hit_value.should_draw() { return }

    let color = hit_value.color();
    let mut image = judgment_helper.get_from_scorehit(hit_value);
    if let Some(image) = &mut image {
        image.current_pos = pos;
        image.depth = -2.0;

        let scale = Vector2::one() * scaling_helper.scale;
        image.initial_scale = scale;
        image.current_scale = scale;
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


struct StandardAutoHelper {
    // point_trail_angle: Vector2,
    point_trail_start_time: f32,
    point_trail_end_time: f32,
    point_trail_start_pos: Vector2,
    point_trail_end_pos: Vector2,

    /// list of notes currently being held, and what key was held for that note
    holding: HashMap<usize, KeyPress>,

    release_queue:Vec<ReplayFrame>,

    press_counter:usize,
}
impl StandardAutoHelper {
    fn new() -> Self {
        Self {
            // point_trail_angle: Vector2::zero(),
            point_trail_start_time: 0.0,
            point_trail_end_time: 0.0,
            point_trail_start_pos: Vector2::zero(),
            point_trail_end_pos: Vector2::zero(),

            holding: HashMap::new(),

            release_queue: Vec::new(),
            press_counter: 0
        }
    }
    fn get_release_queue(&mut self) -> Vec<ReplayFrame> {
        std::mem::take(&mut self.release_queue)
    }

    fn get_key(&self) -> KeyPress {
        if self.press_counter % 2 == 0 {
            KeyPress::LeftMouse
        } else {
            KeyPress::RightMouse
        }
    }

    fn update(&mut self, time:f32, notes: &mut Vec<Box<dyn StandardHitObject>>, scaling_helper: &Arc<ScalingHelper>, frames: &mut Vec<ReplayFrame>) {
        let mut any_checked = false;

        let map_over = time > notes.last().map(|n| n.end_time(100.0)).unwrap_or(0.0);
        if map_over { return; }


        for i in 0..notes.len() {
            let note = &notes[i];
            if note.was_hit() { continue }

            if self.holding.contains_key(&i) {
                if time >= note.end_time(0.0) {
                    let k = self.holding.remove(&i).unwrap_or(KeyPress::LeftMouse);
                    self.release_queue.push(ReplayFrame::Release(k));

                    let pos = scaling_helper.descale_coords(note.pos_at(time));
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
                
                self.press_counter += 1;
                let k = self.get_key();
                frames.push(ReplayFrame::Press(k));
                if note.note_type() == NoteType::Note {
                    // TODO: add some delay to the release?
                    self.release_queue.push(ReplayFrame::Release(k));
                } else {
                    self.holding.insert(i, k);
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
        if any_checked { return }

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



const CIRCLE_RADIUS_BASE:f64 = 64.0;
const NOTE_BORDER_SIZE:f64 = 2.0;

pub const FIELD_SIZE:Vector2 = Vector2::new(512.0, 384.0); // 4:3

#[derive(Copy, Clone)]
pub struct ScalingHelper {
    /// scale setting in settings
    pub settings_scale: f64,
    /// playfield offset in settings
    pub settings_offset: Vector2,

    /// window size to playfield size scale, scales by settings_scale
    pub scale: f64,

    /// window size from settings
    pub window_size: Vector2,

    /// scaled pos offset for the playfield
    pub scaled_pos_offset: Vector2,

    /// cs size scaled
    pub scaled_cs: f64,

    /// border size scaled
    pub border_scaled: f64,

    pub scaled_circle_size: Vector2,

    // /// scaled playfield
    // playfield_scaled: Rectangle,
    /// scaled playfield
    playfield_scaled_with_cs_border: Rectangle,
}
impl ScalingHelper {
    pub async fn new(cs:f32, window_size: Vector2) -> Self {
        let things = get_settings!().standard_settings.get_playfield();
        let settings_scale = things.0;
        let settings_offset = things.1;
        Self::new_offset_scale(cs, window_size, settings_offset, settings_scale)
    }

    pub fn new_offset_scale(cs:f32, window_size: Vector2, settings_offset: Vector2, settings_scale: f64) -> Self {
        let circle_size = CIRCLE_RADIUS_BASE;
        let border_size = NOTE_BORDER_SIZE;
        
        let scale = (window_size.y / FIELD_SIZE.y) * settings_scale;
        let scaled_pos_offset = (window_size - FIELD_SIZE * scale) / 2.0 + settings_offset;

        let cs_base = (1.0 - 0.7 * (cs as f64 - 5.0) / 5.0) / 2.0;
        let scaled_cs = cs_base * scale;

        let circle_size = Vector2::one() * circle_size * scaled_cs;

        let border_scaled = border_size * scale;

        // let playfield_scaled = Rectangle::new(
        //     [0.2, 0.2, 0.2, 0.5].into(),
        //     f64::MAX-4.0,
        //     scaled_pos_offset,
        //     FIELD_SIZE * scale,
        //     None
        // );

        let playfield_scaled_with_cs_border = Rectangle::new(
            [0.2, 0.2, 0.2, 0.5].into(),
            f64::MAX-4.0,
            scaled_pos_offset - circle_size,
            FIELD_SIZE * scale + circle_size * 2.0,
            None
        );

        Self {
            settings_scale,
            settings_offset,
            scale,
            window_size,
            scaled_pos_offset,
            scaled_cs,
            border_scaled,
            scaled_circle_size: circle_size,

            // playfield_scaled,
            playfield_scaled_with_cs_border
        }
    }

    /// turn playfield (osu) coords into window coords
    pub fn scale_coords(&self, osu_coords:Vector2) -> Vector2 {
        self.scaled_pos_offset + osu_coords * self.scale
    }
    /// turn window coords into playfield coords
    pub fn descale_coords(&self, window_coords: Vector2) -> Vector2 {
        (window_coords - self.scaled_pos_offset) / self.scale
    }
}
