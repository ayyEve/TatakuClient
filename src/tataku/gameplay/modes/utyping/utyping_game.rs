/**
 * UTyping game mode
 * Author: ayyEve
 * 
 * depths:
 *  notes: 0..1000
 *  hit area: 1001
 *  timing bars: 1001.5
 *  playfield: 1002
 *  hit indicators: -1
 *  judgement indicators: -2
*/

use crate::prelude::*;
use super::prelude::*;

/// how many beats between timing bars
const BAR_SPACING:f32 = 4.0;

/// bc sv is bonked, divide it by this amount
const SV_FACTOR:f32 = 700.0;
const NOTE_DEPTH_RANGE:Range<f64> = 0.0..1000.0;

pub struct UTypingGame {
    // lists
    pub notes: UTypingNoteQueue,
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
    #[inline]
    pub fn get_depth(time: f32) -> f64 {
        NOTE_DEPTH_RANGE.start + (NOTE_DEPTH_RANGE.end - NOTE_DEPTH_RANGE.end / time as f64)
    }
    // pub fn next_note(&mut self) {self.note_index += 1}
}

#[async_trait]
impl GameMode for UTypingGame {
    async fn new(beatmap:&Beatmap, diff_calc_only:bool) -> TatakuResult<Self> {
        let mut settings = get_settings!().taiko_settings.clone();
        // calculate the hit area
        settings.init_settings().await;
        let settings = Arc::new(settings);
        let mut s = Self {
            notes: UTypingNoteQueue::new(),

            timing_bars: Vec::new(),
            end_time: 0.0,

            hitwindow_100: 0.0,
            hitwindow_300: 0.0,
            hitwindow_miss: 0.0,

            // auto_helper: UTypingAutoHelper::new(),
            game_settings: settings.clone(),
        };

        match beatmap {
            Beatmap::UTyping(beatmap) => {

                for note in beatmap.notes.iter() {
                    let time = note.time;
                    // let mut cutoff_time = 0.0;

                    // for event in beatmap.events.iter() {
                    //     if event.event_type == UTypingEventType::CutOff && event.time > time {
                    //         cutoff_time = time;
                    //         break;
                    //     }
                    // }

                    // info!("adding {} at {time}", note.text);
                    s.notes.push(UTypingNote::new(
                        time, 
                        note.text.clone(), 
                        settings.clone(), 
                        diff_calc_only
                    ).await);
                }


            }
            Beatmap::PTyping(beatmap) => {
                for note in beatmap.def.hit_objects.iter() {
                    // info!("adding {} at {}", note.text, note.time);
                    s.notes.push(UTypingNote::new(
                        note.time as f32, 
                        note.text.clone(), 
                        settings.clone(), 
                        diff_calc_only
                    ).await);
                }
            }
            _ => return Err(BeatmapError::UnsupportedMode.into()),
        }

        
        if s.notes.len() == 0 { return Err(TatakuError::Beatmap(BeatmapError::InvalidFile)); }
        s.notes.sort_by(|a, b|a.time().partial_cmp(&b.time()).unwrap());
        s.end_time = s.notes.iter().last().unwrap().time();

        Ok(s)
    }

    async fn handle_replay_frame(&mut self, frame:ReplayFrame, time:f32, manager:&mut IngameManager) {
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

        let hit_windows = vec![
            (UTypingHitJudgment::X300, 0.0..self.hitwindow_300),
            (UTypingHitJudgment::X100, self.hitwindow_300..self.hitwindow_100), 
            (UTypingHitJudgment::Miss, self.hitwindow_100..self.hitwindow_miss), 
        ];

        let time = manager.time();
        if let Some(judgment) = self.notes.check(input_char, time, &hit_windows, manager) {
            manager.add_judgment(&judgment).await;
        }

        // // draw drum
        // match key {
        //     KeyPress::LeftKat => *self.hit_cache.get_mut(&TaikoHit::LeftKat).unwrap() = time,
        //     KeyPress::LeftDon => *self.hit_cache.get_mut(&TaikoHit::LeftDon).unwrap() = time,
        //     KeyPress::RightDon => *self.hit_cache.get_mut(&TaikoHit::RightDon).unwrap() = time,
        //     KeyPress::RightKat => *self.hit_cache.get_mut(&TaikoHit::RightKat).unwrap() = time,
        //     _=> {}
        // }

    }


    async fn update(&mut self, manager:&mut IngameManager, time: f32) -> Vec<ReplayFrame> {
        let mut autoplay_list = Vec::new();

        // do autoplay things
        if manager.current_mods.has_autoplay() {

            if let Some(current_note) = self.notes.current_note() {
                if current_note.time() <= time {
                    for c in current_note.get_chars() {
                        autoplay_list.push(ReplayFrame::MousePos(c as u8 as f32, 0.0))
                    }
                }
            }
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

        // check missed notes
        if let Some(next_note) = self.notes.next_note() {
            // if its time to hit the next note
            if next_note.time() <= time {
                // force miss the current note
                self.notes.current_note().unwrap().miss(time);

                // add a miss judgment
                manager.add_judgment(&UTypingHitJudgment::Miss).await;

                // increment the note index
                self.notes.next();
            }
        }
        

        // update notes
        for note in self.notes.iter_mut() { note.update(time).await }

        // if theres no more notes to hit, show score screen
        if let Some(note) = self.notes.last() {
            if time > note.end_time(self.hitwindow_miss) && note.was_hit() {
                manager.completed = true;
                return autoplay_list;
            }
        }
        
        // TODO: might move tbs to a (time, speed) tuple
        for tb in self.timing_bars.iter_mut() { tb.update(time); }

        autoplay_list
    }
    async fn draw(&mut self, args:RenderArgs, manager:&mut IngameManager, list: &mut RenderableCollection) {

        // draw the playfield
        list.push(self.game_settings.get_playfield(args.window_size[0], manager.current_timing_point().kiai));

        // draw the hit area
        list.push(Circle::new(
            Color::BLACK,
            1001.0,
            self.game_settings.hit_position,
            self.game_settings.note_radius * self.game_settings.hit_area_radius_mult,
            None
        ));

        // draw notes
        for note in self.notes.iter_mut() { note.draw(args, list).await; }

        // draw timing lines
        for tb in self.timing_bars.iter_mut() { tb.draw(args, list); }
    }


    async fn reset(&mut self, beatmap:&Beatmap) {
        for note in self.notes.iter_mut() {
            note.reset().await;

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
            let mut sv; // = self.game_settings.sv_multiplier;
            let mut time = parent_tps[0].time;
            let mut tp_index = 0;
            let step = beatmap.beat_length_at(time, false);
            time %= step; // get the earliest bar line possible

            loop {
                // if !self.game_settings.static_sv {
                    sv = (beatmap.slider_velocity_at(time) / SV_FACTOR) * self.game_settings.sv_multiplier;
                // }

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

        let x_needed = WindowSize::get().x as f32;
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

        if manager.time() >= time { return }

        if manager.lead_in_time > 0.0 {
            if time > manager.lead_in_time {
                time -= manager.lead_in_time - 0.01;
                manager.lead_in_time = 0.01;
            }
        }
        
        if time < 0.0 { return }
        manager.song.set_position(time);
    }


    
    async fn window_size_changed(&mut self, _window_size: Arc<WindowSize>) {}
    async fn fit_to_area(&mut self, _pos: Vector2, _size: Vector2) {}

    
    async fn force_update_settings(&mut self, _settings: &Settings) {}
    async fn reload_skin(&mut self) {}

    async fn apply_mods(&mut self, _mods: Arc<ModManager>) {}
}

#[async_trait]
impl GameModeInput for UTypingGame {
    async fn key_down(&mut self, _key:Key) -> Option<ReplayFrame> { None }
    
    async fn key_up(&mut self, _key:piston::Key) -> Option<ReplayFrame> { None }

    async fn on_text(&mut self, text: &String, _mods: &KeyModifiers) -> Option<ReplayFrame> {
        if let Some(c) = text.chars().next() {
            Some(ReplayFrame::MousePos(c as u8 as f32, 0.0))
        } else {
            None
        }
    }
}

impl GameModeProperties for UTypingGame {
    fn playmode(&self) -> PlayMode {"utyping".to_owned()}
    fn end_time(&self) -> f32 {self.end_time}

    fn get_possible_keys(&self) -> Vec<(KeyPress, &str)> {Vec::new()}

    fn timing_bar_things(&self) -> Vec<(f32,Color)> {
        vec![
            (self.hitwindow_100, [0.3411, 0.8901, 0.0745, 1.0].into()),
            (self.hitwindow_300, [0.1960, 0.7372, 0.9058, 1.0].into()),
            (self.hitwindow_miss, [0.8549, 0.6823, 0.2745, 1.0].into())
        ]
    }
    
}
