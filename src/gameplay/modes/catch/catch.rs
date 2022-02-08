#![allow(dead_code, unused_variables, unused_mut, unreachable_code)]
use super::*;
use crate::prelude::*;

// const SV_FACTOR:f64 = 700.0; // bc sv is bonked, divide it by this amount

pub const FRUIT_RADIUS_BASE:f64 = 20.0;
pub const DROPLET_RADIUS_BASE:f64 = 10.0;
pub const CATCHER_WIDTH_BASE:f64 = 106.75;
pub const CATCHER_BASE_SPEED:f64 = 1.0;

pub fn hit_y() -> f64 {
    Settings::window_size().y - 100.0
}
// pub const HIT_Y:f64 = window_size().y - 100.0

/// calculate the catch acc for `score`
pub fn calc_acc(score: &Score) -> f64 {
    let x50 = score.x50 as f64;
    let x100 = score.x100 as f64;
    let x300 = score.x300 as f64;
    let geki = score.xgeki as f64;
    let katu = score.xkatu as f64;
    let miss = score.xmiss as f64;

    (x50 + x100 + x300) 
    / (miss + katu + x50 + x100 + x300)
}

pub struct CatchGame {
    // lists
    pub notes: Vec<Box<dyn CatchHitObject>>,
    // list indices
    note_index: usize,
    timing_point_index: usize,

    // hit timing bar stuff
    hitwindow: f32,

    /// when does the map end
    end_time: f32,

    /// when was the last update
    last_update: f32,
    catcher: Catcher,

    
    /// scaling helper to help with scaling
    scaling_helper: ScalingHelper,
    /// needed for scaling recalc
    cs: f32,
    /// autoplay helper
    auto_helper: CatchAutoHelper
}
impl CatchGame {
    pub fn next_note(&mut self) {self.note_index += 1}
}
impl GameMode for CatchGame {
    fn playmode(&self) -> PlayMode {PlayMode::Catch}
    fn end_time(&self) -> f32 {self.end_time}
    fn new(map:&Beatmap) -> Result<Self, crate::errors::TatakuError> {
        let metadata = map.get_beatmap_meta();

        match map {
            Beatmap::Osu(beatmap) => {
                let scaling_helper = ScalingHelper::new(metadata.cs, PlayMode::Catch);
                let mut s = Self {
                    notes: Vec::new(),
                    note_index: 0,

                    timing_point_index: 0,
                    end_time: 0.0,
                    last_update: 0.0,

                    hitwindow: 0.0,
                    catcher: Catcher::new(&map, scaling_helper.clone()),

                    scaling_helper,
                    cs: metadata.cs,
                    auto_helper: CatchAutoHelper::new()
                };

                // add notes
                for note in beatmap.notes.iter() {
                    //TODO!
                    s.notes.push(Box::new(CatchFruit::new(
                        note.time,
                        1.0,
                        FRUIT_RADIUS_BASE, 
                        s.scaling_helper.scale_coords(note.pos).x
                    )));
                }
                for slider in beatmap.sliders.iter() {
                    let SliderDef {time, slides, length, ..} = slider.to_owned();

                    let curve = get_curve(&slider, &map);

                    let l = (length * 1.4) * slides as f32;
                    let v2 = 100.0 * (metadata.slider_multiplier * 1.4);
                    let bl = beatmap.beat_length_at(time, true);
                    let end_time = time + (l / v2 * bl);
                    // let end_time = curve.end_time;
                    
                    let bl = beatmap.beat_length_at(time, metadata.beatmap_version < 8);
                    let skip_period = (bl / metadata.slider_tick_rate).min((end_time - time) / slides as f32);

                    let mut j = time;

                    // // load sounds
                    // // let sound_list_raw = if let Some(list) = split.next() {list.split("|")} else {"".split("")};

                    // // when loading, if unified just have it as sound_types with 1 index
                    // // let mut sound_types:Vec<(HitType, bool)> = Vec::new();

                    // // for i in sound_list_raw {
                    // //     if let Ok(hitsound) = i.parse::<u32>() {
                    // //         let hit_type = if (hitsound & (2 | 8)) > 0 {super::HitType::Kat} else {super::HitType::Don};
                    // //         let finisher = (hitsound & 4) > 0;
                    // //         sound_types.push((hit_type, finisher));
                    // //     }
                    // // }
                    
                    // // let unified_sound_addition = sound_types.len() == 0;
                    // // if unified_sound_addition {
                    // //     sound_types.push((HitType::Don, false));
                    // // }
                    // // println!("{:?}", points);


                    let mut counter = 0;
                    loop {
                        // let sound_type = sound_types[i];
                        if counter % 4 == 0 {
                            s.notes.push(Box::new(CatchFruit::new(
                                j,
                                1.0,//beatmap.slider_velocity_at(j as u64),
                                FRUIT_RADIUS_BASE,
                                s.scaling_helper.scale_coords(curve.position_at_time(j)).x
                            )));
                        } else {
                            s.notes.push(Box::new(CatchDroplet::new(
                                j,
                                1.0,//beatmap.slider_velocity_at(j as u64),
                                DROPLET_RADIUS_BASE,
                                s.scaling_helper.scale_coords(curve.position_at_time(j)).x
                            )));
                        }

                        // if !unified_sound_addition {i = (i + 1) % sound_types.len()}
                        j += skip_period;
                        counter += 1;
                        if !(j < end_time + skip_period / 8.0) {break}
                    }
                }
                for spinner in beatmap.spinners.iter() {
                    let SpinnerDef {time, end_time, ..} = spinner;
                    let length = end_time - time;
                    for i in (0..length as i32).step_by(50) {
                        s.notes.push(Box::new(CatchBanana::new(
                            time + i as f32,
                            1.0,
                            5.0,
                            s.scaling_helper.scale_coords(Vector2::new(i as f64 % FIELD_SIZE.x, 0.0)).x
                        )))
                    }
                }

                s.notes.sort_by(|a, b|a.time().partial_cmp(&b.time()).unwrap());




                // // set dashes
                // // from lazer CatchBeatmapProcessor:214
                // let half_catcher = s.catcher.width / 2.0;
                // let mut last_direction = 0;
                // let mut last_excess = half_catcher;

                // let mut i = 0;
                // let notes = &mut s.notes;
                // 'dash_loop: while i < notes.len() - 1 {
                //     while notes[i].note_type() == NoteType::Spinner {
                //         i += 1;
                //         if i >= notes.len() - 1 {
                //             break 'dash_loop;
                //         }
                //     }
                //     let current = i;
                //     let mut next = i + 1;
                //     while notes[next].note_type() == NoteType::Spinner {
                //         next += 1;
                //         if next >= notes.len() - 1 {
                //             break 'dash_loop;
                //         }
                //     }

                //     // reset dash values
                //     notes[current].reset_dash();
                //     let this_direction = if notes[next].x() > notes[current].x() {-1} else {1};
                //     let time_to_next = notes[next].time() as f64 - notes[current].time() as f64 - 1000.0 / 60.0 / 4.0;
                //     let distance_to_next = 
                //         (notes[next].x() - notes[current].x()) 
                //         - (if last_direction == this_direction {last_excess} else {half_catcher});
                    
                //     let distance_to_hyper = time_to_next * CATCHER_BASE_SPEED - distance_to_next;

                //     // if distance_to_hyper < 0.0 {
                //     //     notes[current].set_dash(&notes[next]);
                //     // } else {
                //     //     notes[current].set_hyper_distance()
                //     // }

                //     i += 1;
                // };


                s.end_time = s.notes.iter().last().unwrap().time();
                Ok(s)
            },
            
            _ => Err(crate::errors::BeatmapError::UnsupportedMode.into()),
        }
    }

    fn update(&mut self, manager:&mut IngameManager, time: f32) {

        if manager.current_mods.autoplay {
            let mut frames = Vec::new();
            self.auto_helper.update(
                time, 
                &mut self.catcher, 
                &mut self.notes, 
                &self.scaling_helper, 
                &mut frames
            );
            for frame in frames {
                self.handle_replay_frame(frame, time, manager)
            }
        }


        self.catcher.update(time as f64);

        // update notes
        for note in self.notes.iter_mut() {note.update(time)}

        // if theres no more notes to hit, show score screen
        if self.note_index >= self.notes.len() {
            manager.completed = true;
            return;
        }

        // check if we missed the current note
        let note_time = self.notes[self.note_index].time();
        if note_time < time {
            if self.notes[self.note_index].causes_miss() {
                let s = &mut manager.score;
                s.xmiss += 1;
                s.combo = 0;
            }
            self.next_note();
        } else if ((note_time - time).abs()) < self.hitwindow {
            let note = self.notes.get_mut(self.note_index).unwrap();

            if self.catcher.catches(note) {
                match note.get_points() {
                    ScoreHit::X300 => {
                        manager.score.hit300(time, note_time);
                        manager.hitbar_timings.push((time, 3.0));
                        self.next_note();
                    }
                    ScoreHit::X100 => {
                        manager.score.hit100(time, note_time);
                        manager.hitbar_timings.push((time, 2.0));
                        self.next_note();
                    }
                    ScoreHit::Other(score, _consume) => { // spinner drop
                        manager.score.score += score as u64;
                        manager.hitbar_timings.push((time, 1.0));
                        self.next_note();
                    }
                    _ => {}
                }
                #[cfg(feature="bass_audio")]
                Audio::play_preloaded("don").unwrap();
                #[cfg(feature="neb_audio")]
                Audio::play_preloaded("don");
            } else {
                if note.causes_miss() {
                    let s = &mut manager.score;
                    s.xmiss += 1;
                    s.combo = 0;
                }
            }
        }

        let timing_points = &manager.timing_points;
        // check timing point
        if self.timing_point_index + 1 < timing_points.len() && timing_points[self.timing_point_index + 1].time <= time {
            self.timing_point_index += 1;
        }

        self.last_update = time;
    }
    fn draw(&mut self, args:RenderArgs, _manager:&mut IngameManager, list:&mut Vec<Box<dyn Renderable>>) {
        // draw the playfield

        list.push(Box::new(self.scaling_helper.playfield_scaled_with_cs_border.clone()));
        
        // let playfield = Rectangle::new(
        //     [0.2, 0.2, 0.2, 1.0].into(),
        //     f64::MAX-4.0,
        //     Vector2::new(x_offset(), 0.0),
        //     Vector2::new(FIELD_SIZE.x, args.window_size[1]),
        //     if manager.beatmap.timing_points[self.timing_point_index].kiai {
        //         Some(Border::new(Color::YELLOW, 2.0))
        //     } else {None}
        // );
        // list.push(Box::new(playfield));
        self.catcher.draw(list);

        // for curve in self.curves.iter() {
        //     for line in curve.path.iter() {
        //         list.push(Box::new(ayyeve_piston_ui::render::Line::new(
        //             line.p1,
        //             line.p2,
        //             5.0,
        //             -999.0,
        //             Color::GREEN
        //         )))
        //     }
        // }

        // draw notes
        for note in self.notes.iter_mut() {note.draw(args, list)}
    }


    fn handle_replay_frame(&mut self, frame:ReplayFrame, time:f32, manager:&mut IngameManager) {
        if !manager.replaying {
            manager.replay.frames.push((time, frame.clone()));
            manager.outgoing_spectator_frame((time, SpectatorFrameData::ReplayFrame{frame}));
        }
        
        match frame {
            ReplayFrame::Press(k) => {
                match k {
                    KeyPress::Left => self.catcher.left_held = true,
                    KeyPress::Right => self.catcher.right_held = true,
                    KeyPress::Dash => self.catcher.dash_held = true,
                    _ => {}
                }
            }
            ReplayFrame::Release(k) => {
                match k {
                    KeyPress::Left => self.catcher.left_held = false,
                    KeyPress::Right => self.catcher.right_held = false,
                    KeyPress::Dash => self.catcher.dash_held = false,
                    _ => {}
                }
            }
            
            _ => {}
        };
    }
    fn key_down(&mut self, key:piston::Key, manager:&mut IngameManager) {
        let settings = Settings::get().catch_settings;
        let time = manager.time();

        
        // dont accept key input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.autoplay || manager.replaying {
            return;
        }


        if key == settings.left_key {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Left), time, manager);
        }
        if key == settings.right_key {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Right), time, manager);
        }
        if key == settings.dash_key { 
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Dash), time, manager);
        }
    }
    fn key_up(&mut self, key:piston::Key, manager:&mut IngameManager) {

        // dont accept key input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.autoplay || manager.replaying {
            return;
        }


        let settings = Settings::get().catch_settings;
        let time = manager.time();

        if !manager.replaying {
            if key == settings.left_key {
                self.handle_replay_frame(ReplayFrame::Release(KeyPress::Left), time, manager);
            }
            if key == settings.right_key {
                self.handle_replay_frame(ReplayFrame::Release(KeyPress::Right), time, manager);
            }
            if key == settings.dash_key { 
                self.handle_replay_frame(ReplayFrame::Release(KeyPress::Dash), time, manager);
            }
        }
    }

    fn reset(&mut self, beatmap:&Beatmap) {
        for note in self.notes.as_mut_slice() {
            note.reset();
        }
        
        self.note_index = 0;
        self.timing_point_index = 0;

        let od = beatmap.get_beatmap_meta().od;
        self.hitwindow = map_difficulty(od, 50.0, 35.0, 20.0);
    }

    fn skip_intro(&mut self, manager: &mut IngameManager) {
        if self.note_index > 0 {return}

        let y_needed = 0.0;
        let mut time = manager.time();

        // loop {
        //     let mut found = false;
        //     for note in self.notes.iter() {if note.x_at(time) <= x_needed {found = true; break}}
        //     if found {break}
        //     time += 1;
        // }

        let mut time = time as f32;
        if manager.lead_in_time > 0.0 {
            if time > manager.lead_in_time {
                time -= manager.lead_in_time - 0.01;
                manager.lead_in_time = 0.01;
            }
        }

        if time < 0.0 {return}
        
        #[cfg(feature="bass_audio")]
        manager.song.set_position(time as f64).unwrap();
        #[cfg(feature="neb_audio")]
        manager.song.upgrade().unwrap().set_position(time);
    }

    fn timing_bar_things(&self) -> (Vec<(f32,Color)>, (f32,Color)) {
        (vec![], (0.0, Color::RED))
    }
    fn combo_bounds(&self) -> Rectangle {
        let window_size = Settings::window_size();
        Rectangle::bounds_only(
            Vector2::new(0.0, window_size.y * (1.0/3.0)),
            Vector2::new(window_size.x, 30.0)
        )
    }

    
    fn apply_auto(&mut self, settings: &crate::game::BackgroundGameSettings) {
        for note in self.notes.iter_mut() {
            note.set_alpha(settings.opacity)
        }
    }
}

struct Catcher {
    width: f64,
    pos: Vector2,

    pub left_held: bool,
    pub right_held: bool,
    pub dash_held: bool,

    last_update: f64,
    move_speed: f64,
    scaling_helper: ScalingHelper
}
impl Catcher {
    fn new(_beatmap:&Beatmap, scaling_helper: ScalingHelper) -> Self {
        let width = 100.0 * scaling_helper.scale;
        Self {
            width,

            move_speed: 0.5, // should calc this somehow
            pos: Vector2::new(scaling_helper.scale_coords(FIELD_SIZE / 2.0).x, hit_y()),
            scaling_helper,
            left_held: false,
            right_held: false,
            dash_held: false,
            last_update: 0.0
        }
    }

    fn update(&mut self, time:f64) {
        if self.last_update == 0.0 {return self.last_update = time}
        let delta = time - self.last_update;

        if self.left_held {
            self.pos.x -= self.move_speed() * delta;
        }
        if self.right_held {
            self.pos.x += self.move_speed() * delta;
        }

        let bounds = self.scaling_helper.playfield_scaled_with_cs_border;

        // check bounds
        self.pos.x = self.pos.x.clamp(bounds.pos.x, bounds.pos.x + bounds.size.x);
        self.last_update = time;
    }
    fn move_speed(&self) -> f64 {
        if self.dash_held {
            self.move_speed * 1.5 //TODO!!
        } else {
            self.move_speed
        }
    }
    fn draw(&mut self, list:&mut Vec<Box<dyn Renderable>>) {
        list.push(Box::new(Rectangle::new(
            Color::BLUE,
            -100.0,
            self.pos,
            Vector2::new(self.width, 10.0),
            None
        )))
    }

    fn catches(&self, note: &Box<dyn CatchHitObject>) -> bool {
        let note_x = note.x();
        let note_width = note.radius() * 2.0;

        note_x + note_width > self.pos.x && note_x < self.pos.x + self.width
    }
}



struct CatchAutoHelper {
    // point_trail_angle: Vector2,
    point_trail_start_time: f32,
    point_trail_end_time: f32,
    point_trail_start_pos: f64,
    point_trail_end_pos: f64,

    current_note_index: usize,
}
impl CatchAutoHelper {

    fn new() -> Self {
        Self {
            // point_trail_angle: Vector2::zero(),
            point_trail_start_time: 0.0,
            point_trail_end_time: 0.0,
            current_note_index: 0,
            point_trail_start_pos: 0.0,
            point_trail_end_pos: 0.0,
        }
    }

    fn update(
        &mut self, 
        time:f32, 
        catcher: &mut Catcher,
        notes: &mut Vec<Box<dyn CatchHitObject>>, 
        scaling_helper: &ScalingHelper, 
        frames: &mut Vec<ReplayFrame>
    ) {
        let mut any_checked = false;

        for i in 0..notes.len() {
            let note = &notes[i];

            if time >= note.time() && !note.was_hit() {
                let pos = Vector2::new(note.pos_at(time, &scaling_helper), 0.0).x;
                // if pos < catcher.pos.x {
                //     frames.push(ReplayFrame::Press(KeyPress::Left));
                //     frames.push(ReplayFrame::Release(KeyPress::Left));
                // } else {
                //     frames.push(ReplayFrame::Press(KeyPress::Right));
                //     frames.push(ReplayFrame::Release(KeyPress::Right));
                // }
                catcher.pos.x = pos - catcher.width / 2.0;
                println!("new pos: {}, index: {}", catcher.pos.x, i);
                return;
                
                // frames.push(ReplayFrame::MousePos(
                //     pos as f32,
                //     0.0
                // ));

                // if i == self.current_note_index {
                //     if note.note_type() != NoteType::Note {
                //         if time <= note.end_time(0.0) {
                //             frames.push(ReplayFrame::Release(KeyPress::LeftMouse));
                //         }
                //     }
                // } else {
                //     frames.push(ReplayFrame::Press(KeyPress::LeftMouse));
                //     frames.push(ReplayFrame::Release(KeyPress::LeftMouse));
                // }

                if i + 1 >= notes.len() {
                    self.point_trail_start_pos = pos;
                    self.point_trail_end_pos = scaling_helper.descale_coords(scaling_helper.window_size / 2.0).x;
                    
                    self.point_trail_start_time = 0.0;
                    self.point_trail_end_time = 1.0;
                    continue;
                }

                // draw a line to the next note
                let next_note = &notes[i + 1];

                self.point_trail_start_pos = pos;
                self.point_trail_end_pos = scaling_helper.descale_coords(Vector2::new(next_note.pos_at(self.point_trail_end_time, scaling_helper), 0.0)).x;
                
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
        
        let new_pos = f64::lerp(self.point_trail_start_pos, self.point_trail_end_pos, len as f64);
        frames.push(ReplayFrame::MousePos(
            new_pos as f32,
            0.0
        ));
    }
}