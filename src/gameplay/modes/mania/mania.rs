use crate::prelude::*;
use super::mania_notes::*;

const FIELD_DEPTH:f64 = 110.0;
const HIT_AREA_DEPTH: f64 = 99.9;

// timing bar consts
pub const BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0); // timing bar color
const BAR_HEIGHT:f64 = 4.0; // how tall is a timing bar
const BAR_SPACING:f32 = 4.0; // how many beats between timing bars
const BAR_DEPTH:f64 = -90.0;

// sv things (TODO!: rework sv to not suck)
const SV_FACTOR:f32 = 700.0; // bc sv is bonked, divide it by this amount
const SV_CHANGE_DELTA:f32 = 0.1; // how much to change the sv by when a sv change key is pressed


/// calculate the mania acc for `score`
pub fn calc_acc(score: &Score) -> f64 {
    let x50 = score.x50 as f64;
    let x100 = score.x100 as f64;
    let x300 = score.x300 as f64;
    let geki = score.xgeki as f64;
    let katu = score.xkatu as f64;
    let miss = score.xmiss as f64;

    // (50*count50 + 100*count100 + 200*count_katu + 300*(count300 + count_geki)) / (300*sum(count_miss, count50, count100, count300, count_geki, count_katu)); 
    (50.0 * x50 + 100.0 * x100 + 200.0 * katu + 300.0 * (x300 + geki))
    / (300.0 * (miss + x50 + x100 + x300 + geki + katu))
}

pub struct ManiaGame {
    // lists
    columns: Vec<Vec<Box<dyn ManiaHitObject>>>,
    timing_bars: Vec<TimingBar>,
    // list indices
    timing_point_index: usize,
    column_indices: Vec<usize>,
    /// true if held
    column_states: Vec<bool>,

    // hit timing bar stuff
    hitwindow_300: f32,
    hitwindow_100: f32,
    hitwindow_miss: f32,

    end_time: f32,
    sv_mult: f32,
    column_count: u8,

    auto_helper: ManiaAutoHelper,
    playfield: Arc<ManiaPlayfieldSettings>,

    mania_skin_settings: Option<Arc<ManiaSkinSettings>>,
}
impl ManiaGame {
    /// get the x_pos for `col`
    pub fn col_pos(&self, col:u8) -> f64 {
        let total_width = self.column_count as f64 * self.playfield.column_width;
        let x_offset = self.playfield.x_offset + (Settings::window_size().x - total_width) / 2.0;

        x_offset + self.playfield.x_offset + (self.playfield.column_width + self.playfield.column_spacing) * col as f64
    }

    pub fn get_color(&self, col:u8) -> Color {
        match col {
            0|3 => Color::BLUE_ORCHID,
            1|2 => Color::ACID_GREEN,

            _ => Color::WHITE
        }
    }

    fn next_note(&mut self, col:usize) {
        (*self.column_indices.get_mut(col).unwrap()) += 1;
    }

    fn set_sv(&mut self, sv:f32) {
        let scaled_sv = (sv / SV_FACTOR) * self.sv_mult;
        for col in self.columns.iter_mut() {
            for note in col.iter_mut() {
                note.set_sv(scaled_sv);
            }
        }
        for bar in self.timing_bars.iter_mut() {
            bar.set_sv(scaled_sv);
        }
    }
}
impl GameMode for ManiaGame {
    fn playmode(&self) -> PlayMode {"mania".to_owned()}
    fn end_time(&self) -> f32 {self.end_time}

    fn new(beatmap:&Beatmap, _diff_calc_only: bool) -> Result<Self, crate::errors::TatakuError> {
        let metadata = beatmap.get_beatmap_meta();

        let settings = get_settings!().mania_settings.clone();
        let playfields = &settings.playfield_settings.clone();
        let auto_helper = ManiaAutoHelper::new();

        let all_mania_skin_settings = &SKIN_MANAGER.read().current_skin_config().mania_settings;
        let mut mania_skin_settings = None;

        const DEFAULT_SNAP: Color = Color::SILVER;

        const SNAP_COLORS:&[(f32, Color)] = &[
            (0.0,        Color::RED),
            (1.0,        Color::RED),
            (1.0 / 2.0,  Color::BLUE),
            (1.0 / 3.0,  Color::PURPLE),
            (2.0 / 3.0,  Color::PURPLE),
            (1.0 / 4.0,  Color::YELLOW),
            (3.0 / 4.0,  Color::YELLOW),
            (1.0 / 6.0,  Color::PINK),
            (5.0 / 6.0,  Color::PINK),
            (1.0 / 8.0,  Color::ORANGE),
            (3.0 / 8.0,  Color::ORANGE),
            (5.0 / 8.0,  Color::ORANGE),
            (7.0 / 8.0,  Color::ORANGE),
            (1.0 / 12.0, Color::AQUA),
            (5.0 / 12.0, Color::AQUA),
            (7.0 / 12.0, Color::AQUA),
            (11.0 / 12.0, Color::AQUA),
            (1.0 / 16.0, Color::GREEN),
            (3.0 / 16.0, Color::GREEN),
            (5.0 / 16.0, Color::GREEN),
            (7.0 / 16.0, Color::GREEN),
            (9.0 / 16.0, Color::GREEN),
            (11.0 / 16.0, Color::GREEN),
            (13.0 / 16.0, Color::GREEN),
            (15.0 / 16.0, Color::GREEN),
        ];
        let timing_points = beatmap.get_timing_points();
        let get_color = |time| {
            let mut tp = &timing_points[0];
            for t in timing_points.iter() {
                if t.is_inherited() { continue }

                if t.time <= time {
                    tp = t
                }
                else { break }
            }
        
            let offset = tp.time;
            let length = tp.beat_length;

            let threshold = 1.0 / length;

            let diff = time - offset;
            let snap = (diff / length) % 1.0;
            
            // temp/debug
            let mut closest_snap = (0.0, 99999.0);

            for (time, color) in SNAP_COLORS {
                let diff = (snap - *time).abs();
                if diff < 2.5 * threshold {
                    return *color;
                }
                if diff < closest_snap.1 {
                    closest_snap = (1.0 / *time, diff);
                }
            }
            
            // println!("threshold: {}", threshold);
            // println!("snap: {} - {:.1}", snap,  1.0 / snap);
            // println!("lowestdiff: {:.5} {:.5}", closest_snap.0, closest_snap.1);

            DEFAULT_SNAP
        };

        match beatmap {
            Beatmap::Osu(beatmap) => {
                let mut s = Self {
                    columns: Vec::new(),
                    column_indices:Vec::new(),
                    column_states: Vec::new(),
                    timing_bars: Vec::new(),
                    timing_point_index: 0,
                    end_time: 0.0,

                    hitwindow_100: 0.0,
                    hitwindow_300: 0.0,
                    hitwindow_miss: 0.0,

                    sv_mult: 1.0,
                    column_count: beatmap.metadata.cs as u8,

                    auto_helper,
                    playfield: Arc::new(playfields[(beatmap.metadata.cs-1.0) as usize].clone()),
                    mania_skin_settings
                };

                for i in all_mania_skin_settings.iter() {
                    if i.keys == s.column_count {
                        s.mania_skin_settings = Some(Arc::new(i.clone()));
                    }
                }

                // init defaults for the columsn
                for _col in 0..s.column_count {
                    s.columns.push(Vec::new());
                    s.column_indices.push(0);
                    s.column_states.push(false);
                }

                // add notes
                for note in beatmap.notes.iter() {
                    if metadata.mode == "mania" {
                        let column = (note.pos.x * s.column_count as f64 / 512.0).floor() as u8;
                        let x = s.col_pos(column);

                        s.columns[column as usize].push(Box::new(ManiaNote::new(
                            note.time,
                            column,
                            get_color(note.time),
                            x,
                            s.playfield.clone(),
                            s.mania_skin_settings.clone(),
                        )));
                    }
                }
                for hold in beatmap.holds.iter() {
                    let HoldDef {pos, time, end_time, ..} = hold.to_owned();
        
                    let column = (pos.x * s.column_count as f64 / 512.0).floor() as u8;
                    let x = s.col_pos(column);
                    s.columns[column as usize].push(Box::new(ManiaHold::new(
                        time,
                        end_time,
                        column,
                        get_color(time),
                        x,
                        s.playfield.clone(),
                        s.mania_skin_settings.clone(),
                    )));
                }

                for _slider in beatmap.sliders.iter() {
                    // let SliderDef {pos, time, slides, length, ..} = slider.to_owned();
                    // let time = time as u64;
                    
                    // let l = (length * 1.4) * slides as f64;
                    // let v2 = 100.0 * (beatmap.metadata.slider_multiplier as f64 * 1.4);
                    // let bl = beatmap.beat_length_at(time as f64, true);
                    // let end_time = time + (l / v2 * bl) as u64;
            
                    // let column = (pos.x * s.column_count as f64 / 512.0).floor() as u8;
                    // let x = s.col_pos(column);
                    // s.columns[column as usize].push(Box::new(ManiaHold::new(
                    //     time as u64,
                    //     end_time as u64,
                    //     x
                    // )));
                }
                for _spinner in beatmap.spinners.iter() {
                    // let SpinnerDef {time, end_time, ..} = spinner;
                    //TODO
                }

                // get end time
                for col in s.columns.iter_mut() {
                    col.sort_by(|a, b|a.time().partial_cmp(&b.time()).unwrap());
                    if let Some(last_note) = col.iter().last() {
                        s.end_time = s.end_time.max(last_note.time());
                    }
                }

                Ok(s)
            },
            Beatmap::Quaver(beatmap) => {
                let column_count = beatmap.mode.into();
                for i in all_mania_skin_settings.iter() {
                    if i.keys == column_count {
                        mania_skin_settings = Some(Arc::new(i.clone()));
                    }
                }

                let mut s = Self {
                    columns: Vec::new(),
                    column_indices:Vec::new(),
                    column_states: Vec::new(),
        
                    timing_bars: Vec::new(),
                    timing_point_index: 0,
                    end_time: 0.0,
        
                    hitwindow_100: 0.0,
                    hitwindow_300: 0.0,
                    hitwindow_miss: 0.0,

                    sv_mult: 1.0,
                    column_count,

                    auto_helper,
                    playfield: Arc::new(playfields[(column_count-1) as usize].clone()),
                    mania_skin_settings
                };
                
                // init defaults for the columns
                for _col in 0..s.column_count {
                    s.columns.push(Vec::new());
                    s.column_indices.push(0);
                    s.column_states.push(false);
                }

                // add notes
                for note in beatmap.hit_objects.iter() {
                    let column = note.lane - 1;
                    let time = note.start_time;
                    let x = s.col_pos(column);

                    if let Some(end_time) = note.end_time {
                        s.columns[column as usize].push(Box::new(ManiaHold::new(
                            time,
                            end_time,
                            column,
                            get_color(time),
                            x,
                            s.playfield.clone(),
                            s.mania_skin_settings.clone(),
                        )));
                    } else {
                        s.columns[column as usize].push(Box::new(ManiaNote::new(
                            time,
                            column,
                            get_color(time),
                            x,
                            s.playfield.clone(),
                            s.mania_skin_settings.clone(),
                        )));
                    }
                }
        
                // get end time
                for col in s.columns.iter_mut() {
                    col.sort_by(|a, b|a.time().partial_cmp(&b.time()).unwrap());
                    if let Some(last_note) = col.iter().last() {
                        s.end_time = s.end_time.max(last_note.time());
                    }
                }
                
        
                Ok(s)
            },
            Beatmap::Stepmania(beatmap) => {
                // stepmania maps are always 4k
                let column_count = 4;
                for i in all_mania_skin_settings.iter() {
                    if i.keys == column_count {
                        mania_skin_settings = Some(Arc::new(i.clone()));
                    }
                }

                let mut s = Self {
                    columns: Vec::new(),
                    column_indices:Vec::new(),
                    column_states: Vec::new(),
        
                    timing_bars: Vec::new(),
                    timing_point_index: 0,
                    end_time: 0.0,
        
                    hitwindow_100: 0.0,
                    hitwindow_300: 0.0,
                    hitwindow_miss: 0.0,

                    sv_mult: 1.0,
                    column_count,

                    auto_helper,
                    playfield: Arc::new(playfields[(column_count-1) as usize].clone()),
                    mania_skin_settings
                };
                
                // init defaults for the columns
                for _col in 0..s.column_count {
                    s.columns.push(Vec::new());
                    s.column_indices.push(0);
                    s.column_states.push(false);
                }

                // add notes
                for note in beatmap.chart_info.notes.iter() {
                    let column = note.column;
                    let time = note.start;
                    let x = s.col_pos(column);

                    if let Some(end_time) = note.end {
                        s.columns[column as usize].push(Box::new(ManiaHold::new(
                            time,
                            end_time,
                            column,
                            get_color(time),
                            x,
                            s.playfield.clone(),
                            s.mania_skin_settings.clone(),
                        )));
                    } else {
                        s.columns[column as usize].push(Box::new(ManiaNote::new(
                            time,
                            column,
                            get_color(time),
                            x,
                            s.playfield.clone(),
                            s.mania_skin_settings.clone(),
                        )));
                    }
                }
        
                // get end time
                for col in s.columns.iter_mut() {
                    col.sort_by(|a, b|a.time().partial_cmp(&b.time()).unwrap());
                    if let Some(last_note) = col.iter().last() {
                        s.end_time = s.end_time.max(last_note.time());
                    }
                }
                
                Ok(s)
            }
            
            _ => Err(BeatmapError::UnsupportedBeatmap.into()),
        }
    }

    fn handle_replay_frame(&mut self, frame:ReplayFrame, time:f32, manager:&mut IngameManager) {
        if !manager.replaying {
            manager.replay.frames.push((time, frame));
            manager.outgoing_spectator_frame((time, SpectatorFrameData::ReplayFrame{frame}));
        }

        let sound = "kat";
        macro_rules! play_sound {
            ($sound:expr) => {
                #[cfg(feature="bass_audio")]
                Audio::play_preloaded($sound).unwrap();
                #[cfg(feature="neb_audio")]
                Audio::play_preloaded($sound);
            }
        }

        match frame {
            ReplayFrame::Press(key) => {
                let col:usize = match key {
                    KeyPress::Mania1 => 0,
                    KeyPress::Mania2 => 1,
                    KeyPress::Mania3 => 2,
                    KeyPress::Mania4 => 3,
                    KeyPress::Mania5 => 4,
                    KeyPress::Mania6 => 5,
                    KeyPress::Mania7 => 6,
                    KeyPress::Mania8 => 7,
                    KeyPress::Mania9 => 8,
                    _ => return
                };
                // let hit_type:HitType = key.into();
                // let hit_volume = get_settings!().get_effect_vol() * (manager.beatmap.timing_points[self.timing_point_index].volume as f32 / 100.0);

                // if theres no more notes to hit, return after playing the sound
                if self.column_indices[col] >= self.columns[col].len() {
                    play_sound!(sound);
                    return;
                }
                let note = &mut self.columns[col][self.column_indices[col]];
                let note_time = note.time();
                *self.column_states.get_mut(col).unwrap() = true;

                let diff = (time - note_time).abs();
                // normal note
                if diff < self.hitwindow_300 {
                    note.hit(time);

                    manager.score.hit300(time, note_time);
                    manager.hitbar_timings.push((time, time - note_time));
                    manager.health.give_life();

                    play_sound!(sound);
                    if note.note_type() != NoteType::Hold {
                        self.next_note(col);
                    }
                } else if diff < self.hitwindow_100 {
                    note.hit(time);

                    manager.score.hit100(time, note_time);
                    manager.hitbar_timings.push((time, time - note_time));
                    manager.health.give_life();
                    play_sound!(sound);
                    //TODO: indicate this was a bad hit

                    if note.note_type() != NoteType::Hold {
                        self.next_note(col);
                    }
                } else if diff < self.hitwindow_miss { // too early, miss
                    note.miss(time);

                    manager.score.hit_miss(time, note_time);
                    manager.hitbar_timings.push((time, time - note_time));
                    
                    manager.health.take_damage();
                    if manager.health.is_dead() {
                        manager.fail()
                    }

                    if note.note_type() != NoteType::Hold {
                        self.next_note(col);
                    }
                    play_sound!(sound);
                    //TODO: indicate this was a miss
                } else { // way too early, ignore
                    // play sound
                    play_sound!(sound);
                }
            
            }
            ReplayFrame::Release(key) => {
                let col:usize = match key {
                    KeyPress::Mania1 => 0,
                    KeyPress::Mania2 => 1,
                    KeyPress::Mania3 => 2,
                    KeyPress::Mania4 => 3,
                    KeyPress::Mania5 => 4,
                    KeyPress::Mania6 => 5,
                    KeyPress::Mania7 => 6,
                    KeyPress::Mania8 => 7,
                    KeyPress::Mania9 => 8,
                    _ => return
                };
                *self.column_states.get_mut(col).unwrap() = false;
                if self.column_indices[col] >= self.columns[col].len() {return}

                let note = &mut self.columns[col][self.column_indices[col]];
                if time < note.time() - self.hitwindow_miss 
                || time > note.end_time(self.hitwindow_miss) {return}
                note.release(time);

                if note.note_type() == NoteType::Hold {
                    let note_time = note.end_time(0.0);
                    let diff = (time - note_time).abs();
                    // normal note
                    if diff < self.hitwindow_300 {
                        manager.score.hit300(time, note_time);
                        manager.hitbar_timings.push((time, time - note_time));
                        manager.health.give_life();
                        
                        // // play sound
                        // play_sound!(sound);

                        self.next_note(col);
                    } else if diff < self.hitwindow_100 {
                        manager.score.hit100(time, note_time);
                        manager.hitbar_timings.push((time, time - note_time));
                        manager.health.give_life();
                        //TODO: indicate this was a bad hit

                        // play sound
                        play_sound!(sound);

                        self.next_note(col);
                    } else if diff < self.hitwindow_miss { // too early, miss
                        manager.score.hit_miss(time, note_time);
                        manager.hitbar_timings.push((time, time - note_time));
                        
                        manager.health.take_damage();
                        if manager.health.is_dead() {
                            manager.fail()
                        }

                        //TODO: indicate this was a miss
                        self.next_note(col);
                    }
                }
                
                // self.columns[col][self.column_indices[col]].release(time);
            }
        
            _ => {}
        }
    }

    fn key_down(&mut self, key:piston::Key, manager:&mut IngameManager) {

        // check sv change keys
        if key == Key::F4 {
            self.sv_mult += SV_CHANGE_DELTA;
            
            let time = manager.time();
            self.set_sv(manager.beatmap.slider_velocity_at(time));
            return;
        }
        if key == Key::F3 {
            self.sv_mult -= SV_CHANGE_DELTA;

            let time = manager.time();
            self.set_sv(manager.beatmap.slider_velocity_at(time));
            return;
        }

        // dont accept key input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.autoplay || manager.replaying {
            return;
        }


        let settings = get_settings!();
        let mut game_key = KeyPress::RightDon;

        let keys = &settings.mania_settings.keys[(self.column_count-1) as usize];
        let base_key = KeyPress::Mania1 as u8;
        for col in 0..self.column_count as usize {
            let k = keys[col];
            if k == key {
                game_key = ((col + base_key as usize) as u8).into();
                break;
            }
        }
        if game_key == KeyPress::RightDon {return}
        let time = manager.time();
        self.handle_replay_frame(ReplayFrame::Press(game_key), time, manager);
    }
    fn key_up(&mut self, key:piston::Key, manager:&mut IngameManager) {

        
        // dont accept key input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.autoplay || manager.replaying {
            return;
        }

        let settings = get_settings!();
        let mut game_key = KeyPress::RightDon;

        let keys = &settings.mania_settings.keys[(self.column_count-1) as usize];
        let base_key = KeyPress::Mania1 as u8;
        for col in 0..self.column_count as usize {
            let k = keys[col];
            if k == key {
                game_key = ((col + base_key as usize) as u8).into();
                break;
            }
        }
        if game_key == KeyPress::RightDon {return}
        let time = manager.time();

        self.handle_replay_frame(ReplayFrame::Release(game_key), time, manager);
    }

    fn reset(&mut self, beatmap:&Beatmap) {
        for col in self.columns.iter_mut() {
            for note in col.iter_mut() {
                note.reset();
            }
        }
        for i in 0..self.columns.len() {
            self.column_indices[i] = 0;
            self.column_states[i] = false;
        }
        
        self.timing_point_index = 0;

        let od = beatmap.get_beatmap_meta().od;
        // setup hitwindows
        self.hitwindow_miss = map_difficulty(od, 188.0, 173.0, 158.0);
        self.hitwindow_100 = map_difficulty(od, 127.0, 112.0, 97.0);
        self.hitwindow_300 = map_difficulty(od, 64.0, 49.0, 34.0);

        let window_size = Settings::window_size();

        // setup timing bars
        //TODO: it would be cool if we didnt actually need timing bar objects, and could just draw them
        if self.timing_bars.len() == 0 {
            let tps = beatmap.get_timing_points();
            // load timing bars
            let parent_tps = tps.iter().filter(|t|!t.is_inherited()).collect::<Vec<&TimingPoint>>();
            let mut time = parent_tps[0].time;
            let mut tp_index = 0;
            let step = beatmap.beat_length_at(time, false);
            time %= step; // get the earliest bar line possible

            let bar_width = self.column_count as f64 * self.playfield.column_width;
            let x = (window_size.x - bar_width) / 2.0;

            loop {
                // if theres a bpm change, adjust the current time to that of the bpm change
                let next_bar_time = beatmap.beat_length_at(time, false) * BAR_SPACING; // bar spacing is actually the timing point measure

                // edge case for aspire maps
                if next_bar_time.is_nan() || next_bar_time == 0.0 {
                    break;
                }

                // add timing bar at current time
                self.timing_bars.push(TimingBar::new(time, bar_width, x, self.playfield.clone()));

                if tp_index < parent_tps.len() && parent_tps[tp_index].time <= time + next_bar_time {
                    time = parent_tps[tp_index].time;
                    tp_index += 1;
                    continue;
                }

                // why isnt this accounting for bpm changes? because the bpm change doesnt allways happen inline with the bar idiot
                time += next_bar_time;
                if time >= self.end_time || time.is_nan() {break}
            }

            println!("created {} timing bars", self.timing_bars.len());
        }

        let sv = beatmap.slider_velocity_at(0.0);
        self.set_sv(sv);
    }


    fn update(&mut self, manager:&mut IngameManager, time: f32) {

        if manager.current_mods.autoplay {
            let mut frames = Vec::new();
            self.auto_helper.update(&self.columns, &mut self.column_indices, time, &mut frames);
            for frame in frames {
                self.handle_replay_frame(frame, time, manager)
            }
        }

        // update notes
        for col in self.columns.iter_mut() {
            for note in col.iter_mut() {note.update(time)}
        }

        // show score screen if map is over
        if time >= self.end_time {
            manager.completed = true;
            return;
        }

        // check if we missed the current note
        for col in 0..self.column_count as usize {
            if self.column_indices[col] >= self.columns[col].len() {continue}
            let note = &self.columns[col][self.column_indices[col]];
            if note.end_time(self.hitwindow_miss) <= time {
                // need to set these manually instead of score.hit_miss,
                // since we dont want to add anything to the hit error list
                let s = &mut manager.score;
                s.xmiss += 1;
                s.combo = 0;
                
                manager.health.take_damage();
                if manager.health.is_dead() {
                    manager.fail()
                }
                
                self.next_note(col);
            }
        }
        
        // TODO: might move tbs to a (time, speed) tuple
        for tb in self.timing_bars.iter_mut() {tb.update(time)}

        let timing_points = &manager.timing_points;
        // check timing point
        if self.timing_point_index + 1 < timing_points.len() && timing_points[self.timing_point_index + 1].time <= time {
            self.timing_point_index += 1;
            // let tp = &timing_points[self.timing_point_index];
            let sv = manager.beatmap.slider_velocity_at(time);
            self.set_sv(sv);
        }
    }
    fn draw(&mut self, args:RenderArgs, manager:&mut IngameManager, list:&mut Vec<Box<dyn Renderable>>) {
        let window_size = Settings::window_size();

        // playfield
        list.push(Box::new(Rectangle::new(
            Color::new(0.0, 0.0, 0.0, 0.8),
            FIELD_DEPTH + 1.0,
            Vector2::new(self.col_pos(0), 0.0),
            Vector2::new(self.col_pos(self.column_count) - self.col_pos(0), window_size.y),
            Some(Border::new(if manager.current_timing_point().kiai {Color::YELLOW} else {Color::BLACK}, 1.2))
        )));

        // draw columns
        for col in 0..self.column_count {
            let x = self.col_pos(col);

            // column background
            list.push(Box::new(Rectangle::new(
                Color::new(0.1, 0.1, 0.1, 0.8),
                FIELD_DEPTH,
                Vector2::new(x, 0.0),
                Vector2::new(self.playfield.column_width, window_size.y),
                Some(Border::new(Color::GREEN, 1.2))
            )));


            // hit area/button state for this col
            let mut drew_image = false;
            if let Some(settings) = &self.mania_skin_settings {
                let map = if self.column_states[col as usize] {&settings.key_image_d} else {&settings.key_image};
                
                if let Some(path) = map.get(&col) {
                    if let Some(img) = SKIN_MANAGER.write().get_texture(path, true) {
                        let mut img = img.clone();
                        img.origin = Vector2::zero();
                        img.current_pos = Vector2::new(x, self.playfield.hit_y());
                        img.current_scale = self.playfield.note_size() / img.tex_size();

                        list.push(Box::new(img));
                        drew_image = true;
                    }
                }
            }


            if !drew_image {
                list.push(Box::new(Rectangle::new(
                    if self.column_states[col as usize] {self.get_color(col)} else {Color::TRANSPARENT_WHITE},
                    HIT_AREA_DEPTH,
                    Vector2::new(x, self.playfield.hit_y()),
                    self.playfield.note_size(),
                    Some(Border::new(Color::RED, self.playfield.note_border_width))
                )));
            }
        }

        // draw notes
        for col in self.columns.iter_mut() {
            for note in col.iter_mut() {note.draw(args, list)}
        }
        // draw timing lines
        for tb in self.timing_bars.iter_mut() {list.extend(tb.draw(args))}
    }

    fn skip_intro(&mut self, manager: &mut IngameManager) {
        let y_needed = 0.0;
        let mut time = manager.time();

        loop {
            let mut found = false;
            for col in self.columns.iter_mut() {
                for note in col.iter_mut() {
                    if note.y_at(time) <= y_needed {found = true; break}
                }
            }
            if found {break}
            time += 1.0;
        }

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

    fn combo_bounds(&self) -> Rectangle {
        let window_size = Settings::window_size();
        Rectangle::bounds_only(
            Vector2::new(0.0, window_size.y * (1.0/3.0)),
            Vector2::new(window_size.x, 30.0)
        )
    }

    fn timing_bar_things(&self) -> (Vec<(f32,Color)>, (f32,Color)) {
        (vec![
            (self.hitwindow_100, [0.3411, 0.8901, 0.0745, 1.0].into()),
            (self.hitwindow_300, [0.1960, 0.7372, 0.9058, 1.0].into()),
        ], (self.hitwindow_miss, [0.8549, 0.6823, 0.2745, 1.0].into()))
    }

    
    fn apply_auto(&mut self, settings: &crate::game::BackgroundGameSettings) {
        for c in self.columns.iter_mut() {
            for note in c.iter_mut() {
                note.set_alpha(settings.opacity)
            }
        }
    }
}



// timing bar struct
//TODO: might be able to reduce this to a (time, speed) and just calc pos on draw
#[derive(Clone, Debug)]
struct TimingBar {
    time: f32,
    speed: f32,
    pos: Vector2,
    size: Vector2,

    playfield: Arc<ManiaPlayfieldSettings>
}
impl TimingBar {
    pub fn new(time:f32, width:f64, x:f64, playfield: Arc<ManiaPlayfieldSettings>) -> TimingBar {
        TimingBar {
            time, 
            size: Vector2::new(width, BAR_HEIGHT),
            speed: 1.0,
            pos: Vector2::new(x, 0.0),

            playfield
        }
    }

    pub fn set_sv(&mut self, sv:f32) {
        self.speed = sv;
    }

    pub fn update(&mut self, time:f32) {
        self.pos.y = (self.playfield.hit_y() + self.playfield.note_size().y-self.size.y) - ((self.time - time) * self.speed) as f64;
        // self.pos = HIT_POSITION + Vector2::new(( - BAR_WIDTH / 2.0, -PLAYFIELD_RADIUS);
    }

    fn draw(&mut self, _args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut renderables: Vec<Box<dyn Renderable>> = Vec::new();
        if self.pos.y < 0.0 || self.pos.y > Settings::window_size().y {return renderables}

        renderables.push(Box::new(Rectangle::new(
            BAR_COLOR,
            BAR_DEPTH,
            self.pos,
            self.size,
            None
        )));

        renderables
    }
}



struct ManiaAutoHelper {
    states: Vec<bool>,
    timers: Vec<(f32, bool)>,
}
impl ManiaAutoHelper {
    fn new() -> Self {
        Self {
            states: Vec::new(),
            timers: Vec::new(),
        }
    }

    fn get_keypress(col: usize) -> KeyPress {
        let base_key = KeyPress::Mania1 as u8;
        ((col + base_key as usize) as u8).into()
    }

    fn update(&mut self, columns: &Vec<Vec<Box<dyn ManiaHitObject>>>, column_indices: &mut Vec<usize>, time: f32, list: &mut Vec<ReplayFrame>) {
        if self.states.len() != columns.len() {
            let new_len = columns.len();
            self.states.resize(new_len, false);
            self.timers.resize(new_len, (0.0, false));
            // self.notes_hit.resize(new_len, Vec::new());
        }

        for c in 0..columns.len() {
            let timer = &mut self.timers[c];
            if time > timer.0 && timer.1 {
                list.push(ReplayFrame::Release(Self::get_keypress(c)));
                timer.1 = false;
            }

            if column_indices[c] >= columns[c].len() {continue}
            for i in column_indices[c]..columns[c].len() {
                let note = &columns[c][i];
                if time > note.end_time(30.0) && !note.was_hit() {
                    column_indices[c] += 1;
                } else {
                    break;
                }
            }

            if column_indices[c] >= columns[c].len() {continue}
            let note = &columns[c][column_indices[c]];
            if time >= note.time() && !note.was_hit() {
                if timer.0 == note.end_time(50.0) && timer.1 {continue}

                list.push(ReplayFrame::Press(Self::get_keypress(c)));
                timer.0 = note.end_time(50.0);
                timer.1 = true;
            }
        }
    }
}