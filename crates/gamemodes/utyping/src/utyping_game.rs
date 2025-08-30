/*
 * UTyping game mode
 * Author: ayyEve
*/

use crate::prelude::*;

/// how many beats between timing bars
const BAR_SPACING:f32 = 4.0;

/// bc sv is bonked, divide it by this amount
const SV_FACTOR:f32 = 700.0;

#[derive(Default)]
pub struct UTypingGame {
    // lists
    pub notes: UTypingNoteQueue,
    #[cfg(feature="graphics")] timing_bars: Vec<UTypingTimingBar>,

    // hit timing bar stuff
    hitwindow_300: f32,
    hitwindow_100: f32,
    hitwindow_miss: f32,

    end_time: f32,
    // auto_helper: UTypingAutoHelper,

    game_settings: Arc<TaikoSettings>,
    playfield: Arc<UTypingPlayfield>,

    autoplay_queue: Option<(Vec<char>, f32, f32)>
}
impl UTypingGame {
    pub fn get_playfield(
        settings: &TaikoSettings, 
        bounds: Bounds, 
        full_window: bool
    ) -> UTypingPlayfield {
        let half_note_width = settings.note_radius * settings.big_note_multiplier;
        let height = half_note_width * 2.0 + settings.playfield_height_padding;

        let mut x_offset = settings.playfield_x_offset;
        let mut y_offset = settings.playfield_y_offset;
        // if not fullscreen, remove the x and y offsets
        if !full_window {
            x_offset = 0.0;
            y_offset = 0.0;
        }


        // load hit_position
        let base = if settings.hit_position_relative_to_window_size {
            bounds.size - Vector2::new(bounds.size.x, bounds.size.y / settings.hit_position_relative_height_div) 
        } else { Vector2::ZERO };

        let hit_position = bounds.pos + base + Vector2::new(x_offset + half_note_width, y_offset);

        UTypingPlayfield {
            bounds,
            height,
            hit_position
        }
    } 

    pub fn update_playfield(&mut self, bounds: Bounds, full_window: bool) {
        self.playfield = Arc::new(Self::get_playfield(&self.game_settings, bounds, full_window));

        // update notes
        self.notes.iter_mut().for_each(|n| n.update_playfield(self.playfield.clone()));

        // update timing bars
        #[cfg(feature="graphics")] 
        self.timing_bars.iter_mut().for_each(|n| n.update_playfield(self.playfield.clone()));
    }
}
impl GameMode for UTypingGame {
    fn new(beatmap: &Beatmap, _:bool, settings: &Settings) -> TatakuResult<Self> {
        // let settings = Arc::new(settings.taiko_settings.clone());
        let settings = Arc::new(settings.gamemode_settings(GAME_INFO).unwrap_or_default());
        let playfield = Arc::new(Self::get_playfield(&settings, Bounds::new(Vector2::ZERO, Vector2::new(1920.0, 1080.0)), false));

        let mut s = Self {
            notes: UTypingNoteQueue::default(),

            game_settings: settings.clone(),
            playfield: playfield.clone(),

            ..Self::default()
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
                        playfield.clone(),
                    ));
                }


            }
            Beatmap::PTyping(beatmap) => {
                for note in beatmap.def.hit_objects.iter() {
                    // info!("adding {} at {}", note.text, note.time);
                    s.notes.push(UTypingNote::new(
                        note.time as f32, 
                        note.text.clone(), 
                        settings.clone(), 
                        playfield.clone(),
                    ));
                }
            }
            _ => return Err(BeatmapError::UnsupportedMode.into()),
        }

        
        if s.notes.is_empty() { return Err(TatakuError::Beatmap(BeatmapError::InvalidFile)); }
        s.notes.sort_by(|a, b|a.time().partial_cmp(&b.time()).unwrap());
        s.end_time = s.notes.iter().last().unwrap().time();

        Ok(s)
    }

    fn handle_replay_frame(
        &mut self, 
        frame: ReplayFrame,
        state: &mut GameplayUpdateShell
    ) {
        // utyping uses chars for input, so we encode it in the mouse pos
        let ReplayAction::MousePos(c, _) = &frame.action else { return };
        // c is actually a u8 encoded as an f32
        let input_char = (*c as u8) as char;


        // let mut hit_volume = Settings::get().get_effect_vol() * (manager.current_timing_point().volume as f32 / 100.0);
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

        if let Some(judgment) = self.notes.check(input_char, frame.time, &hit_windows, state) {
            state.add_judgment(judgment);
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

    fn handle_gameplay_event(&mut self, event: GameplayEvent) {
        match event {
            GameplayEvent::SetBounds { 
                bounds, 
                full_window 
            } => {
                self.update_playfield(bounds, full_window);
            }
            GameplayEvent::ApplyMods(_) => {}
            _ => {}
        }
    }

    fn update(
        &mut self, 
        state: &mut GameplayUpdateShell
    ) {
        // do autoplay things
        if state.mods.has_autoplay() {
            let mut next_note_time = self.notes
                .next_note()
                .map_or(0.0, |n| n.time());


            if let Some((queue, delay, last_hit)) = &mut self.autoplay_queue {
                if state.time - *last_hit > *delay {
                    *last_hit = state.time;

                    let char = queue.remove(0);
                    state.add_replay_action(ReplayAction::MousePos(char as u8 as f32, 0.0));
                }

                if queue.is_empty() {
                    self.autoplay_queue = None;
                }
            } else if let Some(current_note) = self.notes.current_note() {
                if current_note.time() <= state.time {
                    let chars = current_note.get_chars();
                    let len = (chars.len() * 2 + 1) as f32;

                    if next_note_time == 0.0 { next_note_time = current_note.time() + 500.0; }
                    let delay = (next_note_time - current_note.time()) / len;

                    self.autoplay_queue = Some((chars, delay, state.time - delay));
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
            if next_note.time() <= state.time {
                // force miss the current note
                self.notes.current_note().unwrap().miss(state.time);

                // add a miss judgment
                state.add_judgment(UTypingHitJudgment::Miss);

                // increment the note index
                self.notes.next();
            }
        }
        

        // update notes
        for note in self.notes.iter_mut() { note.update(state.time) }

        // if theres no more notes to hit, show score screen
        if let Some(note) = self.notes.last() {
            if state.time > note.end_time(self.hitwindow_miss) && note.was_hit() {
                if !state.complete() {
                    state.add_action(GamemodeAction::MapComplete);
                    // manager.completed = true;
                }
                return;
            }
        }
        
        // TODO: might move tbs to a (time, speed) tuple
        #[cfg(feature="graphics")] 
        for tb in self.timing_bars.iter_mut() { tb.update(state.time); }
    }

    #[cfg(feature="graphics")] 
    fn draw(&mut self, state: GameplayDrawShell, list: &mut RenderableCollection) {

        // draw the playfield
        list.push(self.playfield.get_rectangle(state.current_timing_point.kiai));

        // draw the hit area
        list.push(Circle::new(
            self.playfield.hit_position,
            self.game_settings.note_radius * self.game_settings.hit_area_radius_mult,
            Color::BLACK,
        ));

        // draw timing lines
        for tb in self.timing_bars.iter_mut() { tb.draw(state.time, list); }
        
        // draw notes
        for note in self.notes.iter_mut() { note.draw(state.time, list); }
    }


    fn reset(&mut self, beatmap: &Beatmap) {
        #[cfg(feature="graphics")] 
        let timing_points = TimingPointHelper::new(beatmap.get_timing_points(), beatmap.slider_velocity());
        
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
        #[cfg(feature="graphics")] 
        if self.timing_bars.is_empty() {
            // load timing bars
            let parent_tps = timing_points.iter().filter(|t|!t.is_inherited()).collect::<Vec<&TimingPoint>>();
            let mut sv; // = self.game_settings.sv_multiplier;
            let mut time = parent_tps[0].time;
            let mut tp_index = 0;
            let step = timing_points.beat_length_at(time, false);
            time %= step; // get the earliest bar line possible

            loop {
                // if !self.game_settings.static_sv {
                    sv = (timing_points.slider_velocity_at(time) / SV_FACTOR) * self.game_settings.sv_multiplier;
                // }

                // if theres a bpm change, adjust the current time to that of the bpm change
                let next_bar_time = timing_points.beat_length_at(time, false) * BAR_SPACING; // bar spacing is actually the timing point measure

                // edge case for aspire maps
                if next_bar_time.is_nan() || next_bar_time == 0.0 {
                    break;
                }

                // add timing bar at current time
                self.timing_bars.push(UTypingTimingBar::new(time, sv, self.playfield.clone()));

                if tp_index < parent_tps.len() && parent_tps[tp_index].time <= time + next_bar_time {
                    time = parent_tps[tp_index].time;
                    tp_index += 1;
                    continue;
                }

                // why isnt this accounting for bpm changes? because the bpm change doesnt allways happen inline with the bar idiot
                time += next_bar_time;
                if time >= self.end_time || time.is_nan() { break }
            }

            trace!("created {} timing bars", self.timing_bars.len());
        }
        
        // reset hitcache times
        // self.hit_cache.iter_mut().for_each(|(_, t)| *t = -999.9);
    }


    #[cfg(feature="gameplay")] 
    fn skip_intro(&mut self, game_time: f32) -> Option<f32> {
        // if self.note_index > 0 {return}

        let x_needed = self.playfield.bounds.size.x;
        let mut time = self.end_time; //manager.time();

        for i in self.notes.iter().rev() {
            let time_at = i.time_at(x_needed);
            time = time.min(time_at);
        }
        // loop {
        //     let mut found = false;
        //     for note in self.notes.iter() {if note.x_at(time) <= x_needed {found = true; break}}
        //     if found {break}
        //     time += 1.0;
        // }

        if game_time >= time { return None }

        // if manager.lead_in_time > 0.0 {
        //     if time > manager.lead_in_time {
        //         time -= manager.lead_in_time - 0.01;
        //         manager.lead_in_time = 0.01;
        //     }
        // }
        
        if time < 0.0 { return None }
        Some(time)
    }

    fn force_update_settings(&mut self, _settings: &Settings) {}
    
    #[cfg(feature="graphics")]
    fn reload_skin(&mut self, _beatmap_path: &str, skin_manager: &mut dyn SkinProvider) -> TextureSource {
        for i in self.notes.iter_mut() {
            i.reload_skin(&TextureSource::Skin, skin_manager);
        }
        TextureSource::Skin
    }

    #[cfg(feature="graphics")] 
    fn get_playfield(&self) -> PlayfieldNonsense {
        PlayfieldNonsense::new_simple(self.playfield.bounds)
    }
    fn properties(&self, _: &TimingPointHelper) -> GameModeProperties {
        GameModeProperties { 
            info: &crate::GAME_INFO, 
            keys: Vec::new(), 
            end_time: self.end_time, 
            show_cursor: false, 
            audio_prefix: String::new(),
            timing_bar_things: vec![
                (self.hitwindow_100,  Color::new(0.3411, 0.8901, 0.0745, 1.0)),
                (self.hitwindow_300,  Color::new(0.1960, 0.7372, 0.9058, 1.0)),
                (self.hitwindow_miss, Color::new(0.8549, 0.6823, 0.2745, 1.0))
            ], 
            sound_list: Vec::new(),
        }
    }


    
    #[cfg(feature="gameplay")] 
    fn handle_input(&mut self, input: InputEvent) -> Option<ReplayAction> {
        match input.event {
            InputType::KeyPress(key) => {
                let text = key.text?;
                let c = text.chars().next()?;
                Some(ReplayAction::MousePos(c as u8 as f32, 0.0))
            }

            _ => None
        }
    }
}
