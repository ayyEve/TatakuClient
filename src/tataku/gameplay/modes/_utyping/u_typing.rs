use super::*;
use crate::prelude::*;

/// timing bar color
pub const BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0);
/// how wide is a timing bar
const BAR_WIDTH:f64 = 4.0;
/// how many beats between timing bars
const BAR_SPACING:f32 = 4.0;

/// bc sv is bonked, divide it by this amount
const SV_FACTOR:f32 = 700.0;

/// calculate the taiko acc for `score`
pub fn calc_acc(score: &Score) -> f64 {
    // let x50 = score.x50 as f64;
    let x100 = score.x100 as f64;
    let x300 = score.x300 as f64;
    let geki = score.xgeki as f64;
    let katu = score.xkatu as f64;
    let miss = score.xmiss as f64;

    ((x100 + katu) / 2.0 + x300 + geki) 
    / (miss + x100 + x300 + katu + geki)
}

pub struct UTypingGame {
    // lists
    pub notes: Vec<UTypingNote>,
    timing_bars: Vec<TimingBar>,

    // hit timing bar stuff
    hitwindow_300: f32,
    hitwindow_100: f32,
    hitwindow_miss: f32,

    end_time: f32,
    // auto_helper: UTypingAutoHelper,

    game_settings: Arc<TaikoSettings>,
}
impl UTypingGame {
    // pub fn next_note(&mut self) {self.note_index += 1}
}

#[async_trait]
impl GameMode for UTypingGame {
    async fn new(beatmap:&Beatmap, diff_calc_only:bool) -> Result<Self, crate::errors::TatakuError> {
        let mut settings = get_settings!().taiko_settings.clone();
        // calculate the hit area
        settings.init_settings();
        let settings = Arc::new(settings);

        match beatmap {
            Beatmap::UTyping(beatmap) => {
                let mut s = Self {
                    notes: Vec::new(),

                    timing_bars: Vec::new(),
                    end_time: 0.0,

                    hitwindow_100: 0.0,
                    hitwindow_300: 0.0,
                    hitwindow_miss: 0.0,

                    // auto_helper: UTypingAutoHelper::new(),
                    game_settings: settings.clone(),
                };

                for note in beatmap.notes.iter() {
                    let time = note.time;
                    let mut cutoff_time = 0.0;

                    for event in beatmap.events.iter() {
                        if event.event_type == crate::beatmaps::u_typing::UTypingEventType::CutOff && event.time > time {
                            cutoff_time = time;
                            break;
                        }
                    }

                    s.notes.push(UTypingNote::new(time, note.text.clone(), cutoff_time, settings.clone(), diff_calc_only).await);
                }


                if s.notes.len() == 0 {return Err(TatakuError::Beatmap(BeatmapError::InvalidFile))}
                s.notes.sort_by(|a, b|a.time().partial_cmp(&b.time()).unwrap());
                s.end_time = s.notes.iter().last().unwrap().time();

                Ok(s)
            }

            _ => Err(BeatmapError::UnsupportedMode.into()),
        }
    }

    fn handle_replay_frame(&mut self, frame:ReplayFrame, time:f32, manager:&mut IngameManager) {
        if !manager.replaying {
            manager.replay.frames.push((time, frame.clone()));
            manager.outgoing_spectator_frame((time, SpectatorFrameData::ReplayFrame{frame}));
        }

        let input_char;
        // utyping uses chars for input, so we encode it in the mouse pos
        if let ReplayFrame::MousePos(c, _) = &frame {
            // c is actually a u8 encoded as an f32
            input_char = (*c as u8) as char
        } else {
            return;
        }


        // let mut hit_volume = get_settings!().get_effect_vol() * (manager.current_timing_point().volume as f32 / 100.0);
        // if manager.menu_background {
        //     hit_volume *= manager.background_game_settings.hitsound_volume;
        // }

        // // if theres no more notes to hit, return after playing the sound
        // if self.note_index >= self.notes.len() {
        //     return;
        // }


        let time = manager.time();
        for note in self.notes.iter_mut() {
            let note_time = note.time();

            match note.get_points(time, input_char, vec![self.hitwindow_miss, self.hitwindow_100, self.hitwindow_300]) {
                ScoreHit::None => continue,
                
                ScoreHit::Miss => {
                    manager.score.hit_miss(time, note_time);
                    manager.hitbar_timings.push((time, time - note_time));
                    manager.combo_break();

                    manager.health.take_damage();
                    if manager.health.is_dead() {
                        manager.fail()
                    }

                    //TODO: indicate this was a miss
                    break;
                }
                ScoreHit::X50 => {
                    manager.score.hit50(time, note_time);
                    manager.hitbar_timings.push((time, time - note_time));

                    manager.health.take_damage();

                    //TODO: indicate this was a bad hit
                    break;
                }
                ScoreHit::X100 | ScoreHit::Xkatu => {
                    manager.score.hit100(time, note_time);
                    manager.hitbar_timings.push((time, time - note_time));

                    manager.health.give_life();

                    //TODO: indicate this was a bad hit
                    break;
                }
                ScoreHit::X300 | ScoreHit::Xgeki => {
                    manager.score.hit300(time, note_time);
                    manager.hitbar_timings.push((time, time - note_time));
                    manager.health.give_extra_life();
                    
                    break;
                }
                ScoreHit::Other(score, _consume) => {
                    manager.score.score.score += score as u64;
                    // if consume {self.next_note()}
                    break;
                }
            }
        }

        

        // // draw drum
        // match key {
        //     KeyPress::LeftKat => *self.hit_cache.get_mut(&TaikoHit::LeftKat).unwrap() = time,
        //     KeyPress::LeftDon => *self.hit_cache.get_mut(&TaikoHit::LeftDon).unwrap() = time,
        //     KeyPress::RightDon => *self.hit_cache.get_mut(&TaikoHit::RightDon).unwrap() = time,
        //     KeyPress::RightKat => *self.hit_cache.get_mut(&TaikoHit::RightKat).unwrap() = time,
        //     _=> {}
        // }


        // play sound
    }


    async fn update(&mut self, manager:&mut IngameManager, time: f32) {

        // do autoplay things
        if manager.current_mods.autoplay {
            // let mut pending_frames = Vec::new();
            // let notes = &mut self.notes;

            // // get auto inputs
            // self.auto_helper.update(time, notes, &mut pending_frames);

            // // update index
            // for i in 0..notes.len() {
            //     self.note_index = i;
            //     if (!notes[i].was_hit() && notes[i].note_type() != NoteType::Slider) || (notes[i].note_type() == NoteType::Slider && notes[i].end_time(0.0) > time) {
            //         break;
            //     }
            // }

            // for frame in pending_frames.iter() {
            //     self.handle_replay_frame(*frame, time, manager);
            // }
        }

        // update notes
        for note in self.notes.iter_mut() {note.update(time).await}

        // if theres no more notes to hit, show score screen
        if let Some(note) = self.notes.last() {
            if time > note.end_time(self.hitwindow_miss) {
                manager.completed = true;
                return;
            }
        }

        // check if we missed the current note
        for note in self.notes.iter_mut() {
            if note.check_missed(time, self.hitwindow_miss) {
                let s = &mut manager.score;
                s.xmiss += 1;
                s.combo = 0;

                manager.health.take_damage();
                if manager.health.is_dead() {
                    manager.fail()
                }
            }
        }
        
        // TODO: might move tbs to a (time, speed) tuple
        for tb in self.timing_bars.iter_mut() {tb.update(time)}
    }
    async fn draw(&mut self, args:RenderArgs, manager:&mut IngameManager, list:&mut Vec<Box<dyn Renderable>>) {
        // let time = manager.time();
        // let lifetime_time = DRUM_LIFETIME_TIME * manager.game_speed();
        
        // for (hit_type, hit_time) in self.hit_cache.iter() {
        //     if time - hit_time > lifetime_time {continue}
        //     let alpha = 1.0 - (time - hit_time) / (lifetime_time * 4.0);

        //     match hit_type {
        //         TaikoHit::LeftKat => {
        //             if let Some(kat) = &self.left_kat_image {
        //                 let mut img = kat.clone();
        //                 img.current_color.a = alpha;
        //                 list.push(Box::new(img));
        //             } else {
        //                 list.push(Box::new(HalfCircle::new(
        //                     self.taiko_settings.kat_color.alpha(alpha),
        //                     self.taiko_settings.hit_position,
        //                     1.0,
        //                     self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult,
        //                     true
        //                 )));
        //             }
        //         }
        //         TaikoHit::LeftDon => {
        //             if let Some(don) = &self.left_don_image {
        //                 let mut img = don.clone();
        //                 img.current_color.a = alpha;
        //                 list.push(Box::new(img));
        //             } else {
        //                 list.push(Box::new(HalfCircle::new(
        //                     self.taiko_settings.don_color.alpha(alpha),
        //                     self.taiko_settings.hit_position,
        //                     1.0,
        //                     self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult,
        //                     true
        //                 )));
        //             }
        //         }
        //         TaikoHit::RightDon => {
        //             if let Some(don) = &self.right_don_image {
        //                 let mut img = don.clone();
        //                 img.current_color.a = alpha;
        //                 list.push(Box::new(img));
        //             } else {
        //                 list.push(Box::new(HalfCircle::new(
        //                     self.taiko_settings.don_color.alpha(alpha),
        //                     self.taiko_settings.hit_position,
        //                     1.0,
        //                     self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult,
        //                     false
        //                 )));
        //             }
        //         }
        //         TaikoHit::RightKat => {
        //             if let Some(kat) = &self.right_kat_image {
        //                 let mut img = kat.clone();
        //                 img.current_color.a = alpha;
        //                 list.push(Box::new(img));
        //             } else {
        //                 list.push(Box::new(HalfCircle::new(
        //                     self.taiko_settings.kat_color.alpha(alpha),
        //                     self.taiko_settings.hit_position,
        //                     1.0,
        //                     self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult,
        //                     false
        //                 )));
        //             }
        //         }
        //     }
        // }

        // draw the playfield
        list.push(Box::new(self.game_settings.get_playfield(args.window_size[0], manager.current_timing_point().kiai)));

        // draw the hit area
        list.push(Box::new(Circle::new(
            Color::BLACK,
            f64::MAX,
            self.game_settings.hit_position,
            self.game_settings.note_radius * self.game_settings.hit_area_radius_mult,
            None
        )));

        // draw notes
        for note in self.notes.iter_mut() {list.extend(note.draw(args).await)}
        // draw timing lines
        for tb in self.timing_bars.iter_mut() {tb.draw(args, list)}
    }


    async fn reset(&mut self, beatmap:&Beatmap) {
        for note in self.notes.iter_mut() {
            note.reset();

            // set note svs
            // if self.game_settings.static_sv {
            //     note.set_sv(self.game_settings.sv_multiplier);
            // } else {
            //     let sv = (beatmap.slider_velocity_at(note.time()) / SV_FACTOR) * self.game_settings.sv_multiplier;
            //     note.set_sv(sv);
            // }
        }
        
        // TODO: use proper values lol
        let od = 0.0; //beatmap.get_beatmap_meta().od;
        // setup hitwindows
        self.hitwindow_miss = map_difficulty(od, 135.0, 95.0, 70.0);
        self.hitwindow_100 = map_difficulty(od, 120.0, 80.0, 50.0);
        self.hitwindow_300 = map_difficulty(od, 50.0, 35.0, 20.0);

        // setup timing bars
        //TODO: it would be cool if we didnt actually need timing bar objects, and could just draw them
        if self.timing_bars.len() == 0 {
            let tps = beatmap.get_timing_points();
            // load timing bars
            let parent_tps = tps.iter().filter(|t|!t.is_inherited()).collect::<Vec<&TimingPoint>>();
            let mut sv = self.game_settings.sv_multiplier;
            let mut time = parent_tps[0].time;
            let mut tp_index = 0;
            let step = beatmap.beat_length_at(time, false);
            time %= step; // get the earliest bar line possible

            loop {
                if !self.game_settings.static_sv {sv = (beatmap.slider_velocity_at(time) / SV_FACTOR) * self.game_settings.sv_multiplier}

                // if theres a bpm change, adjust the current time to that of the bpm change
                let next_bar_time = beatmap.beat_length_at(time, false) * BAR_SPACING; // bar spacing is actually the timing point measure

                // edge case for aspire maps
                if next_bar_time.is_nan() || next_bar_time == 0.0 {
                    break;
                }

                // add timing bar at current time
                self.timing_bars.push(TimingBar::new(time, sv, self.game_settings.clone()));

                if tp_index < parent_tps.len() && parent_tps[tp_index].time <= time + next_bar_time {
                    time = parent_tps[tp_index].time;
                    tp_index += 1;
                    continue;
                }

                // why isnt this accounting for bpm changes? because the bpm change doesnt allways happen inline with the bar idiot
                time += next_bar_time;
                if time >= self.end_time || time.is_nan() {break}
            }

            trace!("created {} timing bars", self.timing_bars.len());
        }
        
        // reset hitcache times
        // self.hit_cache.iter_mut().for_each(|(_, t)| *t = -999.9);
    }



    fn skip_intro(&mut self, manager: &mut IngameManager) {
        // if self.note_index > 0 {return}

        let x_needed = Settings::window_size().x as f32;
        let mut time = self.end_time; //manager.time();

        for i in self.notes.iter().rev() {
            let time_at = i.time_at(x_needed);
            time = time.min(time_at)
        }
        // loop {
        //     let mut found = false;
        //     for note in self.notes.iter() {if note.x_at(time) <= x_needed {found = true; break}}
        //     if found {break}
        //     time += 1.0;
        // }

        if manager.time() >= time {return}

        if manager.lead_in_time > 0.0 {
            if time > manager.lead_in_time {
                time -= manager.lead_in_time - 0.01;
                manager.lead_in_time = 0.01;
            }
        }
        
        if time < 0.0 {return}
        manager.song.set_position(time as f64);
    }

    fn apply_auto(&mut self, _settings: &BackgroundGameSettings) {
        // for note in self.notes.iter_mut() {
        //     note.set_alpha(settings.opacity)
        // }
    }

}

impl GameModeInput for UTypingGame {

    fn key_down(&mut self, _key:piston::Key, _manager:&mut IngameManager) {}
    
    fn key_up(&mut self, _key:piston::Key, _manager:&mut IngameManager) {}

    fn on_text(&mut self, text: &String, _mods: &KeyModifiers, manager: &mut IngameManager) {
        let time = manager.time();
        for c in text.chars() {
            let frame = ReplayFrame::MousePos(c as u8 as f32, 0.0);
            self.handle_replay_frame(frame, time, manager)
        }
    }

}

impl GameModeInfo for UTypingGame {
    fn playmode(&self) -> PlayMode {"utyping".to_owned()}
    fn end_time(&self) -> f32 {self.end_time}

    fn get_possible_keys(&self) -> Vec<(KeyPress, &str)> {Vec::new()}

    fn score_hit_string(hit:&ScoreHit) -> String where Self: Sized {
        match hit {
            ScoreHit::Miss  => "Miss".to_owned(),
            ScoreHit::X100  => "x100".to_owned(),
            ScoreHit::X300  => "x300".to_owned(),
            ScoreHit::Xgeki => "Geki".to_owned(),
            ScoreHit::Xkatu => "Katu".to_owned(),
            
            ScoreHit::X50   => String::new(),
            ScoreHit::None  => String::new(),
            ScoreHit::Other(_, _) => String::new(),
        }
    }

    fn timing_bar_things(&self) -> (Vec<(f32,Color)>, (f32,Color)) {
        (vec![
            (self.hitwindow_100, [0.3411, 0.8901, 0.0745, 1.0].into()),
            (self.hitwindow_300, [0.1960, 0.7372, 0.9058, 1.0].into()),
        ], (self.hitwindow_miss, [0.8549, 0.6823, 0.2745, 1.0].into()))
    }
    
}

// timing bar struct
//TODO: might be able to reduce this to a (time, speed) and just calc pos on draw
#[derive(Clone)]
struct TimingBar {
    time: f32,
    speed: f32,
    pos: Vector2,
    settings: Arc<TaikoSettings>,
    size: Vector2
}
impl TimingBar {
    pub fn new(time:f32, speed:f32, settings: Arc<TaikoSettings>) -> TimingBar {
        let size = Vector2::new(BAR_WIDTH, settings.get_playfield(0.0, false).size.y);

        TimingBar {
            time, 
            speed,
            pos: Vector2::new(0.0, settings.hit_position.y - size.y/2.0),
            settings,
            size
        }
    }

    pub fn update(&mut self, time:f32) {
        self.pos.x = self.settings.hit_position.x + ((self.time - time) * self.speed) as f64 - BAR_WIDTH / 2.0;
    }

    fn draw(&mut self, args:RenderArgs, list:&mut Vec<Box<dyn Renderable>>){
        if self.pos.x + BAR_WIDTH < 0.0 || self.pos.x - BAR_WIDTH > args.window_size[0] {return}

        const DEPTH:f64 = f64::MAX-5.0;

        list.push(Box::new(Rectangle::new(
            BAR_COLOR,
            DEPTH,
            self.pos,
            self.size,
            None
        )));
    }
}





// struct TaikoAutoHelper {
//     don_presses: u32,
//     kat_presses: u32,

//     current_note_duration: f32,
//     note_index: i64,

//     last_hit: f32,
//     last_update: f32,
// }
// impl TaikoAutoHelper {
//     fn new() -> Self {
//         Self {
//             don_presses: 0, 
//             kat_presses: 0, 
//             note_index: - 1, 
//             last_hit: 0.0, 
//             current_note_duration: 0.0,
//             last_update: 0.0
//             // notes: Vec::new()
//         }
//     }

//     fn update(&mut self, time: f32, notes: &mut Vec<Box<dyn TaikoHitObject>>, frames: &mut Vec<ReplayFrame>) {
//         let catching_up = time - self.last_update > 20.0;
//         self.last_update = time;

//         if catching_up {trace!("catching up")}

//         for i in 0..notes.len() {
//             let note = &mut notes[i];
            
//             if time >= note.time() 
//             // && time <= note.end_time(100.0) 
//             && !note.was_hit() {

//                 // check if we're catching up
//                 if catching_up {
//                     // pretend the note was hit
//                     note.force_hit();
//                     continue;
//                 }

//                 // otherwise it spams sliders even after it has finished
//                 if let NoteType::Slider = note.note_type() {
//                     if time > note.end_time(0.0) {
//                         continue;
//                     }
//                 }

//                 // we're already working on this note
//                 if i as i64 == self.note_index {
//                     match note.note_type() {
//                         NoteType::Slider | NoteType::Spinner => {
//                             let time_between_hits = self.current_note_duration / (note.hits_to_complete() as f32);
                            
//                             // if its not time to do another hit yet
//                             if time - self.last_hit < time_between_hits {return}
//                         }

//                         // nothing to do for notes (they only need 1 hit) and holds dont exist
//                         // dont do anything else for this object
//                         NoteType::Hold | NoteType::Note => continue,
//                     }
//                 } else {
//                     self.note_index = i as i64;
                        
//                     match note.note_type() {
//                         NoteType::Slider | NoteType::Spinner => self.current_note_duration = note.end_time(0.0) - note.time(),
//                         _ => {},
//                     }
//                 }

//                 self.last_hit = time;
//                 // let note_type = note.note_type();
//                 let is_kat = note.is_kat();
//                 let is_finisher = note.finisher_sound();

//                 let count = self.don_presses + self.kat_presses;
//                 let side = count % 2;

//                 if is_finisher {
//                     if is_kat {
//                         frames.push(ReplayFrame::Press(KeyPress::LeftKat));
//                         frames.push(ReplayFrame::Press(KeyPress::RightKat));
//                     } else {
//                         frames.push(ReplayFrame::Press(KeyPress::LeftDon));
//                         frames.push(ReplayFrame::Press(KeyPress::RightDon));
//                     }
//                 } else {
//                     match (is_kat, side) {
//                         // kat, left side
//                         (true, 0) => frames.push(ReplayFrame::Press(KeyPress::LeftKat)),

//                         // kat, right side
//                         (true, 1) => frames.push(ReplayFrame::Press(KeyPress::RightKat)),

//                         // don, left side
//                         (false, 0) => frames.push(ReplayFrame::Press(KeyPress::LeftDon)),
                        
//                         // don, right side
//                         (false, 1) => frames.push(ReplayFrame::Press(KeyPress::RightDon)),

//                         // shouldnt happen
//                         _ => {}
//                     }
//                 }

//                 if is_kat {
//                     self.kat_presses += 1;
//                 } else {
//                     self.don_presses += 1;
//                 }

//                 return
//             }
//         }
//     }
// }

