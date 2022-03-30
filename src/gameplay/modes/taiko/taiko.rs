/**
 * Taiko game mode
 * Author: ayyEve
 * 
 * 
 * depths:
 *  notes: 0..1000
 *  hit area: 1001
 *  playfield: 1002
 *  hit indicators: -1
 *  judgement indicators: -2
 *  spinners: -5
 */

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

/// how long should the drum buttons last for?
const DRUM_LIFETIME_TIME:f32 = 100.0;

// note texture size. this is required because peppy does dumb stuff with his textures
pub(super) const TAIKO_NOTE_TEX_SIZE:Vector2 = Vector2::new(128.0, 128.0);
pub(super) const TAIKO_JUDGEMENT_TEX_SIZE:Vector2 = Vector2::new(150.0, 150.0);
pub(super) const TAIKO_HIT_INDICATOR_TEX_SIZE:Vector2 = Vector2::new(90.0, 198.0);

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

pub struct TaikoGame {
    // lists
    pub notes: Vec<Box<dyn TaikoHitObject>>,
    timing_bars: Vec<TimingBar>,
    // list indices
    note_index: usize,

    // hit timing bar stuff
    hitwindow_300: f32,
    hitwindow_100: f32,
    hitwindow_miss: f32,

    end_time: f32,
    auto_helper: TaikoAutoHelper,

    hit_cache: HashMap<TaikoHit, f32>,

    taiko_settings: Arc<TaikoSettings>,

    left_kat_image: Option<Image>,
    left_don_image: Option<Image>,
    right_don_image: Option<Image>,
    right_kat_image: Option<Image>,

    judgement_helper: JudgmentImageHelper,
}
impl TaikoGame {
    pub fn next_note(&mut self) {self.note_index += 1}
}

impl GameMode for TaikoGame {
    fn playmode(&self) -> PlayMode {"taiko".to_owned()}
    fn end_time(&self) -> f32 {self.end_time}
    fn score_draw_start_pos(&self) -> Vector2 {
        Vector2::new(
            0.0,
            self.taiko_settings.hit_position.y + self.taiko_settings.note_radius * self.taiko_settings.big_note_multiplier + 50.0
        )
    }
    fn get_possible_keys(&self) -> Vec<(KeyPress, &str)> {
        vec![
            (KeyPress::LeftKat, "LK"),
            (KeyPress::LeftDon, "LD"),
            (KeyPress::RightDon, "RD"),
            (KeyPress::RightKat, "RK"),
        ]
    }

    fn new(beatmap:&Beatmap, diff_calc_only:bool) -> Result<Self, crate::errors::TatakuError> {
        let mut settings = get_settings!().taiko_settings.clone();
        // calculate the hit area
        settings.init_settings();
        let settings = Arc::new(settings);


        let mut hit_cache = HashMap::new();
        let mut left_kat_image = None;
        let mut left_don_image = None;
        let mut right_don_image = None;
        let mut right_kat_image = None;

        if !diff_calc_only {
            for i in [TaikoHit::LeftKat, TaikoHit::LeftDon, TaikoHit::RightDon, TaikoHit::RightKat] {
                hit_cache.insert(i, -999.9);
            }

            if let Some(don) = &mut SKIN_MANAGER.write().get_texture("taiko-drum-inner", true) {
                don.depth = -1.0;
                don.origin.x = don.tex_size().x;
                don.current_pos = settings.hit_position;
                
                let radius = settings.note_radius * settings.hit_area_radius_mult;
                let scale = Vector2::one() * (radius * 2.0) / TAIKO_HIT_INDICATOR_TEX_SIZE.x;

                left_don_image = Some(don.clone());
                left_don_image.as_mut().unwrap().current_scale = scale;

                right_don_image = Some(don.clone());
                right_don_image.as_mut().unwrap().current_scale = Vector2::new(-1.0, 1.0) * scale;
            }
            if let Some(kat) = &mut SKIN_MANAGER.write().get_texture("taiko-drum-outer", true) {
                kat.depth = -1.0;
                kat.origin.x = 0.0;
                kat.current_pos = settings.hit_position;
                
                let radius = settings.note_radius * settings.hit_area_radius_mult;
                let scale = Vector2::one() * (radius * 2.0) / TAIKO_HIT_INDICATOR_TEX_SIZE.x;
                
                left_kat_image = Some(kat.clone());
                left_kat_image.as_mut().unwrap().current_scale = Vector2::new(-1.0, 1.0) * scale;

                right_kat_image = Some(kat.clone());
                right_kat_image.as_mut().unwrap().current_scale = scale;
            }

        }

        
        let od = beatmap.get_beatmap_meta().get_od(&ModManager::get());

        let judgement_helper = {
            let mut skin = SKIN_MANAGER.write();
            JudgmentImageHelper::new(
                skin.get_texture("taiko-hit0", true),
                None, // 50 doesnt exist
                skin.get_texture("taiko-hit100", true),
                skin.get_texture("taiko-hit300", true),
                skin.get_texture("taiko-hit100k", true),
                skin.get_texture("taiko-hit300g", true),
            )
        };
        

        match beatmap {
            Beatmap::Osu(beatmap) => {
                let mut s = Self {
                    notes: Vec::new(),
                    note_index: 0,

                    timing_bars: Vec::new(),
                    end_time: 0.0,

                    hitwindow_100: 0.0,
                    hitwindow_300: 0.0,
                    hitwindow_miss: 0.0,

                    auto_helper: TaikoAutoHelper::new(),
                    taiko_settings: settings.clone(),
                    hit_cache,

                    left_kat_image,
                    left_don_image,
                    right_don_image,
                    right_kat_image,
                    judgement_helper,
                };

                // add notes
                for note in beatmap.notes.iter() {
                    let hit_type = if (note.hitsound & (2 | 8)) > 0 {super::HitType::Kat} else {super::HitType::Don};
                    let finisher = (note.hitsound & 4) > 0;

                    let note = Box::new(TaikoNote::new(
                        note.time,
                        hit_type,
                        finisher,
                        settings.clone(),
                        diff_calc_only,
                    ));
                    s.notes.push(note);
                }
                for slider in beatmap.sliders.iter() {
                    let SliderDef {time, slides, length, ..} = slider.to_owned();
                    let time = time;
                    let finisher = (slider.hitsound & 4) > 0;

                    let l = (length * 1.4) * slides as f32;
                    let v2 = 100.0 * (beatmap.slider_multiplier * 1.4);
                    let bl = beatmap.beat_length_at(time, true);
                    let end_time = time + (l / v2 * bl);
                    
                    // convert vars
                    let v = beatmap.slider_velocity_at(time);
                    let bl = beatmap.beat_length_at(time, beatmap.beatmap_version < 8);
                    let skip_period = (bl / beatmap.slider_tick_rate).min((end_time - time) / slides as f32);

                    if skip_period > 0.0 && beatmap.metadata.mode != "taiko" && l / v * 1000.0 < 2.0 * bl {
                        let mut i = 0;
                        let mut j = time;

                        // load sounds
                        // let sound_list_raw = if let Some(list) = split.next() {list.split("|")} else {"".split("")};

                        // when loading, if unified just have it as sound_types with 1 index
                        let mut sound_types:Vec<(HitType, bool)> = Vec::new();

                        for hitsound in slider.edge_sounds.iter() {
                            let hit_type = if (hitsound & (2 | 8)) > 0 {super::HitType::Kat} else {super::HitType::Don};
                            let finisher = (hitsound & 4) > 0;
                            sound_types.push((hit_type, finisher));
                        }
                        
                        let unified_sound_addition = sound_types.len() == 0;
                        if unified_sound_addition {
                            sound_types.push((HitType::Don, false));
                        }

                        //TODO: could this be turned into a for i in (x..y).step(n) ?
                        loop {
                            let sound_type = sound_types[i];

                            let note = Box::new(TaikoNote::new(
                                j,
                                sound_type.0,
                                sound_type.1,
                                settings.clone(),
                                diff_calc_only,
                            ));
                            s.notes.push(note);

                            if !unified_sound_addition {i = (i + 1) % sound_types.len()}

                            j += skip_period;
                            if !(j < end_time + skip_period / 8.0) {break}
                        }
                    } else {
                        let slider = Box::new(TaikoSlider::new(
                            time, 
                            end_time, 
                            finisher, 
                            settings.clone(),
                            diff_calc_only,
                        ));
                        s.notes.push(slider);
                    }
                }
                for spinner in beatmap.spinners.iter() {
                    let SpinnerDef {time, end_time, ..} = spinner;

                    let length = end_time - time;
                    let diff_map = map_difficulty(od, 3.0, 5.0, 7.5);
                    let hits_required:u16 = ((length / 1000.0 * diff_map) * 1.65).max(1.0) as u16; 

                    let spinner = Box::new(TaikoSpinner::new(
                        *time, 
                        *end_time, 
                        hits_required, 
                        settings.clone(),
                        diff_calc_only,
                    ));
                    s.notes.push(spinner);
                }

                if s.notes.len() == 0 {return Err(TatakuError::Beatmap(BeatmapError::InvalidFile))}
                s.notes.sort_by(|a, b|a.time().partial_cmp(&b.time()).unwrap());
                s.end_time = s.notes.iter().last().unwrap().time();

                Ok(s)
            }
            Beatmap::Adofai(beatmap) => {
                let settings = Arc::new(get_settings!().taiko_settings.clone());
                let mut s = Self {
                    notes: Vec::new(),
                    note_index: 0,

                    timing_bars: Vec::new(),
                    end_time: 0.0,

                    hitwindow_100: 0.0,
                    hitwindow_300: 0.0,
                    hitwindow_miss: 0.0,
                    auto_helper: TaikoAutoHelper::new(),
                    
                    taiko_settings: settings.clone(),
                    hit_cache,

                    left_kat_image,
                    left_don_image,
                    right_don_image,
                    right_kat_image,
                    judgement_helper,
                };

                // add notes
                for note in beatmap.notes.iter() {
                    let hit_type = super::HitType::Don;

                    let note = Box::new(TaikoNote::new(
                        note.time,
                        hit_type,
                        false,
                        settings.clone(),
                        diff_calc_only,
                    ));
                    s.notes.push(note);
                }

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
        let key = match frame {
            ReplayFrame::Press(k) => {
                manager.key_counter.key_down(k);
                k
            },
            ReplayFrame::Release(k) => {
                manager.key_counter.key_up(k);
                k
            },
            _ => return,
        };

        // draw drum
        match key {
            KeyPress::LeftKat => *self.hit_cache.get_mut(&TaikoHit::LeftKat).unwrap() = time,
            KeyPress::LeftDon => *self.hit_cache.get_mut(&TaikoHit::LeftDon).unwrap() = time,
            KeyPress::RightDon => *self.hit_cache.get_mut(&TaikoHit::RightDon).unwrap() = time,
            KeyPress::RightKat => *self.hit_cache.get_mut(&TaikoHit::RightKat).unwrap() = time,
            _=> {}
        }

        let hit_type:HitType = key.into();
        let mut sound = match hit_type {HitType::Don => "don", HitType::Kat => "kat"};
        let mut hit_volume = get_settings!().get_effect_vol() * (manager.current_timing_point().volume as f32 / 100.0);
        if manager.menu_background {
            hit_volume *= manager.background_game_settings.hitsound_volume;
        }

        // if theres no more notes to hit, return after playing the sound
        if self.note_index >= self.notes.len() {
            #[cfg(feature="bass_audio")]
            if let Ok(a) = Audio::play_preloaded(sound) {
                a.set_volume(hit_volume).unwrap();
            }
            #[cfg(feature="neb_audio")] {
                let a = Audio::play_preloaded(sound);
                a.upgrade().unwrap().set_volume(hit_volume);
            }
            return;
        }

        // check for finisher 2nd hit. 
        if self.note_index > 0 {
            let last_note = self.notes.get_mut(self.note_index-1).unwrap();

            match last_note.check_finisher(hit_type, time) {
                ScoreHit::Miss | ScoreHit::X50 => {return},
                ScoreHit::X100 | ScoreHit::Xkatu => {
                    manager.score.add_pts(100, true);
                    add_hit_indicator(time, &ScoreHit::X100, true, &self.taiko_settings, &self.judgement_helper, manager);
                    return;
                },
                ScoreHit::X300 | ScoreHit::Xgeki => {
                    manager.score.add_pts(300, true);
                    add_hit_indicator(time, &ScoreHit::X300, true, &self.taiko_settings, &self.judgement_helper, manager);
                    return;
                },
                ScoreHit::Other(points, _) => {
                    manager.score.add_pts(points as u64, false);
                    return;
                },
                ScoreHit::None => {},
            }
        }

        let note = self.notes.get_mut(self.note_index).unwrap();
        let note_time = note.time();
        match note.get_points(hit_type, time, (self.hitwindow_miss, self.hitwindow_100, self.hitwindow_300)) {
            ScoreHit::None | ScoreHit::X50 => {},
            ScoreHit::Miss => {
                manager.score.hit_miss(time, note_time);
                manager.hitbar_timings.push((time, time - note_time));
                manager.combo_break();

                manager.health.take_damage();
                if manager.health.is_dead() {
                    manager.fail()
                }

                // indicate this was a miss
                add_hit_indicator(time, &ScoreHit::Miss, false, &self.taiko_settings, &self.judgement_helper, manager);
                
                // next note
                self.next_note();
            }
            ScoreHit::X100 | ScoreHit::Xkatu => {
                manager.score.hit100(time, note_time);
                manager.hitbar_timings.push((time, time - note_time));

                manager.health.give_life();

                // only play finisher sounds if the note is both a finisher and was hit
                // could maybe also just change this to HitObject.get_sound() -> &str
                if note.finisher_sound() {sound = match hit_type {HitType::Don => "bigdon", HitType::Kat => "bigkat"}}
                //TODO: indicate this was a bad hit

                add_hit_indicator(time, &ScoreHit::X100, false, &self.taiko_settings, &self.judgement_helper, manager);
                
                // next note
                self.next_note();
            }
            ScoreHit::X300 | ScoreHit::Xgeki => {
                manager.score.hit300(time, note_time);
                manager.hitbar_timings.push((time, time - note_time));
                manager.health.give_extra_life();
                
                if note.finisher_sound() {sound = match hit_type {HitType::Don => "bigdon", HitType::Kat => "bigkat"}}

                add_hit_indicator(time, &ScoreHit::X300, false, &self.taiko_settings, &self.judgement_helper, manager);
                
                // next note
                self.next_note();
            }
            ScoreHit::Other(score, consume) => { // used by sliders and spinners
                // lol
                manager.score.score.score += score as u64;
                if consume {self.next_note()}
            }
        }

        #[cfg(feature="bass_audio")]
        if let Ok(a) = Audio::play_preloaded(sound) {
            a.set_volume(hit_volume).unwrap();
        }
        #[cfg(feature="neb_audio")] {
            let a = Audio::play_preloaded(sound);
            a.upgrade().unwrap().set_volume(hit_volume);
        }
    }


    fn update(&mut self, manager:&mut IngameManager, time: f32) {

        // do autoplay things
        if manager.current_mods.autoplay {
            let mut pending_frames = Vec::new();
            let notes = &mut self.notes;

            // get auto inputs
            self.auto_helper.update(time, notes, &mut pending_frames);

            // update index
            for i in 0..notes.len() {
                self.note_index = i;
                if (!notes[i].was_hit() && notes[i].note_type() != NoteType::Slider) || (notes[i].note_type() == NoteType::Slider && notes[i].end_time(0.0) > time) {
                    break;
                }
            }

            for frame in pending_frames.iter() {
                self.handle_replay_frame(*frame, time, manager);
            }
        }

        // update notes
        for note in self.notes.iter_mut() {note.update(time)}

        // if theres no more notes to hit, show score screen
        if self.note_index >= self.notes.len() {
            if manager.time() >= self.notes.last().unwrap().end_time(self.hitwindow_miss) + 1000.0 {
                manager.completed = true;
            }

            return;
        }

        // check if we missed the current note
        if self.notes[self.note_index].end_time(self.hitwindow_miss) < time {
            if self.notes[self.note_index].causes_miss() {
                // need to set these manually instead of score.hit_miss,
                // since we dont want to add anything to the hit error list
                let s = &mut manager.score;
                s.xmiss += 1;
                s.combo = 0;
                add_hit_indicator(time, &ScoreHit::Miss, false, &self.taiko_settings, &self.judgement_helper, manager);
                
                manager.health.take_damage();
                if manager.health.is_dead() {
                    manager.fail()
                }

            }
            self.next_note();
        }
        
        // TODO: might move tbs to a (time, speed) tuple
        for tb in self.timing_bars.iter_mut() {tb.update(time)}
    }
    fn draw(&mut self, args:RenderArgs, manager:&mut IngameManager, list:&mut Vec<Box<dyn Renderable>>) {
        let time = manager.time();
        let lifetime_time = DRUM_LIFETIME_TIME * manager.game_speed();
        
        for (hit_type, hit_time) in self.hit_cache.iter() {
            if time - hit_time > lifetime_time {continue}
            let alpha = 1.0 - (time - hit_time) / (lifetime_time * 4.0);
            let depth = -1.0;
            match hit_type {
                TaikoHit::LeftKat => {
                    if let Some(kat) = &self.left_kat_image {
                        let mut img = kat.clone();
                        img.current_color.a = alpha;
                        list.push(Box::new(img));
                    } else {
                        list.push(Box::new(HalfCircle::new(
                            self.taiko_settings.kat_color.alpha(alpha),
                            self.taiko_settings.hit_position,
                            depth,
                            self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult,
                            true
                        )));
                    }
                }
                TaikoHit::LeftDon => {
                    if let Some(don) = &self.left_don_image {
                        let mut img = don.clone();
                        img.current_color.a = alpha;
                        list.push(Box::new(img));
                    } else {
                        list.push(Box::new(HalfCircle::new(
                            self.taiko_settings.don_color.alpha(alpha),
                            self.taiko_settings.hit_position,
                            depth,
                            self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult,
                            true
                        )));
                    }
                }
                TaikoHit::RightDon => {
                    if let Some(don) = &self.right_don_image {
                        let mut img = don.clone();
                        img.current_color.a = alpha;
                        list.push(Box::new(img));
                    } else {
                        list.push(Box::new(HalfCircle::new(
                            self.taiko_settings.don_color.alpha(alpha),
                            self.taiko_settings.hit_position,
                            depth,
                            self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult,
                            false
                        )));
                    }
                }
                TaikoHit::RightKat => {
                    if let Some(kat) = &self.right_kat_image {
                        let mut img = kat.clone();
                        img.current_color.a = alpha;
                        list.push(Box::new(img));
                    } else {
                        list.push(Box::new(HalfCircle::new(
                            self.taiko_settings.kat_color.alpha(alpha),
                            self.taiko_settings.hit_position,
                            depth,
                            self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult,
                            false
                        )));
                    }
                }
            }
        }

        // draw the playfield
        list.push(Box::new(self.taiko_settings.get_playfield(args.window_size[0], manager.current_timing_point().kiai)));

        // draw the hit area
        list.push(Box::new(Circle::new(
            Color::BLACK,
            1001.0,
            self.taiko_settings.hit_position,
            self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult,
            None
        )));

        // draw notes
        for note in self.notes.iter_mut() {note.draw(args, list)}
        // draw timing lines
        for tb in self.timing_bars.iter_mut() {tb.draw(args, list)}
    }


    fn key_down(&mut self, key:piston::Key, manager:&mut IngameManager) {
        // dont accept key input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.autoplay || manager.replaying {
            return;
        }

        let time = manager.time();

        if key == self.taiko_settings.left_kat {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftKat), time, manager);
        }
        if key == self.taiko_settings.left_don {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftDon), time, manager);
        }
        if key == self.taiko_settings.right_don {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::RightDon), time, manager);
        }
        if key == self.taiko_settings.right_kat {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::RightKat), time, manager);
        }
    }
    fn key_up(&mut self, _key:piston::Key, _manager:&mut IngameManager) {}

    fn mouse_down(&mut self, btn:piston::MouseButton, manager:&mut IngameManager) {
        
        // dont accept mouse input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.autoplay || manager.replaying || self.taiko_settings.ignore_mouse_buttons {
            return;
        }
        
        let time = manager.time();
        match btn {
            piston::MouseButton::Left => self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftDon), time, manager),
            piston::MouseButton::Right => self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftKat), time, manager),
            _ => {}
        }
    }


    fn controller_press(&mut self, c: &Box<dyn Controller>, btn: u8, manager:&mut IngameManager) {
        // dont accept controller input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.autoplay || manager.replaying {
            return;
        }

        if let Some(c_config) = self.taiko_settings.clone().controller_config.get(&*c.get_name()) {
            let time = manager.time();

            if c_config.left_kat.check_button(btn) {
                self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftKat), time, manager);
            }

            if c_config.left_don.check_button(btn) {
                self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftDon), time, manager);
            }

            if c_config.right_don.check_button(btn) {
                self.handle_replay_frame(ReplayFrame::Press(KeyPress::RightDon), time, manager);
            }

            if c_config.right_kat.check_button(btn) {
                self.handle_replay_frame(ReplayFrame::Press(KeyPress::RightKat), time, manager);
            }

            // skip
            if Some(ControllerButton::Y) == c.map_button(btn) {
                self.skip_intro(manager);
            }

        } else {
            trace!("Controller with no setup");

            // TODO: if this is slow, we should store controller configs separately
            // but i dont think this will be an issue, as its unlikely to happen in the first place,
            // and if there is lag, the user is likely to retry the man anyways
            trace!("Setting up new controller");
            let mut new_settings = self.taiko_settings.as_ref().clone();
            new_settings.controller_config.insert((*c.get_name()).clone(), TaikoControllerConfig::defaults(c.get_name()));

            // update the global settings
            {
                let mut settings = get_settings_mut!();
                settings.taiko_settings = new_settings.clone();
                settings.save();
            }
            
            self.taiko_settings = Arc::new(new_settings);
            // rerun the handler now that the thing is setup
            self.controller_press(c, btn, manager);
        }
    }


    fn reset(&mut self, beatmap:&Beatmap) {
        for note in self.notes.as_mut_slice() {
            note.reset();

            // set note svs
            if self.taiko_settings.static_sv {
                note.set_sv(self.taiko_settings.sv_multiplier);
            } else {
                let sv = (beatmap.slider_velocity_at(note.time()) / SV_FACTOR) * self.taiko_settings.sv_multiplier;
                note.set_sv(sv);
            }
        }
        
        self.note_index = 0;

        let od = beatmap.get_beatmap_meta().od;
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
            let mut sv = self.taiko_settings.sv_multiplier;
            let mut time = parent_tps[0].time;
            let mut tp_index = 0;
            let step = beatmap.beat_length_at(time, false);
            time %= step; // get the earliest bar line possible

            loop {
                if !self.taiko_settings.static_sv {sv = (beatmap.slider_velocity_at(time) / SV_FACTOR) * self.taiko_settings.sv_multiplier}

                // if theres a bpm change, adjust the current time to that of the bpm change
                let next_bar_time = beatmap.beat_length_at(time, false) * BAR_SPACING; // bar spacing is actually the timing point measure

                // edge case for aspire maps
                if next_bar_time.is_nan() || next_bar_time == 0.0 {
                    break;
                }

                // add timing bar at current time
                self.timing_bars.push(TimingBar::new(time, sv, self.taiko_settings.clone()));

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
        self.hit_cache.iter_mut().for_each(|(_, t)| *t = -999.9);
    }


    fn skip_intro(&mut self, manager: &mut IngameManager) {
        if self.note_index > 0 {return}

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
        #[cfg(feature="bass_audio")]
        manager.song.set_position(time as f64).unwrap();
        #[cfg(feature="neb_audio")]
        manager.song.upgrade().unwrap().set_position(time);
    }

    fn timing_bar_things(&self) -> (Vec<(f32,Color)>, (f32,Color)) {
        (vec![
            (self.hitwindow_100, [0.3411, 0.8901, 0.0745, 1.0].into()),
            (self.hitwindow_300, [0.1960, 0.7372, 0.9058, 1.0].into()),
        ], (self.hitwindow_miss, [0.8549, 0.6823, 0.2745, 1.0].into()))
    }

    fn combo_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(
            Vector2::new(0.0, self.taiko_settings.hit_position.y - self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult/2.0),
            Vector2::new(self.taiko_settings.hit_position.x - self.taiko_settings.note_radius, self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult)
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
            ScoreHit::X100  => "x100".to_owned(),
            ScoreHit::X300  => "x300".to_owned(),
            ScoreHit::Xgeki => "Geki".to_owned(),
            ScoreHit::Xkatu => "Katu".to_owned(),
            
            ScoreHit::X50   => String::new(),
            ScoreHit::None  => String::new(),
            ScoreHit::Other(_, _) => String::new(),
        }
    }
}


fn add_hit_indicator(time: f32, mut hit_value: &ScoreHit, finisher_hit: bool, game_settings: &Arc<TaikoSettings>, judgment_helper: &JudgmentImageHelper, manager: &mut IngameManager) {
    let color = match hit_value {
        ScoreHit::Miss => Color::RED,
        ScoreHit::X100 | ScoreHit::Xkatu => Color::LIME,
        ScoreHit::X300 | ScoreHit::Xgeki => Color::new(0.0, 0.7647, 1.0, 1.0),
        ScoreHit::None | ScoreHit::X50 | ScoreHit::Other(_, _) => return,
    };

    // if finisher, upgrade to geki or katu
    if finisher_hit {
        if let &ScoreHit::X100 = hit_value {
            hit_value = &ScoreHit::Xkatu;
        } else if let &ScoreHit::X300 = hit_value {
            hit_value = &ScoreHit::Xgeki;
        }
    }

    let mut image = judgment_helper.get_from_scorehit(&hit_value);
    if let Some(image) = &mut image {
        image.current_pos = game_settings.hit_position;
        image.depth = -2.0;

        let radius = game_settings.note_radius * game_settings.big_note_multiplier * game_settings.hit_area_radius_mult;
        let scale = Vector2::one() * (radius * 2.0) / TAIKO_JUDGEMENT_TEX_SIZE;
        image.initial_scale = scale;
        image.current_scale = scale;
    }

    manager.add_judgement_indicator(BasicJudgementIndicator::new(
        game_settings.hit_position, 
        time,
        -2.0,
        game_settings.note_radius * 0.5 * if finisher_hit {game_settings.big_note_multiplier} else {1.0},
        color,
        image
    ))
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





struct TaikoAutoHelper {
    don_presses: u32,
    kat_presses: u32,

    current_note_duration: f32,
    note_index: i64,

    last_hit: f32,
    last_update: f32,
}
impl TaikoAutoHelper {
    fn new() -> Self {
        Self {
            don_presses: 0, 
            kat_presses: 0, 
            note_index: - 1, 
            last_hit: 0.0, 
            current_note_duration: 0.0,
            last_update: 0.0
            // notes: Vec::new()
        }
    }

    fn update(&mut self, time: f32, notes: &mut Vec<Box<dyn TaikoHitObject>>, frames: &mut Vec<ReplayFrame>) {
        let catching_up = time - self.last_update > 20.0;
        self.last_update = time;

        if catching_up {trace!("catching up")}

        for i in 0..notes.len() {
            let note = &mut notes[i];
            
            if time >= note.time() 
            // && time <= note.end_time(100.0) 
            && !note.was_hit() {

                // check if we're catching up
                if catching_up {
                    // pretend the note was hit
                    note.force_hit();
                    continue;
                }

                // otherwise it spams sliders even after it has finished
                if let NoteType::Slider = note.note_type() {
                    if time > note.end_time(0.0) {
                        continue;
                    }
                }

                // we're already working on this note
                if i as i64 == self.note_index {
                    match note.note_type() {
                        NoteType::Slider | NoteType::Spinner => {
                            let time_between_hits = self.current_note_duration / (note.hits_to_complete() as f32);
                            
                            // if its not time to do another hit yet
                            if time - self.last_hit < time_between_hits {return}
                        }

                        // nothing to do for notes (they only need 1 hit) and holds dont exist
                        // dont do anything else for this object
                        NoteType::Hold | NoteType::Note => continue,
                    }
                } else {
                    self.note_index = i as i64;
                        
                    match note.note_type() {
                        NoteType::Slider | NoteType::Spinner => self.current_note_duration = note.end_time(0.0) - note.time(),
                        _ => {},
                    }
                }

                self.last_hit = time;
                // let note_type = note.note_type();
                let is_kat = note.is_kat();
                let is_finisher = note.finisher_sound();

                let count = self.don_presses + self.kat_presses;
                let side = count % 2;

                if is_finisher {
                    if is_kat {
                        frames.push(ReplayFrame::Press(KeyPress::LeftKat));
                        frames.push(ReplayFrame::Press(KeyPress::RightKat));
                    } else {
                        frames.push(ReplayFrame::Press(KeyPress::LeftDon));
                        frames.push(ReplayFrame::Press(KeyPress::RightDon));
                    }
                } else {
                    match (is_kat, side) {
                        // kat, left side
                        (true, 0) => frames.push(ReplayFrame::Press(KeyPress::LeftKat)),

                        // kat, right side
                        (true, 1) => frames.push(ReplayFrame::Press(KeyPress::RightKat)),

                        // don, left side
                        (false, 0) => frames.push(ReplayFrame::Press(KeyPress::LeftDon)),
                        
                        // don, right side
                        (false, 1) => frames.push(ReplayFrame::Press(KeyPress::RightDon)),

                        // shouldnt happen
                        _ => {}
                    }
                }

                if is_kat {
                    self.kat_presses += 1;
                } else {
                    self.don_presses += 1;
                }

                return
            }
        }
    }
}


#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum TaikoHit {
    LeftKat,
    LeftDon,
    RightDon,
    RightKat
}

