/**
 * UTyping game mode
 * Author: ayyEve
*/

use crate::prelude::*;
use super::prelude::*;

/// how many beats between timing bars
const BAR_SPACING:f32 = 4.0;

/// bc sv is bonked, divide it by this amount
const SV_FACTOR:f32 = 700.0;

pub struct UTypingGame {
    // lists
    pub notes: UTypingNoteQueue,
    timing_bars: Vec<UTypingTimingBar>,

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
    pub fn get_playfield(settings: &TaikoSettings, bounds: Bounds) -> UTypingPlayfield {
        let half_note_width = settings.note_radius * settings.big_note_multiplier;
        let height = half_note_width * 2.0 + settings.playfield_height_padding;

        let mut x_offset = settings.playfield_x_offset;
        let mut y_offset = settings.playfield_y_offset;
        // if not fullscreen, remove the x and y offsets
        if bounds.size != WindowSize::get().0 {
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

    pub fn update_playfield(&mut self, bounds: Bounds) {
        self.playfield = Arc::new(Self::get_playfield(&self.game_settings, bounds));

        // update notes
        self.notes.iter_mut().for_each(|n|n.update_playfield(self.playfield.clone()));

        // update timing bars
        self.timing_bars.iter_mut().for_each(|n|n.update_playfield(self.playfield.clone()));
    }
}

#[async_trait]
impl GameMode for UTypingGame {
    async fn new(beatmap:&Beatmap, _:bool) -> TatakuResult<Self> {
        let settings = Arc::new(Settings::get().taiko_settings.clone());
        let playfield = Arc::new(Self::get_playfield(&settings, Bounds::new(Vector2::ZERO, WindowSize::get().0)));

        let mut s = Self {
            notes: UTypingNoteQueue::new(),

            timing_bars: Vec::new(),
            end_time: 0.0,

            hitwindow_100: 0.0,
            hitwindow_300: 0.0,
            hitwindow_miss: 0.0,

            // auto_helper: UTypingAutoHelper::new(),
            game_settings: settings.clone(),
            playfield: playfield.clone(),
            autoplay_queue: None
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
                        playfield.clone(),
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

    async fn handle_replay_frame(&mut self, frame:ReplayAction, _time:f32, manager:&mut IngameManager) {
        // if !manager.replaying {
        //     manager.replay.frames.push(ReplayFrame::new(time, frame.clone()));
        //     manager.outgoing_spectator_frame(SpectatorFrame::new(time, SpectatorAction::ReplayAction {action:frame}));
        // }

        let input_char;
        // utyping uses chars for input, so we encode it in the mouse pos
        if let ReplayAction::MousePos(c, _) = &frame {
            // c is actually a u8 encoded as an f32
            input_char = (*c as u8) as char
        } else {
            return;
        }


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


    async fn update(&mut self, manager:&mut IngameManager, time: f32) -> Vec<ReplayAction> {
        let mut autoplay_list = Vec::new();

        // do autoplay things
        if manager.current_mods.has_autoplay() {
            let mut next_note_time = self.notes.next_note().map(|n|n.time()).unwrap_or(0.0);


            if let Some((queue, delay, last_hit)) = &mut self.autoplay_queue {
                if time - *last_hit > *delay {
                    *last_hit = time;

                    let char = queue.remove(0);
                    autoplay_list.push(ReplayAction::MousePos(char as u8 as f32, 0.0));
                }

                if queue.len() == 0 {
                    self.autoplay_queue = None;
                }
            } else {
                if let Some(current_note) = self.notes.current_note() {
                    if current_note.time() <= time {
                        let chars = current_note.get_chars();
                        let len = (chars.len() * 2 + 1) as f32;

                        if next_note_time == 0.0 { next_note_time = current_note.time() + 500.0; }
                        let delay = (next_note_time - current_note.time()) / len;

                        self.autoplay_queue = Some((chars, delay, time - delay));
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
    async fn draw(&mut self, time: f32, manager:&mut IngameManager, list: &mut RenderableCollection) {

        // draw the playfield
        list.push(self.playfield.get_rectangle(manager.current_timing_point().kiai));

        // draw the hit area
        list.push(Circle::new(
            self.playfield.hit_position,
            self.game_settings.note_radius * self.game_settings.hit_area_radius_mult,
            Color::BLACK,
            None
        ));

        // draw timing lines
        for tb in self.timing_bars.iter_mut() { tb.draw(time, list); }
        
        // draw notes
        for note in self.notes.iter_mut() { note.draw(time, list).await; }
    }


    async fn reset(&mut self, beatmap:&Beatmap) {
        let timing_points = TimingPointHelper::new(beatmap.get_timing_points(), beatmap.slider_velocity());
        
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



    fn skip_intro(&mut self, manager: &mut IngameManager) -> Option<f32> {
        // if self.note_index > 0 {return}

        let x_needed = WindowSize::get().x;
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

        if manager.time() >= time { return None }

        if manager.lead_in_time > 0.0 {
            if time > manager.lead_in_time {
                time -= manager.lead_in_time - 0.01;
                manager.lead_in_time = 0.01;
            }
        }
        
        if time < 0.0 { return None }
        Some(time)
    }


    
    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.update_playfield(Bounds::new(Vector2::ZERO, window_size.0));
    }
    async fn fit_to_area(&mut self, pos: Vector2, size: Vector2) {
        self.update_playfield(Bounds::new(pos, size));
    }

    
    async fn force_update_settings(&mut self, _settings: &Settings) {}
    async fn reload_skin(&mut self, skin_manager: &mut SkinManager) {
        for i in self.notes.iter_mut() {
            i.reload_skin(skin_manager).await;
        }
    }

    async fn apply_mods(&mut self, _mods: Arc<ModManager>) {}
    
    async fn beat_happened(&mut self, _pulse_length: f32) {}
    async fn kiai_changed(&mut self, _is_kiai: bool) {}
}

#[async_trait]
impl GameModeInput for UTypingGame {
    async fn key_down(&mut self, _key:Key) -> Option<ReplayAction> { None }
    
    async fn key_up(&mut self, _key:Key) -> Option<ReplayAction> { None }

    async fn on_text(&mut self, text: &String, _mods: &KeyModifiers) -> Option<ReplayAction> {
        if let Some(c) = text.chars().next() {
            Some(ReplayAction::MousePos(c as u8 as f32, 0.0))
        } else {
            None
        }
    }
}

impl GameModeProperties for UTypingGame {
    fn playmode(&self) -> String {"utyping".to_owned()}
    fn end_time(&self) -> f32 {self.end_time}

    fn get_possible_keys(&self) -> Vec<(KeyPress, &str)> {Vec::new()}

    fn timing_bar_things(&self) -> Vec<(f32, Color)> {
        vec![
            (self.hitwindow_100,  Color::new(0.3411, 0.8901, 0.0745, 1.0)),
            (self.hitwindow_300,  Color::new(0.1960, 0.7372, 0.9058, 1.0)),
            (self.hitwindow_miss, Color::new(0.8549, 0.6823, 0.2745, 1.0))
        ]
    }
    
}
