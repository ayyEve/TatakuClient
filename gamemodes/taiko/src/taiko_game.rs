/**
 * Taiko game mode
 * Author: ayyEve
 * 
 * NOTE! gekis and katus are for DISPLAY ONLY!!
 * they are not factored into acc!!
*/

use crate::prelude::*;

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


pub const FINISHER_LENIENCY:f32 = 20.0; // ms
pub const NOTE_BORDER_SIZE:f32 = 2.0;

pub const GRAVITY_SCALING:f32 = 400.0;


pub struct TaikoGame {
    // lists
    pub notes: TaikoNoteQueue,
    pub other_notes: TaikoNoteQueue,

    timing_bars: Vec<TimingBar>,

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
    
    hit_windows: Vec<(HitJudgment, Range<f32>)>,
    hit_cache: HashMap<TaikoHit, f32>,
    miss_window: f32,

    last_judgment: HitJudgment,
    current_mods: Arc<ModManager>,
    healthbar_swap_pending: bool,
}
impl TaikoGame {
    async fn play_sound (
        &self, 
        state: &mut GameplayStateForUpdate<'_>, 
        note_time: f32, 
        hit_type: HitType, 
        finisher: bool,
    ) {
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

        let hitsound = Hitsound::from_hitsamples(
            hitsound, 
            samples, 
            false, 
            state.timing_points.timing_point_at(note_time, true)
        );
        state.add_action(GamemodeAction::play_hitsounds(hitsound));
        // manager.play_note_sound(&hitsound).await;
    }

    async fn setup_hitwindows(&mut self) {
        let od = Self::get_od(&self.metadata, &self.current_mods);

        // windows
        let w_miss = map_difficulty(od, 135.0, 95.0, 70.0);
        let w_100 = map_difficulty(od, 120.0, 80.0, 50.0);
        let w_300 = map_difficulty(od, 50.0, 35.0, 20.0);

        // use TaikoHitJudgments::*;
        self.hit_windows = vec![
            (TaikoHitJudgments::X300, 0.0..w_300),
            (TaikoHitJudgments::X100, w_300..w_100),
            (TaikoHitJudgments::Miss, w_100..w_miss),
        ];
        self.miss_window = w_miss;

        let diff_map = map_difficulty(od, 3.0, 5.0, 7.5);
        for note in self.other_notes.iter_mut() {
            if note.note_type() == NoteType::Spinner {
                let length = note.end_time(0.0) - note.time();
                let required_hits = ((length / 1000.0 * diff_map) * 1.65).max(1.0) as u16; 
                note.set_required_hits(required_hits);
            }
        }
    }


    fn add_hit_indicator(
        mut hit_value: &HitJudgment, 
        finisher_hit: bool, 
        game_settings: &TaikoSettings, 
        playfield: &TaikoPlayfield,
        judgment_helper: &JudgmentImageHelper, 
        state: &mut GameplayStateForUpdate<'_>
    ) {
        let pos = playfield.hit_position + Vector2::with_y(game_settings.judgement_indicator_offset);

        // if finisher, upgrade to geki or katu
        if finisher_hit {
            // remove the normal hit indicator, its being replaced with a finisher
            state.add_action(GamemodeAction::RemoveLastJudgment);
            // manager.judgement_indicators.pop();

            if hit_value == &TaikoHitJudgments::X100 {
                hit_value = &TaikoHitJudgments::Katu;
            } else if hit_value == &TaikoHitJudgments::X300 {
                hit_value = &TaikoHitJudgments::Geki;
            }
        }

        let color = hit_value.color;
        let mut image = if game_settings.use_skin_judgments { judgment_helper.get_from_scorehit(hit_value) } else { None };
        if let Some(image) = &mut image {
            image.pos = pos;

            let radius = game_settings.note_radius * game_settings.big_note_multiplier; // * game_settings.hit_area_radius_mult;
            image.scale = Vector2::ONE * (radius * 2.0) / TAIKO_JUDGEMENT_TEX_SIZE;
        }

        state.add_indicator(BasicJudgementIndicator::new(
            pos, 
            state.time,
            game_settings.note_radius * 0.5 * if finisher_hit { game_settings.big_note_multiplier } else { 1.0 },
            color,
            image
        ))
    }

    #[inline]
    pub fn scale_by_mods<V:std::ops::Mul<Output=V>>(val:V, ez_scale: V, hr_scale: V, mods: &ModManager) -> V {
        if mods.has_mod(Easy) {
            val * ez_scale
        } else if mods.has_mod(HardRock) {
            val * hr_scale
        } else {
            val
        }
    }


    #[inline]
    pub fn get_od(meta: &BeatmapMeta, mods: &ModManager) -> f32 {
        Self::scale_by_mods(meta.od, 0.5, 1.4, mods).clamp(1.0, 10.0)
    }


    

    pub fn get_taiko_playfield(settings: &TaikoSettings, bounds: Bounds) -> TaikoPlayfield {
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

        TaikoPlayfield {
            bounds,
            height,
            hit_position
        }
    } 

    fn update_playfield(&mut self, bounds: Bounds) {
        self.playfield = Arc::new(Self::get_taiko_playfield(&self.taiko_settings, bounds));

        // update notes
        for note in self.notes.iter_mut().chain(self.other_notes.iter_mut()) { 
            note.playfield_changed(self.playfield.clone());
        }

        // update timing bars
        for tb in self.timing_bars.iter_mut() {
            tb.playfield_changed(self.playfield.clone());
        }

        // update hit indicator sprite positions
        for i in [ &mut self.left_kat_image, &mut self.left_don_image, &mut self.right_don_image, &mut self.right_kat_image ] {
            i.as_mut().map(|i|i.pos = self.playfield.hit_position);
        }
    }
}

#[async_trait]
impl GameMode for TaikoGame {
    async fn new(beatmap:&Beatmap, _diff_calc_only:bool) -> TatakuResult<Self> {
        let metadata = beatmap.get_beatmap_meta();
        let settings = Arc::new(Settings::get().taiko_settings.clone());

        let playfield = Arc::new(Self::get_taiko_playfield(&settings, Bounds::new(Vector2::ZERO, WindowSize::get().0)));

        let mut hit_cache = HashMap::new();
        let left_kat_image = None;
        let left_don_image = None;
        let right_don_image = None;
        let right_kat_image = None;
        let judgement_helper = JudgmentImageHelper::new(TaikoHitJudgments::variants().to_vec()).await;

        for i in [TaikoHit::LeftKat, TaikoHit::LeftDon, TaikoHit::RightDon, TaikoHit::RightKat] {
            hit_cache.insert(i, -999.9);
        }

        let timing_points = TimingPointHelper::new(beatmap.get_timing_points(), beatmap.slider_velocity());

        let mut s = Self {
            notes: TaikoNoteQueue::new(),
            other_notes: TaikoNoteQueue::new(),

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
            current_mods: Arc::new(ModManager::new()),
            healthbar_swap_pending: false
        };

        match beatmap {
            Beatmap::Osu(beatmap) => {
                // add notes
                for note in beatmap.notes.iter() {
                    let hit_type = if (note.hitsound & (2 | 8)) > 0 {HitType::Kat} else {HitType::Don};
                    let finisher = (note.hitsound & 4) > 0;

                    s.notes.push(Box::new(TaikoNote::new(
                        note.time,
                        hit_type,
                        finisher,
                        settings.clone(),
                        playfield.clone(),
                    ).await));
                }
                for slider in beatmap.sliders.iter() {
                    let SliderDef {time, slides, length, ..} = slider.to_owned();
                    let finisher = (slider.hitsound & 4) > 0;

                    let l = (length * 1.4) * slides as f32;
                    let v2 = 100.0 * (beatmap.slider_multiplier * 1.4);
                    let bl = timing_points.beat_length_at(time, true);
                    let end_time = time + (l / v2 * bl);
                    
                    // convert vars
                    let v = timing_points.slider_velocity_at(time);
                    let bl = timing_points.beat_length_at(time, beatmap.beatmap_version < 8);
                    let skip_period = (bl / beatmap.slider_tick_rate).min((end_time - time) / slides as f32);

                    if skip_period > 0.0 && beatmap.metadata.mode != "taiko" && l / v * 1000.0 < 2.0 * bl {
                        let mut i = 0;
                        let mut j = time;

                        // load sounds
                        // let sound_list_raw = if let Some(list) = split.next() {list.split("|")} else {"".split("")};

                        // when loading, if unified just have it as sound_types with 1 index
                        let mut sound_types:Vec<(HitType, bool)> = Vec::new();

                        for hitsound in slider.edge_sounds.iter() {
                            let hit_type = if (hitsound & (2 | 8)) > 0 { HitType::Kat } else { HitType::Don };
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

                            s.notes.push(Box::new(TaikoNote::new(
                                j,
                                sound_type.0,
                                sound_type.1,
                                settings.clone(),
                                playfield.clone(),
                            ).await));

                            if !unified_sound_addition { i = (i + 1) % sound_types.len() }

                            j += skip_period;
                            if !(j < end_time + skip_period / 8.0) { break }
                        }
                    } else {
                        s.other_notes.push(Box::new(TaikoDrumroll::new(
                            time, 
                            end_time, 
                            finisher, 
                            settings.clone(),
                            playfield.clone(),
                        ).await));
                    }
                }
                for spinner in beatmap.spinners.iter() {
                    s.other_notes.push(Box::new(TaikoSpinner::new(
                        spinner.time,
                        spinner.end_time, 
                        0, 
                        settings.clone(),
                        playfield.clone(),
                    ).await));
                }
            }

            Beatmap::Tja(beatmap) => {
                for note in beatmap.circles.iter() {
                    s.notes.push(Box::new(TaikoNote::new(
                        note.time,
                        if note.is_don {HitType::Don} else {HitType::Kat},
                        note.is_big,
                        settings.clone(),
                        playfield.clone(),
                    ).await));
                }

                for drumroll in beatmap.drumrolls.iter() {
                    s.other_notes.push(Box::new(TaikoDrumroll::new(
                        drumroll.time, 
                        drumroll.end_time, 
                        drumroll.is_big, 
                        settings.clone(),
                        playfield.clone(),
                    ).await));
                }

                for balloon in beatmap.balloons.iter() {
                    s.other_notes.push(Box::new(TaikoSpinner::new(
                        balloon.time,
                        balloon.end_time, 
                        balloon.hits_required as u16, 
                        settings.clone(),
                        playfield.clone(),
                    ).await));
                }
            }
            _ => return Err(BeatmapError::UnsupportedMode.into()),
        };

        if s.notes.len() + s.other_notes.len() == 0 { return Err(TatakuError::Beatmap(BeatmapError::InvalidFile)) }
        s.notes.sort_by(|a, b|a.time().partial_cmp(&b.time()).unwrap());
        s.other_notes.sort_by(|a, b|a.time().partial_cmp(&b.time()).unwrap());

        // theres probably a better way to do this lol
        if let Some(last) = s.notes.last() {
            s.end_time = s.end_time.max(last.end_time(0.0));
        }
        if let Some(last) = s.other_notes.last() {
            s.end_time = s.end_time.max(last.end_time(0.0));
        }
        s.end_time += 1000.0;

        s.setup_hitwindows().await;

        // // i wonder if not doing this has been causing issues
        // s.apply_mods(s.current_mods.clone()).await;

        Ok(s)
    }

    async fn handle_replay_frame<'a>(
        &mut self, 
        frame: ReplayFrame, 
        state: &mut GameplayStateForUpdate<'a>
    ) {
        let ReplayAction::Press(key) = frame.action else { return };

        // turn the keypress into a hit type
        let taiko_hit_type = match key {
            KeyPress::LeftKat  => TaikoHit::LeftKat,
            KeyPress::LeftDon  => TaikoHit::LeftDon,
            KeyPress::RightDon => TaikoHit::RightDon,
            KeyPress::RightKat => TaikoHit::RightKat,
            _ => TaikoHit::LeftKat
        };
        let is_left = taiko_hit_type == TaikoHit::LeftKat || taiko_hit_type == TaikoHit::LeftDon;
        
        if is_left { state.add_stat(TaikoStatLeftPresses, 1.0) }
        else { state.add_stat(TaikoStatRightPresses, 1.0) }

        // check fullalt
        if state.mods.has_mod(FullAlt) {
            if !self.counter.add_hit(taiko_hit_type) {
                return;
            }
        }

        let mut hit_type:HitType = key.into();
        let mut finisher_sound = false;
        // let mut sound = match hit_type {HitType::Don => "don", HitType::Kat => "kat"};

        let mut hit_time = frame.time;
        let has_relax = state.mods.has_mod(Relax);

        let mut did_hit = false;
        for queue in [&mut self.notes, &mut self.other_notes] {
            // if theres no more notes to hit, return after playing the sound
            if queue.done() { continue; }

            // check for finisher 2nd hit. 
            if !did_hit && self.last_judgment != TaikoHitJudgments::Miss {
                if let Some(last_note) = queue.previous_note() {
                    if last_note.check_finisher(hit_type, frame.time, state.game_speed) {

                        // i cant match on these contants bc i dont use the derive macro :c
                        // let j = match &self.last_judgment {
                        //     &TaikoHitJudgments::X300 | &TaikoHitJudgments::Geki => &TaikoHitJudgments::Geki,
                        //     &TaikoHitJudgments::X100 | &TaikoHitJudgments::Katu => &TaikoHitJudgments::Katu,
                        //     _ => return, // this shouldnt happen, last judgment will always be one of the above
                        // };
                        let j = if [&TaikoHitJudgments::X300, &TaikoHitJudgments::Geki].contains(&&self.last_judgment) {
                            &TaikoHitJudgments::Geki
                        } else if [&TaikoHitJudgments::X100, &TaikoHitJudgments::Katu].contains(&&self.last_judgment) {
                            &TaikoHitJudgments::Katu
                        } else {
                            return
                        };

                        // add whatever the last judgment was as a finisher score
                        state.add_judgment(*j);
                        Self::add_hit_indicator(
                            j, 
                            true, 
                            &self.taiko_settings, 
                            &self.playfield, 
                            &self.judgement_helper, 
                            state
                        );

                        // draw drum
                        *self.hit_cache.get_mut(&taiko_hit_type).unwrap() = state.time;

                        return; // return and note continue because we dont want the 2nd finisher press to count towards anything
                    }
                }
            }

            // check note hit
            if let Some(note) = queue.current_note() {
                let note_time = note.time();
                match note.note_type() {
                    NoteType::Note => {
                        let cond = || note.hit_type() == hit_type || has_relax;

                        let hit_maybe = state.check_judgment_condition(
                            &self.hit_windows, 
                            frame.time, 
                            note_time, 
                            cond, 
                            &TaikoHitJudgments::Miss
                        ).await;

                        if let Some(judge) = hit_maybe {
                            // if note.finisher_sound() { sound = match hit_type { HitType::Don => "bigdon", HitType::Kat => "bigkat" } }
                            finisher_sound = note.finisher_sound();
                            if has_relax {
                                hit_type = note.hit_type();
                            }

                            if judge == &TaikoHitJudgments::Miss {
                                note.miss(state.time);
                            } else {
                                note.hit(state.time);
                            }

                            Self::add_hit_indicator(judge, false, &self.taiko_settings, &self.playfield, &self.judgement_helper, state);
                            
                            self.last_judgment = *judge;
                            queue.next();
                        }
                    }

                    // slider or spinner, special hit stuff
                    NoteType::Slider  if note.hit(state.time) => state.add_judgment(TaikoHitJudgments::SliderPoint),
                    NoteType::Spinner if note.hit(state.time) => state.add_judgment(TaikoHitJudgments::SpinnerPoint),
                    _ => {}
                }

                // if was hit, the sound already played
                if !did_hit {
                    hit_time = note_time;
                    // self.play_sound(manager, note_time, hit_type, finisher_sound).await;
                }
            }
            
            did_hit = true;
        }

        // account for relax changing the hit type
        let new_hit_type = match (is_left, hit_type) {
            (false, HitType::Don) => TaikoHit::RightDon,
            (true, HitType::Don) => TaikoHit::LeftDon,
            (true, HitType::Kat) => TaikoHit::LeftKat,
            (false, HitType::Kat) => TaikoHit::RightKat,
        };

        // draw drum
        *self.hit_cache.get_mut(&new_hit_type).unwrap() = frame.time;

        // play sound
        self.play_sound(state, hit_time, hit_type, finisher_sound).await;
    }


    async fn update<'a>(&mut self, state: &mut GameplayStateForUpdate<'a>) {

        // check healthbar swap
        if self.healthbar_swap_pending {
            self.healthbar_swap_pending = false;
            // println!("swapping health");

            // reset health helper to default
            state.add_action(GamemodeAction::ResetHealth); // manager.health = Default::default();

            // if we're using battery health
            if !self.current_mods.has_mod(NoBattery) {
                let note_count = self.notes.iter().filter(|n| n.note_type() == NoteType::Note).count() as f32;

                const MAX_HEALTH:f32 = 200.0;

                // this is essentially stolen from peppy's 2016 osu code
                let normal_health = MAX_HEALTH / (0.06 * 6.0 * note_count * map_difficulty(self.metadata.hp, 0.5, 0.75, 0.98));
                
                // println!("normal health: {normal_health}");
                
                // random fudge because osu makes no sense
                const FACTOR: f32 = 15.0;
                let normal_health = normal_health / FACTOR;
                // println!("normal health: {normal_health}");

                let health_per_300 = normal_health * 6.0;
                let health_per_100 = normal_health * map_difficulty(self.metadata.hp, 6.0, 2.2, 2.2);
                let health_per_miss = map_difficulty(self.metadata.hp, -6.0, -25.0, -40.0) / FACTOR;

                
                // println!("note count: {note_count}");
                // println!("health_per_300: {health_per_300}");
                // println!("health_per_100: {health_per_100}");
                // println!("health_per_miss: {health_per_miss}");

                state.add_action(GamemodeAction::replace_health(TaikoBatteryHealthManager::new(
                    health_per_300,
                    health_per_100,
                    health_per_miss
                )));

                // let health = &mut manager.health;
                // health.max_health = MAX_HEALTH;
                // health.current_health = 0.0;
                // health.initial_health = 0.0;
                // health.check_fail_at_end = true;

                // health.check_fail = Arc::new(move |s| s.current_health < PASS_HEALTH);
                // health.do_health = Arc::new(move |s, j, _score| {
                //     s.current_health += match j.id {
                //         "x300" => health_per_300,
                //         "x100" => health_per_100,
                //         "xmiss" => health_per_miss,
                //         _ => return
                //     };

                //     s.validate_health()
                // });
            }
        }

        // do autoplay things
        if state.mods.has_autoplay() {
            let mut pending_frames = Vec::new();
            let mut queues = vec![
                std::mem::take(&mut self.notes),
                std::mem::take(&mut self.other_notes),
            ];

            // get auto inputs
            self.auto_helper.update(state.time, &mut queues, &mut pending_frames);

            self.notes = queues.remove(0);
            self.other_notes = queues.remove(0);

            for frame in pending_frames.into_iter() {
                self.handle_replay_frame(ReplayFrame::new(state.time, frame), state).await;
            }

        }
        
        for queue in [&mut self.notes, &mut self.other_notes] {
            for note in queue.notes.iter_mut() {
                note.update(state.time).await;
            }

            if queue.done() {
                if !state.complete() && state.time > self.end_time {
                    state.add_action(GamemodeAction::MapComplete);
                    // manager.completed = true;
                }

                continue;
            }

            if let Some(do_miss) = queue.check_missed(state.time, self.miss_window) {
                if do_miss {
                    // queue.current_note().miss(time); // done in check_missed

                    let j = TaikoHitJudgments::Miss;
                    state.add_judgment(j);
                    Self::add_hit_indicator(
                        &j, 
                        false, 
                        &self.taiko_settings, 
                        &self.playfield, 
                        &self.judgement_helper, 
                        state
                    );
                }

                queue.next()
            }
        }

        // TODO: might move tbs to a (time, speed) tuple
        for tb in self.timing_bars.iter_mut() { tb.update(state.time); }

    }
    
    async fn draw<'a>(&mut self, state: GameplayStateForDraw<'a>, list: &mut RenderableCollection) {

        // draw the playfield
        list.push(self.playfield.get_rectangle(state.current_timing_point.kiai));
        
        // draw the hit area
        list.push(Circle::new(
            self.playfield.hit_position,
            self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult,
            Color::BLACK,
            None
        ));

        // draw timing lines
        for tb in self.timing_bars.iter_mut() { tb.draw(list) }

        // draw notes
        // higher sv notes are drawn overtop of lower sv notes
        // if sv is equal, earlier notes are drawn on top of later notes
        let mut note_list = self.notes.iter_mut().chain(self.other_notes.iter_mut()).collect::<Vec<_>>();
        note_list.sort_by(|a, b|{
            match a.get_sv().partial_cmp(&b.get_sv()).unwrap_or(core::cmp::Ordering::Equal) {
                core::cmp::Ordering::Equal => b.time().partial_cmp(&a.time()).unwrap_or(core::cmp::Ordering::Equal),
                other => other
            }
        });

        for note in note_list { 
            note.draw(state.time, list).await 
        }

        // draw hit indicators
        let lifetime_time = DRUM_LIFETIME_TIME * state.mods.get_speed();
        for (hit_type, hit_time) in self.hit_cache.iter() {
            if state.time - hit_time > lifetime_time { continue }
            let alpha = 1.0 - (state.time - hit_time) / (lifetime_time * 4.0);
            match hit_type {
                TaikoHit::LeftKat => {
                    if let Some(kat) = &self.left_kat_image {
                        let mut img = kat.clone();
                        img.color.a = alpha;
                        list.push(img);
                    } else {
                        list.push(HalfCircle::new(
                            self.playfield.hit_position,
                            self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult,
                            self.taiko_settings.kat_color.alpha(alpha),
                            true
                        ));
                    }
                }
                TaikoHit::LeftDon => {
                    if let Some(don) = &self.left_don_image {
                        let mut img = don.clone();
                        img.color.a = alpha;
                        list.push(img);
                    } else {
                        list.push(HalfCircle::new(
                            self.playfield.hit_position,
                            self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult,
                            self.taiko_settings.don_color.alpha(alpha),
                            true
                        ));
                    }
                }
                TaikoHit::RightDon => {
                    if let Some(don) = &self.right_don_image {
                        let mut img = don.clone();
                        img.color.a = alpha;
                        list.push(img);
                    } else {
                        list.push(HalfCircle::new(
                            self.playfield.hit_position,
                            self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult,
                            self.taiko_settings.don_color.alpha(alpha),
                            false
                        ));
                    }
                }
                TaikoHit::RightKat => {
                    if let Some(kat) = &self.right_kat_image {
                        let mut img = kat.clone();
                        img.color.a = alpha;
                        list.push(img);
                    } else {
                        list.push(HalfCircle::new(
                            self.playfield.hit_position,
                            self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult,
                            self.taiko_settings.kat_color.alpha(alpha),
                            false
                        ));
                    }
                }
            }
        }

        if state.mods.has_mod(Flashlight) {
            let radius = match state.score.combo {
                0..=99 => 125.0,
                100..=199 => 100.0,
                _ => 75.0
            } * self.taiko_settings.sv_multiplier * 2.0;
            let fade_radius = radius / 5.0;

            list.push(FlashlightDrawable::new(
                self.playfield.hit_position,
                radius - fade_radius,
                fade_radius,
                self.playfield.bounds,
                Color::BLACK
            ));
        }
    }

    async fn reset(&mut self, beatmap: &Beatmap) {
        let timing_points = TimingPointHelper::new(beatmap.get_timing_points(), beatmap.slider_velocity());

        for queue in [&mut self.notes, &mut self.other_notes] {
            queue.index = 0;
            
            for note in queue.iter_mut() {
                note.reset().await;

                // set note svs
                if self.current_mods.has_mod(NoSV) {
                    note.set_sv(self.taiko_settings.sv_multiplier);
                } else {
                    let sv = (timing_points.slider_velocity_at(note.time()) / SV_FACTOR) * self.taiko_settings.sv_multiplier;
                    note.set_sv(sv);
                }
            }
        }

        self.last_judgment = TaikoHitJudgments::Miss;
        self.counter = FullAltCounter::new();

        // setup timing bars
        if self.timing_bars.len() == 0 {
            // load timing bars
            let parent_tps = timing_points.iter().filter(|t|!t.is_inherited()).collect::<Vec<&TimingPoint>>();
            let mut sv = self.taiko_settings.sv_multiplier;
            let mut time = parent_tps[0].time;
            let mut tp_index = 0;
            let step = timing_points.beat_length_at(time, false);
            time %= step; // get the earliest bar line possible

            loop {
                if !self.current_mods.has_mod(NoSV) {sv = (timing_points.slider_velocity_at(time) / SV_FACTOR) * self.taiko_settings.sv_multiplier}

                // if theres a bpm change, adjust the current time to that of the bpm change
                let next_bar_time = timing_points.beat_length_at(time, false) * BAR_SPACING; // bar spacing is actually the timing point measure

                // edge case for aspire maps
                if next_bar_time.is_nan() || next_bar_time == 0.0 { break; }

                // add timing bar at current time
                self.timing_bars.push(TimingBar::new(time, sv, self.playfield.clone()));

                if tp_index < parent_tps.len() && parent_tps[tp_index].time <= time + next_bar_time {
                    time = parent_tps[tp_index].time;
                    tp_index += 1;
                    continue;
                }

                // why isnt this accounting for bpm changes? because the bpm change doesnt allways happen inline with the bar idiot
                time += next_bar_time;
                if time >= self.end_time || time.is_nan() { break }
            } 

        }
        
        // reset hitcache times
        self.hit_cache.iter_mut().for_each(|(_, t)| *t = -999.9);

        self.healthbar_swap_pending = true;
    }

    fn skip_intro(&mut self, game_time: f32) -> Option<f32> {
        let x_needed = self.playfield.pos.x + self.playfield.size.x;
        let mut time = self.end_time; //manager.time();
        
        for queue in [&self.notes, &self.other_notes] {
            if queue.index > 0 { return None }

            for i in queue.notes.iter().rev() {
                let time_at = i.time_at(x_needed);
                time = time.min(time_at)
            }

        }

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

    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.update_playfield(Bounds::new(Vector2::ZERO, window_size.0));
    }

    async fn fit_to_area(&mut self, bounds: Bounds) {
        self.update_playfield(bounds);
    }


    async fn force_update_settings(&mut self, settings: &Settings) {
        let settings = settings.taiko_settings.clone();

        if &settings == &*self.taiko_settings { return }
        let settings = Arc::new(settings);
        let playfield = Arc::new(Self::get_taiko_playfield(&settings, self.playfield.bounds));
        self.playfield = playfield.clone();

        let old_sv_mult = self.taiko_settings.sv_multiplier;
        let sv_static = self.current_mods.has_mod(NoSV);

        self.taiko_settings = settings.clone();


        // update notes
        for n in self.notes.iter_mut().chain(self.other_notes.iter_mut()) {
            n.set_settings(settings.clone());
            n.playfield_changed(playfield.clone());

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
            bar.playfield_changed(playfield.clone());

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
        let scale = Vector2::ONE * (radius * 2.0) / TAIKO_HIT_INDICATOR_TEX_SIZE.x;

        for i in [ &mut self.left_don_image, &mut self.right_kat_image ] {
            if let Some(i) = i {
                i.scale = scale;
                i.pos = self.playfield.hit_position;
            }
        }
        
        for i in [ &mut self.left_kat_image, &mut self.right_don_image] {
            let scale = scale * Vector2::new(-1.0, 1.0);
            if let Some(i) = i {
                i.scale = scale;
                i.pos = self.playfield.hit_position;
            }
        }

    }
    
    #[cfg(feature="graphics")]
    async fn reload_skin(&mut self, beatmap_path: &String, skin_manager: &mut dyn SkinProvider) -> TextureSource {
        let source = TextureSource::Beatmap(beatmap_path.clone()); // TODO: yeah

        let radius = self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult;
        let scale = Vector2::ONE * (radius * 2.0) / TAIKO_HIT_INDICATOR_TEX_SIZE.x;

        if let Some(mut don) = skin_manager.get_texture("taiko-drum-inner", &source, SkinUsage::Gamemode, true).await {
            don.origin.x = (don.tex_size() / don.base_scale).x;
            don.pos = self.playfield.hit_position;
            don.scale = scale;
            self.left_don_image = Some(don.clone());
            
            let mut rdon = don;
            rdon.scale *= Vector2::new(-1.0, 1.0);
            self.right_don_image = Some(rdon);
        }
        if let Some(mut kat) = skin_manager.get_texture("taiko-drum-outer", &source, SkinUsage::Gamemode, true).await {
            kat.origin.x = 0.0;
            kat.pos = self.playfield.hit_position;
            kat.scale = scale;
            self.right_kat_image = Some(kat.clone());

            let mut lkat = kat; 
            lkat.scale *= Vector2::new(-1.0, 1.0);
            self.left_kat_image = Some(lkat);
        }

        self.judgement_helper = JudgmentImageHelper::new(TaikoHitJudgments::variants().to_vec()).await;

        for n in self.notes.iter_mut().chain(self.other_notes.iter_mut()) {
            n.reload_skin(&source, skin_manager).await;
        }

        source
    }

    
    async fn apply_mods(&mut self, mods: Arc<ModManager>) {
        let old_sv_mult = self.taiko_settings.sv_multiplier;
        let old_mods = self.current_mods.clone();

        let old_sv_static = old_mods.has_mod(NoSV);
        let current_sv_static = mods.has_mod(NoSV);
        self.current_mods = mods;

        // let old_no_finisher = old_mods.has_mod(NoFinisher);
        let new_no_finisher = self.current_mods.has_mod(NoFinisher);
        
        // update bars
        if current_sv_static != old_sv_static {
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

        // update notes
        for note in self.notes.iter_mut().chain(self.other_notes.iter_mut()) {
            
            // set note svs
            if current_sv_static != old_sv_static {
                if current_sv_static {
                    note.set_sv(self.taiko_settings.sv_multiplier);
                } else {
                    let sv = if old_sv_static {
                        note.get_sv()
                    } else {
                        note.get_sv() / old_sv_mult
                    } * self.taiko_settings.sv_multiplier;
                    note.set_sv(sv);
                }
            }

            // check nofinisher change
            note.toggle_finishers(!new_no_finisher);
        }


        if old_mods.has_mod(NoBattery) != self.current_mods.has_mod(NoBattery) {
            self.healthbar_swap_pending = true;
        }
    }

    
    async fn time_jump(&mut self, new_time: f32) {
        let mut latest_time = 0f32;
        for i in self.hit_cache.values() { latest_time = latest_time.max(*i) }
        // info!("{new_time} < {latest_time}");

        if new_time < latest_time {
            for queue in [&mut self.notes, &mut self.other_notes] {
                let mut index = 0;
                for (i, note) in queue.iter_mut().enumerate() {
                    note.reset().await;
                    if note.time() <= new_time {
                        index = i
                    }
                }
                queue.index = index;
            }
            
            // reset hitcache times
            self.hit_cache.iter_mut().for_each(|(_, t)| *t = -999.9);
        }
    }

    
    async fn beat_happened(&mut self, pulse_length: f32) {
        self.notes.iter_mut().chain(self.other_notes.iter_mut()).for_each(|n|n.beat_happened(pulse_length))
    }
    async fn kiai_changed(&mut self, is_kiai: bool) {
        self.notes.iter_mut().chain(self.other_notes.iter_mut()).for_each(|n|n.kiai_changed(is_kiai))
    }
}

#[async_trait]
#[cfg(feature="graphics")]
impl GameModeInput for TaikoGame {
    async fn key_down(&mut self, key:Key) -> Option<ReplayAction> {
        // // dont accept key input when autoplay is enabled, or a replay is being watched
        // if manager.current_mods.has_autoplay() || manager.replaying {
        //     return;
        // }

        if key == self.taiko_settings.left_kat {
            Some(ReplayAction::Press(KeyPress::LeftKat))
        } else if key == self.taiko_settings.left_don {
            Some(ReplayAction::Press(KeyPress::LeftDon))
        } else if key == self.taiko_settings.right_don {
            Some(ReplayAction::Press(KeyPress::RightDon))
        } else if key == self.taiko_settings.right_kat {
            Some(ReplayAction::Press(KeyPress::RightKat))
        } else {
            None
        }
    }
    
    async fn key_up(&mut self, key:Key) -> Option<ReplayAction> {

        if key == self.taiko_settings.left_kat {
            Some(ReplayAction::Release(KeyPress::LeftKat))
        } else if key == self.taiko_settings.left_don {
            Some(ReplayAction::Release(KeyPress::LeftDon))
        } else if key == self.taiko_settings.right_don {
            Some(ReplayAction::Release(KeyPress::RightDon))
        } else if key == self.taiko_settings.right_kat {
            Some(ReplayAction::Release(KeyPress::RightKat))
        } else {
            None
        }
    }


    async fn mouse_down(&mut self, btn:MouseButton) -> Option<ReplayAction> {
        if self.taiko_settings.ignore_mouse_buttons { return None }
        
        match btn {
            MouseButton::Left  => Some(ReplayAction::Press(KeyPress::LeftDon)),
            MouseButton::Right => Some(ReplayAction::Press(KeyPress::LeftKat)),
            _ => None
        }
    }

    async fn mouse_up(&mut self, btn:MouseButton) -> Option<ReplayAction> {
        if self.taiko_settings.ignore_mouse_buttons { return None }
        
        match btn {
            MouseButton::Left =>  Some(ReplayAction::Release(KeyPress::LeftDon)),
            MouseButton::Right => Some(ReplayAction::Release(KeyPress::LeftKat)),
            _ => None
        }
    }


    async fn controller_press(&mut self, c: &GamepadInfo, btn: ControllerButton) -> Option<ReplayAction> {

        if let Some(c_config) = self.taiko_settings.controller_config.get(&*c.name) {

            // skip
            if ControllerButton::North == btn {
                Some(ReplayAction::Press(KeyPress::SkipIntro))
            } else if c_config.left_kat.check_button(btn) {
                Some(ReplayAction::Press(KeyPress::LeftKat))
            } else if c_config.left_don.check_button(btn) {
                Some(ReplayAction::Press(KeyPress::LeftDon))
            } else if c_config.right_don.check_button(btn) {
                Some(ReplayAction::Press(KeyPress::RightDon))
            } else if c_config.right_kat.check_button(btn) {
                Some(ReplayAction::Press(KeyPress::RightKat))
            } else {
                None
            }

        } else {
            trace!("Controller with no setup");

            // TODO: if this is slow, we should store controller configs separately
            // but i dont think this will be an issue, as its unlikely to happen in the first place,
            // and if there is lag, the user is likely to retry the man anyways
            trace!("Setting up new controller");
            let mut new_settings = self.taiko_settings.as_ref().clone();
            new_settings.controller_config.insert((*c.name).clone(), TaikoControllerConfig::defaults(c.name.clone()));

            // update the global settings
            {
                let mut settings = Settings::get_mut();
                settings.taiko_settings = new_settings.clone();
                // settings.save().await;
            }
            
            self.taiko_settings = Arc::new(new_settings);
            // rerun the handler now that the thing is setup
            self.controller_press(c, btn).await
        }
    }

    async fn controller_release(&mut self, c: &GamepadInfo, btn: ControllerButton) -> Option<ReplayAction> {
        if let Some(c_config) = self.taiko_settings.controller_config.get(&*c.name) {
            if c_config.left_kat.check_button(btn) {
                Some(ReplayAction::Release(KeyPress::LeftKat))
            } else if c_config.left_don.check_button(btn) {
                Some(ReplayAction::Release(KeyPress::LeftDon))
            } else if c_config.right_don.check_button(btn) {
                Some(ReplayAction::Release(KeyPress::RightDon))
            } else if c_config.right_kat.check_button(btn) {
                Some(ReplayAction::Release(KeyPress::RightKat))
            } else {
                None
            }

        } else {
            trace!("Controller with no setup");

            // TODO: if this is slow, we should store controller configs separately
            // but i dont think this will be an issue, as its unlikely to happen in the first place,
            // and if there is lag, the user is likely to retry the man anyways
            trace!("Setting up new controller");
            let mut new_settings = self.taiko_settings.as_ref().clone();
            new_settings.controller_config.insert((*c.name).clone(), TaikoControllerConfig::defaults(c.name.clone()));

            // update the global settings
            {
                let mut settings = Settings::get_mut();
                settings.taiko_settings = new_settings.clone();
                // settings.save(&mut self.actions);
            }
            
            self.taiko_settings = Arc::new(new_settings);
            // rerun the handler now that the thing is setup
            self.controller_release(c, btn).await
        }
    }

}


#[cfg(not(feature="graphics"))]
impl GameModeInput for TaikoGame {}

#[async_trait]
impl GameModeProperties for TaikoGame {
    fn playmode(&self) -> Cow<'static, str> { Cow::Borrowed("taiko") }
    fn end_time(&self) -> f32 {self.end_time}

    fn get_info(&self) -> Arc<dyn GameModeInfo> {
        Arc::new(super::GameInfo)
    }
 
    fn get_possible_keys(&self) -> Vec<(KeyPress, &str)> {
        vec![
            (KeyPress::LeftKat, "LK"),
            (KeyPress::LeftDon, "LD"),
            (KeyPress::RightDon, "RD"),
            (KeyPress::RightKat, "RK"),
        ]
    }

    fn timing_bar_things(&self) -> Vec<(f32, Color)> {
        self.hit_windows
            .iter()
            .map(|(j, w)| (w.end, j.color))
            .collect()
    }

    async fn get_ui_elements(&self, _window_size: Vector2, ui_elements: &mut Vec<UIElement>) {
        let playmode = self.playmode();
        let get_name = |name| {
            format!("{playmode}_{name}")
        };

        let combo_bounds = Bounds::new(
            Vector2::ZERO,
            Vector2::new(self.playfield.hit_position.x - self.taiko_settings.note_radius, self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult)
        );
        
        // combo
        ui_elements.push(UIElement::new(
            &get_name("combo".to_owned()),
            Vector2::new(0.0, self.playfield.hit_position.y - self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult/2.0),
            ComboElement::new(combo_bounds).await
        ).await);

        // TODO: !!!!!
        // // Leaderboard
        // ui_elements.push(UIElement::new(
        //     &get_name("leaderboard".to_owned()),
        //     Vector2::with_y(self.playfield.hit_position.y + self.taiko_settings.note_radius * self.taiko_settings.big_note_multiplier + 50.0),
        //     LeaderboardElement::new().await
        // ).await);

        // don chan
        ui_elements.push(UIElement::new(
            &get_name("don_chan".to_owned()),
            self.playfield.pos,
            DonChan::new().await
        ).await);
    }

}
