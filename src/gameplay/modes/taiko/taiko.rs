use super::*;
use crate::prelude::*;


// taiko drum controller:
// outer left: button 6
// inner left: button 10
// innert right: button 11
// outer right: button 7




pub const NOTE_RADIUS:f64 = 32.0;
pub const HIT_AREA_RADIUS:f64 = NOTE_RADIUS * 1.3;
pub const HIT_POSITION:Vector2 = Vector2::new(180.0, 200.0);
pub const PLAYFIELD_RADIUS:f64 = NOTE_RADIUS * 2.0; // actually height, oops

pub const BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0); // timing bar color
const BAR_WIDTH:f64 = 4.0; // how wide is a timing bar
const BAR_SPACING:f32 = 4.0; // how many beats between timing bars

const SV_FACTOR:f32 = 700.0; // bc sv is bonked, divide it by this amount

/// how long should the drum buttons last for?
const DRUM_LIFETIME_TIME:u64 = 100;


pub struct TaikoGame {
    // lists
    pub notes: Vec<Box<dyn TaikoHitObject>>,
    timing_bars: Vec<TimingBar>,
    // list indices
    note_index: usize,
    timing_point_index: usize,

    // hit timing bar stuff
    hitwindow_300: f32,
    hitwindow_100: f32,
    hitwindow_miss: f32,

    end_time: f32,

    render_queue: Vec<Box<HalfCircle>>,

    auto_helper: TaikoAutoHelper
}
impl TaikoGame {
    pub fn next_note(&mut self) {self.note_index += 1}
}

impl GameMode for TaikoGame {
    fn playmode(&self) -> PlayMode {PlayMode::Taiko}
    fn end_time(&self) -> f32 {self.end_time}
    fn new(beatmap:&Beatmap) -> Result<Self, crate::errors::TatakuError> {

        match beatmap {
            Beatmap::Osu(beatmap) => {
                let mut s = Self {
                    notes: Vec::new(),
                    note_index: 0,

                    timing_bars: Vec::new(),
                    timing_point_index: 0,
                    end_time: 0.0,

                    hitwindow_100: 0.0,
                    hitwindow_300: 0.0,
                    hitwindow_miss: 0.0,

                    render_queue: Vec::new(),
                    auto_helper: TaikoAutoHelper::new()
                };

                // add notes
                for note in beatmap.notes.iter() {
                    let hit_type = if (note.hitsound & (2 | 8)) > 0 {super::HitType::Kat} else {super::HitType::Don};
                    let finisher = (note.hitsound & 4) > 0;

                    let note = Box::new(TaikoNote::new(
                        note.time,
                        hit_type,
                        finisher
                    ));
                    s.notes.push(note);
                }
                for slider in beatmap.sliders.iter() {
                    let SliderDef {time, slides, length, ..} = slider.to_owned();
                    let time = time;
                    let finisher = (slider.hitsound & 4) > 0;

                    let l = (length * 1.4) * slides as f32;
                    let v2 = 100.0 * (beatmap.metadata.slider_multiplier * 1.4);
                    let bl = beatmap.beat_length_at(time, true);
                    let end_time = time + (l / v2 * bl);
                    
                    // convert vars
                    let v = beatmap.slider_velocity_at(time);
                    let bl = beatmap.beat_length_at(time, beatmap.metadata.beatmap_version < 8);
                    let skip_period = (bl / beatmap.metadata.slider_tick_rate).min((end_time - time) / slides as f32);

                    if skip_period > 0.0 && beatmap.metadata.mode != PlayMode::Taiko && l / v * 1000.0 < 2.0 * bl {
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
                                sound_type.1
                            ));
                            s.notes.push(note);

                            if !unified_sound_addition {i = (i + 1) % sound_types.len()}

                            j += skip_period;
                            if !(j < end_time + skip_period / 8.0) {break}
                        }
                    } else {
                        let slider = Box::new(TaikoSlider::new(time, end_time, finisher));
                        s.notes.push(slider);
                    }
                }
                for spinner in beatmap.spinners.iter() {
                    let SpinnerDef {time, end_time, ..} = spinner;

                    let length = end_time - time;
                    let diff_map = map_difficulty(beatmap.metadata.od, 3.0, 5.0, 7.5);
                    let hits_required:u16 = ((length / 1000.0 * diff_map) * 1.65).max(1.0) as u16; // ((this.Length / 1000.0 * this.MapDifficultyRange(od, 3.0, 5.0, 7.5)) * 1.65).max(1.0)

                    let spinner = Box::new(TaikoSpinner::new(*time, *end_time, hits_required));
                    s.notes.push(spinner);
                }

                s.notes.sort_by(|a, b|a.time().partial_cmp(&b.time()).unwrap());
                s.end_time = s.notes.iter().last().unwrap().time();

                Ok(s)
            },
            Beatmap::Adofai(beatmap) => {
                let mut s = Self {
                    notes: Vec::new(),
                    note_index: 0,

                    timing_bars: Vec::new(),
                    timing_point_index: 0,
                    end_time: 0.0,

                    hitwindow_100: 0.0,
                    hitwindow_300: 0.0,
                    hitwindow_miss: 0.0,

                    render_queue: Vec::new(),
                    auto_helper: TaikoAutoHelper::new()
                };

                // add notes
                for note in beatmap.notes.iter() {
                    let hit_type = super::HitType::Don;

                    let note = Box::new(TaikoNote::new(
                        note.time,
                        hit_type,
                        false
                    ));
                    s.notes.push(note);
                }

                s.notes.sort_by(|a, b|a.time().partial_cmp(&b.time()).unwrap());
                s.end_time = s.notes.iter().last().unwrap().time();

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
        let key = match frame {
            ReplayFrame::Press(k) => k,
            ReplayFrame::Release(k) => k,
            _ => return,
        };

        // draw drum
        match key {
            KeyPress::LeftKat => {
                let mut hit = HalfCircle::new(
                    Color::BLUE,
                    HIT_POSITION,
                    1.0,
                    HIT_AREA_RADIUS,
                    true
                );
                hit.set_lifetime(DRUM_LIFETIME_TIME);
                self.render_queue.push(Box::new(hit));
            },
            KeyPress::LeftDon => {
                let mut hit = HalfCircle::new(
                    Color::RED,
                    HIT_POSITION,
                    1.0,
                    HIT_AREA_RADIUS,
                    true
                );
                hit.set_lifetime(DRUM_LIFETIME_TIME);
                self.render_queue.push(Box::new(hit));
            },
            KeyPress::RightDon => {
                let mut hit = HalfCircle::new(
                    Color::RED,
                    HIT_POSITION,
                    1.0,
                    HIT_AREA_RADIUS,
                    false
                );
                hit.set_lifetime(DRUM_LIFETIME_TIME);
                self.render_queue.push(Box::new(hit));
            },
            KeyPress::RightKat => {
                let mut hit = HalfCircle::new(
                    Color::BLUE,
                    HIT_POSITION,
                    1.0,
                    HIT_AREA_RADIUS,
                    false
                );
                hit.set_lifetime(DRUM_LIFETIME_TIME);
                self.render_queue.push(Box::new(hit));
            },
            _=> {}
        }

        let hit_type:HitType = key.into();
        let mut sound = match hit_type {HitType::Don => "don", HitType::Kat => "kat"};
        let hit_volume = Settings::get().get_effect_vol() * (manager.timing_points[self.timing_point_index].volume as f32 / 100.0);

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
                    return;
                },
                ScoreHit::X300 | ScoreHit::Xgeki => {
                    manager.score.add_pts(300, true);
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
            ScoreHit::None | ScoreHit::X50 => {
                // play sound
                // Audio::play_preloaded(sound);
            },
            ScoreHit::Miss => {
                manager.score.hit_miss(time, note_time);
                manager.hitbar_timings.push((time, time - note_time));
                self.next_note();
                // Audio::play_preloaded(sound);

                //TODO: play miss sound
                //TODO: indicate this was a miss
            },
            ScoreHit::X100 | ScoreHit::Xkatu => {
                manager.score.hit100(time, note_time);
                manager.hitbar_timings.push((time, time - note_time));

                // only play finisher sounds if the note is both a finisher and was hit
                // could maybe also just change this to HitObject.get_sound() -> &str
                if note.finisher_sound() {sound = match hit_type {HitType::Don => "bigdon", HitType::Kat => "bigkat"}}
                // Audio::play_preloaded(sound);
                //TODO: indicate this was a bad hit

                self.next_note();
            },
            ScoreHit::X300 | ScoreHit::Xgeki => {
                manager.score.hit300(time, note_time);
                manager.hitbar_timings.push((time, time - note_time));
                
                if note.finisher_sound() {sound = match hit_type {HitType::Don => "bigdon", HitType::Kat => "bigkat"}}
                // Audio::play_preloaded(sound);

                self.next_note();
            },
            ScoreHit::Other(score, consume) => { // used by sliders and spinners
                manager.score.score += score as u64;
                if consume {self.next_note()}
                // Audio::play_preloaded(sound);
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
                if !notes[i].was_hit() && notes[i].note_type() != NoteType::Slider {
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
            manager.completed = true;
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
            }
            self.next_note();
        }
        
        // TODO: might move tbs to a (time, speed) tuple
        for tb in self.timing_bars.iter_mut() {tb.update(time)}

        let timing_points = &manager.timing_points;
        // check timing point
        if self.timing_point_index + 1 < timing_points.len() && timing_points[self.timing_point_index + 1].time <= time {
            self.timing_point_index += 1;
        }
    }
    fn draw(&mut self, args:RenderArgs, manager:&mut IngameManager, list:&mut Vec<Box<dyn Renderable>>) {
        list.reserve(self.render_queue.len());
        for i in self.render_queue.iter() {
            list.push(i.clone());
        }
        self.render_queue.clear();

        // draw the playfield
        let playfield = Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            f64::MAX-4.0,
            Vector2::new(0.0, HIT_POSITION.y - (PLAYFIELD_RADIUS + 2.0)),
            Vector2::new(args.window_size[0], (PLAYFIELD_RADIUS+2.0) * 2.0),
            if manager.timing_points[self.timing_point_index].kiai {
                Some(Border::new(Color::YELLOW, 2.0))
            } else {None}
        );
        list.push(Box::new(playfield));

        // draw the hit area
        list.push(Box::new(Circle::new(
            Color::BLACK,
            f64::MAX,
            HIT_POSITION,
            HIT_AREA_RADIUS + 2.0
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

        let settings = Settings::get().taiko_settings;
        let time = manager.time();

        if key == settings.left_kat {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftKat), time, manager);
        }
        if key == settings.left_don {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftDon), time, manager);
        }
        if key == settings.right_don {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::RightDon), time, manager);
        }
        if key == settings.right_kat {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::RightKat), time, manager);
        }
    }
    fn key_up(&mut self, _key:piston::Key, _manager:&mut IngameManager) {}

    fn mouse_down(&mut self, btn:piston::MouseButton, manager:&mut IngameManager) {
        
        // dont accept mouse input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.autoplay || manager.replaying {
            return;
        }
        
        {
            let settings = Settings::get().taiko_settings;
            if settings.ignore_mouse_buttons {return}
        }
        let time = manager.time();

        match btn {
            piston::MouseButton::Left => self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftDon), time, manager),
            piston::MouseButton::Right => self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftKat), time, manager),
            _ => {}
        }
    }


    fn controller_press(&mut self, _c_id: u32, btn: u8, manager:&mut IngameManager) {
        // let id = btn.id;
        let time = manager.time();
        match btn {
            6 => self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftKat), time, manager),
            7 => self.handle_replay_frame(ReplayFrame::Press(KeyPress::RightKat), time, manager),
            10 => self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftDon), time, manager),
            11 => self.handle_replay_frame(ReplayFrame::Press(KeyPress::RightDon), time, manager),
            _ => {println!("[Taiko::Controller] other button")} // other key, not accepted for now
        }
    }


    fn reset(&mut self, beatmap:&Beatmap) {
        let settings = Settings::get().taiko_settings;
        
        for note in self.notes.as_mut_slice() {
            note.reset();

            // set note svs
            if settings.static_sv {
                note.set_sv(settings.sv_multiplier);
            } else {
                let sv = beatmap.slider_velocity_at(note.time()) / SV_FACTOR;
                note.set_sv(sv);
            }
        }
        
        self.note_index = 0;
        self.timing_point_index = 0;

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
            let mut sv = settings.sv_multiplier;
            let mut time = parent_tps[0].time;
            let mut tp_index = 0;
            let step = beatmap.beat_length_at(time, false);
            time %= step; // get the earliest bar line possible

            loop {
                if !settings.static_sv {sv = beatmap.slider_velocity_at(time) / SV_FACTOR}

                // if theres a bpm change, adjust the current time to that of the bpm change
                let next_bar_time = beatmap.beat_length_at(time, false) * BAR_SPACING; // bar spacing is actually the timing point measure

                // edge case for aspire maps
                if next_bar_time.is_nan() || next_bar_time == 0.0 {
                    break;
                }

                // add timing bar at current time
                self.timing_bars.push(TimingBar::new(time, sv));

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
    
    }



    fn skip_intro(&mut self, manager: &mut IngameManager) {
        if self.note_index > 0 {return}

        let x_needed = Settings::window_size().x as f32;
        let mut time = manager.time();

        loop {
            let mut found = false;
            for note in self.notes.iter() {if note.x_at(time) <= x_needed {found = true; break}}
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

    fn timing_bar_things(&self) -> (Vec<(f32,Color)>, (f32,Color)) {
        (vec![
            (self.hitwindow_100, [0.3411, 0.8901, 0.0745, 1.0].into()),
            (self.hitwindow_300, [0.1960, 0.7372, 0.9058, 1.0].into()),
        ], (self.hitwindow_miss, [0.8549, 0.6823, 0.2745, 1.0].into()))
    }

    fn combo_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(
            Vector2::new(0.0, HIT_POSITION.y - HIT_AREA_RADIUS/2.0),
            Vector2::new(HIT_POSITION.x - NOTE_RADIUS, HIT_AREA_RADIUS)
        )
    }

    fn apply_auto(&mut self, settings: &crate::game::BackgroundGameSettings) {
        for note in self.notes.iter_mut() {
            note.set_alpha(settings.opacity)
        }
    }
}


// timing bar struct
//TODO: might be able to reduce this to a (time, speed) and just calc pos on draw
#[derive(Copy, Clone, Debug)]
struct TimingBar {
    time: f32,
    speed: f32,
    pos: Vector2
}
impl TimingBar {
    pub fn new(time:f32, speed:f32) -> TimingBar {
        TimingBar {
            time, 
            speed,
            pos: Vector2::zero(),
        }
    }

    pub fn update(&mut self, time:f32) {
        self.pos = HIT_POSITION + Vector2::new(((self.time - time) * self.speed) as f64 - BAR_WIDTH / 2.0, -PLAYFIELD_RADIUS);
    }

    fn draw(&mut self, _args:RenderArgs, list:&mut Vec<Box<dyn Renderable>>){
        if self.pos.x + BAR_WIDTH < 0.0 || self.pos.x - BAR_WIDTH > Settings::window_size().x as f64 {return}

        const SIZE:Vector2 = Vector2::new(BAR_WIDTH, PLAYFIELD_RADIUS*2.0);
        const DEPTH:f64 = f64::MAX-5.0;

        list.push(Box::new(Rectangle::new(
            BAR_COLOR,
            DEPTH,
            self.pos,
            SIZE,
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

        if catching_up {println!("catching up")}

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
