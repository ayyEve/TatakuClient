/**
 * Taiko game mode
 * Author: ayyEve
 * 
 * NOTE! gekis and katus are for DISPLAY ONLY!!
 * they are not factored into acc!!
 * 
 * 
 * depths:
 *  notes: 0..1000
 *  hit area: 1001
 *  timing bars: 1001.5
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
pub const SV_OVERRIDE:f32 = 2000.0;

/// how long should the drum buttons last for?
const DRUM_LIFETIME_TIME:f32 = 100.0;

// note texture size. this is required because peppy does dumb stuff with his textures
pub(super) const TAIKO_NOTE_TEX_SIZE:Vector2 = Vector2::new(128.0, 128.0);
pub(super) const TAIKO_JUDGEMENT_TEX_SIZE:Vector2 = Vector2::new(150.0, 150.0);
pub(super) const TAIKO_HIT_INDICATOR_TEX_SIZE:Vector2 = Vector2::new(90.0, 198.0);

pub struct TaikoGame {
    // lists
    pub notes: Vec<Box<dyn TaikoHitObject>>,
    timing_bars: Vec<TimingBar>,
    // list indices
    note_index: usize,

    end_time: f32,
    auto_helper: TaikoAutoHelper,

    taiko_settings: Arc<TaikoSettings>,
    metadata: Arc<BeatmapMeta>,
    playfield: Arc<TaikoPlayfield>,

    left_kat_image: Option<Image>,
    left_don_image: Option<Image>,
    right_don_image: Option<Image>,
    right_kat_image: Option<Image>,
    judgement_helper: JudgmentImageHelper,

    counter: FullAltCounter,
    
    hit_windows: Vec<(TaikoHitJudgments, Range<f32>)>,
    hit_cache: HashMap<TaikoHit, f32>,
    miss_window: f32,

    last_judgment: TaikoHitJudgments,
    current_mods: Arc<ModManager>
}
impl TaikoGame {
    pub fn next_note(&mut self) { self.note_index += 1 }


    async fn play_sound(&self, manager: &mut IngameManager, note_time:f32,  hit_type: HitType, finisher: bool) {
        let hitsound;
        match (hit_type, finisher) {
            (HitType::Don, false) => hitsound = 1, // normal is don
            (HitType::Don, true)  => hitsound = 4, // finish is bigdon
            (HitType::Kat, false) => hitsound = 8, // clap is kat
            (HitType::Kat, true)  => hitsound = 2, // whistle is bigkat
        }

        let samples = HitSamples {
            normal_set: 0,
            addition_set: 0,
            index: 0,
            volume: 0,
            filename: None,
        };

        manager.play_note_sound(note_time, hitsound, samples, false).await;
    }

    async fn setup_hitwindows(&mut self) {
        let od = get_od(&self.metadata, &self.current_mods);

        // windows
        let w_miss = map_difficulty(od, 135.0, 95.0, 70.0);
        let w_100 = map_difficulty(od, 120.0, 80.0, 50.0);
        let w_300 = map_difficulty(od, 50.0, 35.0, 20.0);

        use TaikoHitJudgments::*;
        self.hit_windows = vec![
            (X300, 0.0..w_300),
            (X100, w_300..w_100),
            (Miss, w_100..w_miss),
        ];
        self.miss_window = w_miss;

        for note in self.notes.iter_mut() {
            if note.note_type() == NoteType::Spinner {
                let length = note.end_time(0.0) - note.time();
                let diff_map = map_difficulty(od, 3.0, 5.0, 7.5);
                let required_hits = ((length / 1000.0 * diff_map) * 1.65).max(1.0) as u16; 
                note.set_required_hits(required_hits);
            }
        }
    }
}
#[async_trait]
impl GameMode for TaikoGame {
    async fn new(beatmap:&Beatmap, diff_calc_only:bool) -> Result<Self, crate::errors::TatakuError> {
        let mut settings = get_settings!().taiko_settings.clone();
        let metadata = beatmap.get_beatmap_meta();
        // calculate the hit area
        settings.init_settings().await;
        let settings = Arc::new(settings);

        let mut hit_cache = HashMap::new();
        let left_kat_image = None;
        let left_don_image = None;
        let right_don_image = None;
        let right_kat_image = None;
        let judgement_helper = JudgmentImageHelper::new(DefaultHitJudgments::None).await;

        for i in [TaikoHit::LeftKat, TaikoHit::LeftDon, TaikoHit::RightDon, TaikoHit::RightKat] {
            hit_cache.insert(i, -999.9);
        }

        let current_mods = Arc::new(ModManager::get().await.clone());


        let playfield = Arc::new(TaikoPlayfield {
            pos: Vector2::zero(),
            size: WindowSize::get().0
        });


        let mut s = match beatmap {
            Beatmap::Osu(beatmap) => {
                let mut s = Self {
                    notes: Vec::new(),
                    note_index: 0,

                    timing_bars: Vec::new(),
                    end_time: 0.0,

                    auto_helper: TaikoAutoHelper::new(),
                    taiko_settings: settings.clone(),
                    playfield: playfield.clone(),
                    metadata,
                    

                    left_kat_image,
                    left_don_image,
                    right_don_image,
                    right_kat_image,
                    judgement_helper,

                    hit_windows: Vec::new(),
                    miss_window: 0.0,
                    hit_cache,
                    last_judgment: TaikoHitJudgments::Miss,
                    counter: FullAltCounter::new(),
                    current_mods,
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
                        playfield.clone(),
                        diff_calc_only,
                    ).await);
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
                                playfield.clone(),
                                diff_calc_only,
                            ).await);
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
                            playfield.clone(),
                            diff_calc_only,
                        ).await);
                        s.notes.push(slider);
                    }
                }
                for spinner in beatmap.spinners.iter() {

                    let spinner = Box::new(TaikoSpinner::new(
                        spinner.time,
                        spinner.end_time, 
                        0, 
                        settings.clone(),
                        playfield.clone(),
                        diff_calc_only,
                    ).await);
                    s.notes.push(spinner);
                }
                s
            }
            Beatmap::Adofai(beatmap) => {
                let settings = Arc::new(get_settings!().taiko_settings.clone());
                let mut s = Self {
                    notes: Vec::new(),
                    note_index: 0,

                    timing_bars: Vec::new(),
                    end_time: 0.0,

                    auto_helper: TaikoAutoHelper::new(),
                    taiko_settings: settings.clone(),
                    playfield: playfield.clone(),
                    metadata,

                    left_kat_image,
                    left_don_image,
                    right_don_image,
                    right_kat_image,
                    judgement_helper,

                    hit_windows: Vec::new(),
                    miss_window: 0.0,
                    hit_cache,
                    last_judgment: TaikoHitJudgments::Miss,
                    counter: FullAltCounter::new(),
                    current_mods,
                };

                // add notes
                for note in beatmap.notes.iter() {
                    let hit_type = super::HitType::Don;

                    let note = Box::new(TaikoNote::new(
                        note.time,
                        hit_type,
                        false,
                        settings.clone(),
                        playfield.clone(),
                        diff_calc_only,
                    ).await);

                    s.notes.push(note);
                }

                s
            }

            _ => return Err(BeatmapError::UnsupportedMode.into()),
        };

        if s.notes.len() == 0 {return Err(TatakuError::Beatmap(BeatmapError::InvalidFile))}
        s.notes.sort_by(|a, b|a.time().partial_cmp(&b.time()).unwrap());
        s.end_time = s.notes.iter().last().unwrap().time();

        if !diff_calc_only {
            s.reload_skin().await;
        }

        s.setup_hitwindows().await;

        Ok(s)
    }

    async fn handle_replay_frame(&mut self, frame:ReplayFrame, time:f32, manager:&mut IngameManager) {
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
                // should probably return here lol
                k
            },
            _ => return,
        };


        // turn the keypress into a hit type
        let hit_type = match key {
            KeyPress::LeftKat  => TaikoHit::LeftKat,
            KeyPress::LeftDon  => TaikoHit::LeftDon,
            KeyPress::RightDon => TaikoHit::RightDon,
            KeyPress::RightKat => TaikoHit::RightKat,
            _ => TaikoHit::LeftKat
        };

        // check fullalt
        if manager.current_mods.has_mod(FullAlt.name()) {
            if !self.counter.add_hit(hit_type) {
                return;
            }
        }

        // draw drum
        *self.hit_cache.get_mut(&hit_type).unwrap() = time;

        let hit_type:HitType = key.into();
        let mut finisher_sound = false;
        // let mut sound = match hit_type {HitType::Don => "don", HitType::Kat => "kat"};

        // if theres no more notes to hit, return after playing the sound
        if self.note_index >= self.notes.len() {
            self.play_sound(manager, time, hit_type, false).await;
            return;
        }

        // check for finisher 2nd hit. 
        if self.last_judgment != TaikoHitJudgments::Miss {
            let last_note = self.notes.get_mut(self.note_index-1).unwrap();
            if last_note.check_finisher(hit_type, time, manager.current_mods.get_speed()) {
                let j = match &self.last_judgment {
                    TaikoHitJudgments::X300 | TaikoHitJudgments::Geki => &TaikoHitJudgments::Geki,
                    TaikoHitJudgments::X100 | TaikoHitJudgments::Katu => &TaikoHitJudgments::Katu,
                    _ => return, // this shouldnt happen, last judgment will always be one of the above
                };
                
                // add whatever the last judgment was as a finisher score
                manager.add_judgment(j).await;
                add_hit_indicator(time, j, true, &self.taiko_settings, &self.judgement_helper, manager);
                return;
            }
        }

        let note = self.notes.get_mut(self.note_index).unwrap();
        let note_time = note.time();

        match note.note_type() {
            NoteType::Note => {
                let cond = || note.hit_type() == hit_type;

                if let Some(judge) = manager.check_judgment_condition(&self.hit_windows, time, note_time, cond, &TaikoHitJudgments::Miss).await {
                    // if note.finisher_sound() { sound = match hit_type { HitType::Don => "bigdon", HitType::Kat => "bigkat" } }
                    finisher_sound = note.finisher_sound();

                    if let TaikoHitJudgments::Miss = judge {
                        note.miss(time);
                    } else {
                        note.hit(time);
                    }

                    add_hit_indicator(time, judge, false, &self.taiko_settings, &self.judgement_helper, manager);
                    
                    self.last_judgment = *judge;
                    self.next_note();
                }
            },

            // slider or spinner, special hit stuff
            NoteType::Slider  if note.hit(time) => manager.add_judgment(&TaikoHitJudgments::SliderPoint).await,
            NoteType::Spinner if note.hit(time) => manager.add_judgment(&TaikoHitJudgments::SpinnerPoint).await,
            _ => {}
        }

        self.play_sound(manager, note_time, hit_type, finisher_sound).await;
    }


    async fn update(&mut self, manager:&mut IngameManager, time: f32) {

        // do autoplay things
        if manager.current_mods.has_autoplay() {
            let mut pending_frames = Vec::new();
            let notes = &mut self.notes;

            // get auto inputs
            self.auto_helper.update(time, notes, &mut pending_frames);

            if !manager.completed {
                
                // update index
                for i in 0..notes.len() + 1 {
                    self.note_index = i;
                    if i == notes.len() { break }

                    if (!notes[i].was_hit() && notes[i].note_type() != NoteType::Slider) || (notes[i].note_type() == NoteType::Slider && notes[i].end_time(0.0) > time) {
                        break;
                    }
                }

            }

            for frame in pending_frames.iter() {
                self.handle_replay_frame(*frame, time, manager).await;
            }
        }

        // update notes
        for note in self.notes.iter_mut() { 
            note.update(time).await;
        }

        // if theres no more notes to hit, show score screen
        if self.note_index >= self.notes.len() {
            if manager.time() >= self.notes.last().unwrap().end_time(self.miss_window) + 1000.0 {
                manager.completed = true;
            }

            return;
        }

        // check if we missed the current note
        if self.notes[self.note_index].end_time(self.miss_window) < time {
            if self.notes[self.note_index].causes_miss() {
                self.notes[self.note_index].miss(time);

                let j = &TaikoHitJudgments::Miss;
                manager.add_judgment(j).await;
                add_hit_indicator(time, j, false, &self.taiko_settings, &self.judgement_helper, manager);
            }

            self.next_note();
        }
        
        // TODO: might move tbs to a (time, speed) tuple
        for tb in self.timing_bars.iter_mut() { tb.update(time); }
    }
    async fn draw(&mut self, args:RenderArgs, manager:&mut IngameManager, list:&mut Vec<Box<dyn Renderable>>) {
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
        for note in self.notes.iter_mut() { list.extend(note.draw(args).await) }
        // draw timing lines
        for tb in self.timing_bars.iter_mut() { tb.draw(args, list) }
    }

    async fn reset(&mut self, beatmap:&Beatmap) {
        for note in self.notes.iter_mut() {
            note.reset().await;

            // set note svs
            if self.current_mods.has_mod("no_sv") {
                note.set_sv(self.taiko_settings.sv_multiplier);
            } else {
                let sv = (beatmap.slider_velocity_at(note.time()) / SV_FACTOR) * self.taiko_settings.sv_multiplier;
                note.set_sv(sv);
            }
        }
        
        self.note_index = 0;
        self.last_judgment = TaikoHitJudgments::Miss;
        self.counter = FullAltCounter::new();

        // setup timing bars
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
                if !self.current_mods.has_mod("no_sv") {sv = (beatmap.slider_velocity_at(time) / SV_FACTOR) * self.taiko_settings.sv_multiplier}

                // if theres a bpm change, adjust the current time to that of the bpm change
                let next_bar_time = beatmap.beat_length_at(time, false) * BAR_SPACING; // bar spacing is actually the timing point measure

                // edge case for aspire maps
                if next_bar_time.is_nan() || next_bar_time == 0.0 {
                    break;
                }

                // add timing bar at current time
                self.timing_bars.push(TimingBar::new(time, sv, self.taiko_settings.clone(), self.playfield.clone()));

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
        if self.note_index > 0 { return }

        // TODO: self.playfield.window_size.x
        let x_needed = self.playfield.size.x as f32;
        let mut time = self.end_time; //manager.time();

        for i in self.notes.iter().rev() {
            let time_at = i.time_at(x_needed);
            time = time.min(time_at)
        }

        if manager.time() >= time { return }

        if manager.lead_in_time > 0.0 {
            if time > manager.lead_in_time {
                time -= manager.lead_in_time - 0.01;
                manager.lead_in_time = 0.01;
            }
        }
        
        if time < 0.0 { return }
        #[cfg(feature="bass_audio")]
        manager.song.set_position(time as f64).unwrap();
        #[cfg(feature="neb_audio")]
        manager.song.upgrade().unwrap().set_position(time);
    }

    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.playfield = Arc::new(TaikoPlayfield { pos: Vector2::zero(), size: window_size.0 });
        
        // update notes
        for note in self.notes.iter_mut() { 
            note.playfield_changed(self.playfield.clone());
        }

        for tb in self.timing_bars.iter_mut() {
            tb.playfield_changed(self.playfield.clone());
        }
    }

    async fn fit_to_area(&mut self, pos:Vector2, mut size:Vector2) {
        size.x = self.playfield.size.x;
        self.playfield = Arc::new(TaikoPlayfield { pos, size });
        
        // update notes
        for note in self.notes.iter_mut() { 
            note.playfield_changed(self.playfield.clone());
        }
        
        for tb in self.timing_bars.iter_mut() {
            tb.playfield_changed(self.playfield.clone());
        }
    }


    async fn force_update_settings(&mut self, settings: &Settings) {
        let old_sv_mult = self.taiko_settings.sv_multiplier;
        let sv_static = self.current_mods.has_mod("no_sv");
        
        let mut settings = settings.taiko_settings.clone();
        // calculate the hit area
        settings.init_settings().await;
        let settings = Arc::new(settings);
        self.taiko_settings = settings.clone();


        // update notes
        for n in self.notes.iter_mut() {
            n.set_settings(settings.clone());

            // set note svs
            if sv_static {
                n.set_sv(self.taiko_settings.sv_multiplier);
            } else {
                let sv = if sv_static {
                    1.0
                } else {
                    n.get_sv() / old_sv_mult
                } * self.taiko_settings.sv_multiplier;
                n.set_sv(sv);
            }
        }


        // update bars
        for bar in self.timing_bars.iter_mut() {
            bar.set_settings(settings.clone());

            if sv_static {
                bar.speed = self.taiko_settings.sv_multiplier;
            } else {
                let sv = if sv_static {
                    1.0
                } else {
                    bar.speed / old_sv_mult
                } * self.taiko_settings.sv_multiplier;
                bar.speed = sv;
            }
        }


        // update images
        let radius = settings.note_radius * settings.hit_area_radius_mult;
        let scale = Vector2::one() * (radius * 2.0) / TAIKO_HIT_INDICATOR_TEX_SIZE.x;

        for i in [ &mut self.left_don_image, &mut self.right_kat_image ] {
            if let Some(i) = i {
                i.initial_scale = scale;
                i.current_scale = scale;
            }
        }
        
        for i in [ &mut self.left_kat_image, &mut self.right_don_image] {
            let scale = scale * Vector2::new(-1.0, 1.0);
            if let Some(i) = i {
                i.initial_scale = scale;
                i.current_scale = scale;
            }
        }
        



    }
    async fn reload_skin(&mut self) {
        if let Some(don) = &mut SkinManager::get_texture("taiko-drum-inner", true).await {
            don.depth = -1.0;
            don.origin.x = don.tex_size().x;
            don.current_pos = self.taiko_settings.hit_position;
            
            let radius = self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult;
            let scale = Vector2::one() * (radius * 2.0) / TAIKO_HIT_INDICATOR_TEX_SIZE.x;

            self.left_don_image = Some(don.clone());
            self.left_don_image.as_mut().unwrap().current_scale = scale;

            self.right_don_image = Some(don.clone());
            self.right_don_image.as_mut().unwrap().current_scale = Vector2::new(-1.0, 1.0) * scale;
        }
        if let Some(kat) = &mut SkinManager::get_texture("taiko-drum-outer", true).await {
            kat.depth = -1.0;
            kat.origin.x = 0.0;
            kat.current_pos = self.taiko_settings.hit_position;
            
            let radius = self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult;
            let scale = Vector2::one() * (radius * 2.0) / TAIKO_HIT_INDICATOR_TEX_SIZE.x;
            
            self.left_kat_image = Some(kat.clone());
            self.left_kat_image.as_mut().unwrap().current_scale = Vector2::new(-1.0, 1.0) * scale;

            self.right_kat_image = Some(kat.clone());
            self.right_kat_image.as_mut().unwrap().current_scale = scale;
        }

        self.judgement_helper = JudgmentImageHelper::new(TaikoHitJudgments::Miss).await;

        for n in self.notes.iter_mut() {
            n.reload_skin().await;
        }
    }

    
    async fn apply_mods(&mut self, mods: Arc<ModManager>) {
        let old_sv_mult = self.taiko_settings.sv_multiplier;
        let old_sv_static = self.current_mods.has_mod("no_sv");
        let current_sv_static = mods.has_mod("no_sv");
        self.current_mods = mods;
        
        if current_sv_static != old_sv_static {
            for n in self.notes.iter_mut() {

                // set note svs
                if current_sv_static {
                    n.set_sv(self.taiko_settings.sv_multiplier);
                } else {
                    let sv = if old_sv_static {
                        n.get_sv()
                    } else {
                        n.get_sv() / old_sv_mult
                    } * self.taiko_settings.sv_multiplier;
                    n.set_sv(sv);
                }
            }


            // update bars
            for bar in self.timing_bars.iter_mut() {
                if current_sv_static {
                    bar.speed = self.taiko_settings.sv_multiplier;
                } else {
                    let sv = if old_sv_static {
                        bar.speed
                    } else {
                        bar.speed / old_sv_mult
                    } * self.taiko_settings.sv_multiplier;
                    bar.speed = sv;
                }
            }
        }

    }
}

#[async_trait]
impl GameModeInput for TaikoGame {

    async fn key_down(&mut self, key:piston::Key, manager:&mut IngameManager) {
        // dont accept key input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.has_autoplay() || manager.replaying {
            return;
        }

        let time = manager.time();

        if key == self.taiko_settings.left_kat {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftKat), time, manager).await;
        }
        if key == self.taiko_settings.left_don {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftDon), time, manager).await;
        }
        if key == self.taiko_settings.right_don {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::RightDon), time, manager).await;
        }
        if key == self.taiko_settings.right_kat {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::RightKat), time, manager).await;
        }
    }
    
    async fn key_up(&mut self, _key:piston::Key, _manager:&mut IngameManager) {}


    async fn mouse_down(&mut self, btn:piston::MouseButton, manager:&mut IngameManager) {
        
        // dont accept mouse input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.has_autoplay() || manager.replaying || self.taiko_settings.ignore_mouse_buttons {
            return;
        }
        
        let time = manager.time();
        match btn {
            piston::MouseButton::Left => self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftDon), time, manager).await,
            piston::MouseButton::Right => self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftKat), time, manager).await,
            _ => {}
        }
    }

    // async fn mouse_up(&mut self, btn:piston::MouseButton, manager:&mut IngameManager) {
        
    //     // dont accept mouse input when autoplay is enabled, or a replay is being watched
    //     if manager.current_mods.autoplay || manager.replaying || self.taiko_settings.ignore_mouse_buttons {
    //         return;
    //     }
        
    //     let time = manager.time();
    //     match btn {
    //         piston::MouseButton::Left => self.handle_replay_frame(ReplayFrame::Release(KeyPress::LeftDon), time, manager),
    //         piston::MouseButton::Right => self.handle_replay_frame(ReplayFrame::Release(KeyPress::LeftKat), time, manager),
    //         _ => {}
    //     }
    // }


    async fn controller_press(&mut self, c: &Box<dyn Controller>, btn: u8, manager:&mut IngameManager) {
        // dont accept controller input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.has_autoplay() || manager.replaying {
            return;
        }

        if let Some(c_config) = self.taiko_settings.clone().controller_config.get(&*c.get_name()) {
            let time = manager.time();

            if c_config.left_kat.check_button(btn) {
                self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftKat), time, manager).await;
            }

            if c_config.left_don.check_button(btn) {
                self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftDon), time, manager).await;
            }

            if c_config.right_don.check_button(btn) {
                self.handle_replay_frame(ReplayFrame::Press(KeyPress::RightDon), time, manager).await;
            }

            if c_config.right_kat.check_button(btn) {
                self.handle_replay_frame(ReplayFrame::Press(KeyPress::RightKat), time, manager).await;
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
                settings.save().await;
            }
            
            self.taiko_settings = Arc::new(new_settings);
            // rerun the handler now that the thing is setup
            self.controller_press(c, btn, manager).await;
        }
    }

    async fn controller_release(&mut self, c: &Box<dyn Controller>, btn: u8, manager:&mut IngameManager) {
        // dont accept controller input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.has_autoplay() || manager.replaying {
            return;
        }

        if let Some(c_config) = self.taiko_settings.clone().controller_config.get(&*c.get_name()) {
            let time = manager.time();

            if c_config.left_kat.check_button(btn) {
                self.handle_replay_frame(ReplayFrame::Release(KeyPress::LeftKat), time, manager).await;
            }

            if c_config.left_don.check_button(btn) {
                self.handle_replay_frame(ReplayFrame::Release(KeyPress::LeftDon), time, manager).await;
            }

            if c_config.right_don.check_button(btn) {
                self.handle_replay_frame(ReplayFrame::Release(KeyPress::RightDon), time, manager).await;
            }

            if c_config.right_kat.check_button(btn) {
                self.handle_replay_frame(ReplayFrame::Release(KeyPress::RightKat), time, manager).await;
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
                settings.save().await;
            }
            
            self.taiko_settings = Arc::new(new_settings);
            // rerun the handler now that the thing is setup
            self.controller_release(c, btn, manager).await;
        }
    }

}
#[async_trait]
impl GameModeProperties for TaikoGame {
    fn playmode(&self) -> PlayMode {"taiko".to_owned()}
    fn end_time(&self) -> f32 {self.end_time}

    fn get_possible_keys(&self) -> Vec<(KeyPress, &str)> {
        vec![
            (KeyPress::LeftKat, "LK"),
            (KeyPress::LeftDon, "LD"),
            (KeyPress::RightDon, "RD"),
            (KeyPress::RightKat, "RK"),
        ]
    }

    fn timing_bar_things(&self) -> Vec<(f32,Color)> {
        self.hit_windows
            .iter()
            .map(|(j, w) | {
                (w.end, j.color())
            })
            .collect()
    }

    async fn get_ui_elements(&self, _window_size: Vector2, ui_elements: &mut Vec<UIElement>) {
        let playmode = self.playmode();
        let get_name = |name| {
            format!("{playmode}_{name}")
        };

        let combo_bounds = Rectangle::bounds_only(
            Vector2::zero(),
            Vector2::new(self.taiko_settings.hit_position.x - self.taiko_settings.note_radius, self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult)
        );
        
        // combo
        ui_elements.push(UIElement::new(
            &get_name("combo".to_owned()),
            Vector2::new(0.0, self.taiko_settings.hit_position.y - self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult/2.0),
            ComboElement::new(combo_bounds).await
        ).await);

        // Leaderboard
        ui_elements.push(UIElement::new(
            &get_name("leaderboard".to_owned()),
            Vector2::y_only(self.taiko_settings.hit_position.y + self.taiko_settings.note_radius * self.taiko_settings.big_note_multiplier + 50.0),
            LeaderboardElement::new()
        ).await);

        // don chan
        ui_elements.push(UIElement::new(
            &get_name("don_chan".to_owned()),
            self.taiko_settings.get_playfield(0.0, false).current_pos,
            DonChan::new().await
        ).await);
    }

}


fn add_hit_indicator(time: f32, mut hit_value: &TaikoHitJudgments, finisher_hit: bool, game_settings: &Arc<TaikoSettings>, judgment_helper: &JudgmentImageHelper, manager: &mut IngameManager) {
    let pos = game_settings.hit_position + Vector2::y_only(game_settings.judgement_indicator_offset);

    // if finisher, upgrade to geki or katu
    if finisher_hit {
        if let &TaikoHitJudgments::X100 = hit_value {
            hit_value = &TaikoHitJudgments::Katu;
        } else if let &TaikoHitJudgments::X300 = hit_value {
            hit_value = &TaikoHitJudgments::Geki;
        }
    }

    let color = hit_value.color();
    let mut image = judgment_helper.get_from_scorehit(hit_value);
    if let Some(image) = &mut image {
        image.current_pos = pos;
        image.depth = -2.0;

        let radius = game_settings.note_radius * game_settings.big_note_multiplier * game_settings.hit_area_radius_mult;
        let scale = Vector2::one() * (radius * 2.0) / TAIKO_JUDGEMENT_TEX_SIZE;
        image.initial_scale = scale;
        image.current_scale = scale;
    }

    manager.add_judgement_indicator(BasicJudgementIndicator::new(
        pos, 
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
    playfield: Arc<TaikoPlayfield>,
    size: Vector2
}
impl TimingBar {
    pub fn new(time:f32, speed:f32, settings: Arc<TaikoSettings>, playfield: Arc<TaikoPlayfield>) -> TimingBar {
        let size = Vector2::new(BAR_WIDTH, settings.get_playfield(0.0, false).size.y);

        TimingBar {
            time, 
            speed,
            pos: Vector2::new(0.0, settings.hit_position.y - size.y/2.0),
            settings,
            playfield,
            size
        }
    }

    pub fn update(&mut self, time:f32) {
        self.pos.x = self.settings.hit_position.x + self.x_at(time) as f64 - BAR_WIDTH / 2.0;
    }

    fn x_at(&self, time: f32) -> f32 {
        ((self.time - time) / SV_OVERRIDE) * self.speed * self.playfield.size.x as f32
    }
    fn draw(&mut self, args:RenderArgs, list:&mut Vec<Box<dyn Renderable>>){
        if self.pos.x + BAR_WIDTH < 0.0 || self.pos.x - BAR_WIDTH > args.window_size[0] {return}

        list.push(Box::new(Rectangle::new(
            BAR_COLOR,
            1001.5,
            self.pos,
            self.size,
            None
        )));
    }

    fn playfield_changed(&mut self, new: Arc<TaikoPlayfield>) {
        self.playfield = new;
    }

    fn set_settings(&mut self, settings: Arc<TaikoSettings>) {
        self.settings = settings;
        self.size = Vector2::new(BAR_WIDTH, self.settings.get_playfield(0.0, false).size.y);
        self.pos = Vector2::new(0.0, self.settings.hit_position.y - self.size.y/2.0);
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
            note_index: -1, 
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

pub struct TaikoPlayfield {
    pub pos: Vector2,
    pub size: Vector2,
}


#[derive(Default)]
pub struct FullAltCounter {
    // hits: HashMap<TaikoHit, usize>,
    last_hit: Option<TaikoHit>,
    // playmode: TaikoPlaymode
}
impl FullAltCounter {
    pub fn new() -> Self {
        Self::default()
    }

    fn add_hit(&mut self, hit: TaikoHit) -> bool {

        if self.last_hit.is_none() {
            self.last_hit = Some(hit);
            return true;
        }

        let is_left = Self::hit_is_left(hit);
        let last_is_left = Self::hit_is_left(self.last_hit.unwrap());
        self.last_hit = Some(hit);
        
        is_left != last_is_left
    }

    fn hit_is_left(hit: TaikoHit) -> bool {
        match hit {
            TaikoHit::LeftKat | TaikoHit::LeftDon => true,
            TaikoHit::RightDon | TaikoHit::RightKat => false,
        }
    }

}


#[inline]
pub fn scale_by_mods<V:std::ops::Mul<Output=V>>(val:V, ez_scale: V, hr_scale: V, mods: &ModManager) -> V {
    if mods.mods.contains(Easy.name()) {
        val * ez_scale
    } else if mods.mods.contains(HardRock.name()) {
        val * hr_scale
    } else {
        val
    }
}


#[inline]
pub fn get_od(meta: &BeatmapMeta, mods: &ModManager) -> f32 {
    scale_by_mods(meta.od, 0.5, 1.4, mods).clamp(1.0, 10.0)
}
