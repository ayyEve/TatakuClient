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

use crate::prelude::*;
use super::prelude::*;

/// timing bar color
pub const BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0);
/// how wide is a timing bar
const BAR_WIDTH:f32 = 4.0;
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

const NOTE_DEPTH_RANGE:Range<f32> = 0.0..1000.0;

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
    
    hit_windows: Vec<(TaikoHitJudgments, Range<f32>)>,
    hit_cache: HashMap<TaikoHit, f32>,
    miss_window: f32,

    last_judgment: TaikoHitJudgments,
    current_mods: Arc<ModManager>,
    healthbar_swap_pending: bool,
}
impl TaikoGame {
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

        let hitsound = Hitsound::from_hitsamples(hitsound, samples, false, manager.timing_point_at(note_time, true));

        manager.play_note_sound(&hitsound).await;
    }

    async fn setup_hitwindows(&mut self) {
        let od = Self::get_od(&self.metadata, &self.current_mods);

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

        let diff_map = map_difficulty(od, 3.0, 5.0, 7.5);
        for note in self.other_notes.iter_mut() {
            if note.note_type() == NoteType::Spinner {
                let length = note.end_time(0.0) - note.time();
                let required_hits = ((length / 1000.0 * diff_map) * 1.65).max(1.0) as u16; 
                note.set_required_hits(required_hits);
            }
        }
    }


    fn add_hit_indicator(time: f32, mut hit_value: &TaikoHitJudgments, finisher_hit: bool, game_settings: &Arc<TaikoSettings>, judgment_helper: &JudgmentImageHelper, manager: &mut IngameManager) {
        let pos = game_settings.hit_position + Vector2::with_y(game_settings.judgement_indicator_offset);

        // if finisher, upgrade to geki or katu
        if finisher_hit {
            // remove the normal hit indicator, its being replaced with a finisher
            manager.judgement_indicators.pop();

            if let &TaikoHitJudgments::X100 = hit_value {
                hit_value = &TaikoHitJudgments::Katu;
            } else if let &TaikoHitJudgments::X300 = hit_value {
                hit_value = &TaikoHitJudgments::Geki;
            }
        }

        let color = hit_value.color();
        let mut image = if game_settings.use_skin_judgments { judgment_helper.get_from_scorehit(hit_value) } else { None };
        if let Some(image) = &mut image {
            image.pos = pos;
            image.depth = -2.0;

            let radius = game_settings.note_radius * game_settings.big_note_multiplier; // * game_settings.hit_area_radius_mult;
            image.scale = Vector2::ONE * (radius * 2.0) / TAIKO_JUDGEMENT_TEX_SIZE;
        }

        manager.add_judgement_indicator(BasicJudgementIndicator::new(
            pos, 
            time,
            -2.0,
            game_settings.note_radius * 0.5 * if finisher_hit { game_settings.big_note_multiplier } else { 1.0 },
            color,
            image
        ))
    }

    #[inline]
    pub fn get_depth(time: f32) -> f32 {
        NOTE_DEPTH_RANGE.start + (NOTE_DEPTH_RANGE.end - NOTE_DEPTH_RANGE.end / time)
    }
    #[inline]
    pub fn get_slider_depth(_time: f32) -> f32 {
        NOTE_DEPTH_RANGE.end
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
        Self::scale_by_mods(meta.od, 0.5, 1.4, mods).clamp(1.0, 10.0)
    }

}

#[async_trait]
impl GameMode for TaikoGame {
    async fn new(beatmap:&Beatmap, diff_calc_only:bool) -> TatakuResult<Self> {
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

        let playfield = Arc::new(TaikoPlayfield {
            pos: Vector2::ZERO,
            size: WindowSize::get().0
        });

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
            current_mods: ModManager::get(),
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
                        diff_calc_only,
                    ).await));
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
                                diff_calc_only,
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
                            diff_calc_only,
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
                        diff_calc_only,
                    ).await));
                }
            }
            Beatmap::Adofai(beatmap) => {

                // add notes
                for note in beatmap.notes.iter() {
                    let hit_type = HitType::Don;

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

        if !diff_calc_only {
            s.reload_skin().await;
        }

        s.setup_hitwindows().await;

        // // i wonder if not doing this has been causing issues
        // s.apply_mods(s.current_mods.clone()).await;


        Ok(s)
    }

    async fn handle_replay_frame(&mut self, frame:ReplayFrame, time:f32, manager:&mut IngameManager) {
        let ReplayFrame::Press(key) = frame else { return };

        // turn the keypress into a hit type
        let taiko_hit_type = match key {
            KeyPress::LeftKat  => TaikoHit::LeftKat,
            KeyPress::LeftDon  => TaikoHit::LeftDon,
            KeyPress::RightDon => TaikoHit::RightDon,
            KeyPress::RightKat => TaikoHit::RightKat,
            _ => TaikoHit::LeftKat
        };
        let is_left = taiko_hit_type == TaikoHit::LeftKat || taiko_hit_type == TaikoHit::LeftDon;
        
        if is_left { manager.add_stat(TaikoStatLeftPresses, 1.0) }
        else { manager.add_stat(TaikoStatRightPresses, 1.0) }

        // check fullalt
        if manager.current_mods.has_mod(FullAlt.name()) {
            if !self.counter.add_hit(taiko_hit_type) {
                return;
            }
        }

        let mut hit_type:HitType = key.into();
        let mut finisher_sound = false;
        // let mut sound = match hit_type {HitType::Don => "don", HitType::Kat => "kat"};

        let mut hit_time = time;
        let has_relax = manager.current_mods.has_mod(Relax.name());

        let mut did_hit = false;
        for queue in [&mut self.notes, &mut self.other_notes] {
            // if theres no more notes to hit, return after playing the sound
            if queue.done() { continue; }

            // check for finisher 2nd hit. 
            if !did_hit && self.last_judgment != TaikoHitJudgments::Miss {
                if let Some(last_note) = queue.previous_note() {
                    if last_note.check_finisher(hit_type, time, manager.current_mods.get_speed()) {
                        let j = match &self.last_judgment {
                            TaikoHitJudgments::X300 | TaikoHitJudgments::Geki => &TaikoHitJudgments::Geki,
                            TaikoHitJudgments::X100 | TaikoHitJudgments::Katu => &TaikoHitJudgments::Katu,
                            _ => return, // this shouldnt happen, last judgment will always be one of the above
                        };
                        
                        // add whatever the last judgment was as a finisher score
                        manager.add_judgment(j).await;
                        Self::add_hit_indicator(time, j, true, &self.taiko_settings, &self.judgement_helper, manager);

                        // draw drum
                        *self.hit_cache.get_mut(&taiko_hit_type).unwrap() = time;

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

                        if let Some(judge) = manager.check_judgment_condition(&self.hit_windows, time, note_time, cond, &TaikoHitJudgments::Miss).await {
                            // if note.finisher_sound() { sound = match hit_type { HitType::Don => "bigdon", HitType::Kat => "bigkat" } }
                            finisher_sound = note.finisher_sound();
                            if has_relax {
                                hit_type = note.hit_type();
                            }

                            if let TaikoHitJudgments::Miss = judge {
                                note.miss(time);
                            } else {
                                note.hit(time);
                            }

                            Self::add_hit_indicator(time, judge, false, &self.taiko_settings, &self.judgement_helper, manager);
                            
                            self.last_judgment = *judge;
                            queue.next();
                        }
                    },

                    // slider or spinner, special hit stuff
                    NoteType::Slider  if note.hit(time) => manager.add_judgment(&TaikoHitJudgments::SliderPoint).await,
                    NoteType::Spinner if note.hit(time) => manager.add_judgment(&TaikoHitJudgments::SpinnerPoint).await,
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
        *self.hit_cache.get_mut(&new_hit_type).unwrap() = time;

        // play sound
        self.play_sound(manager, hit_time, hit_type, finisher_sound).await;
    }


    async fn update(&mut self, manager:&mut IngameManager, time: f32) -> Vec<ReplayFrame> {

        // check healthbar swap
        if self.healthbar_swap_pending {
            self.healthbar_swap_pending = false;
            // println!("swapping health");

            // reset health helper to default
            manager.health = Default::default();

            // if we're using battery health
            if !self.current_mods.has_mod(NoBattery.name()) {
                let note_count = self.notes.iter().filter(|n|n.note_type() == NoteType::Note).count() as f32;

                const MAX_HEALTH:f32 = 200.0;
                const PASS_HEALTH:f32 = MAX_HEALTH / 2.0;

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


                let health = &mut manager.health;
                health.max_health = MAX_HEALTH;
                health.current_health = 0.0;
                health.initial_health = 0.0;
                health.check_fail_at_end = true;

                health.check_fail = Arc::new(move |s| s.current_health < PASS_HEALTH);
                health.do_health = Arc::new(move |s, j, _score| {
                    s.current_health += match j.as_str_internal() {
                        "x300" => health_per_300,
                        "x100" => health_per_100,
                        "xmiss" => health_per_miss,
                        _ => return
                    };

                    s.validate_health()
                });
            }
        }

        // do autoplay things
        if manager.current_mods.has_autoplay() {
            let mut pending_frames = Vec::new();
            let mut queues = vec![
                std::mem::take(&mut self.notes),
                std::mem::take(&mut self.other_notes),
            ];

            // get auto inputs
            self.auto_helper.update(time, &mut queues, &mut pending_frames);

            self.notes = queues.remove(0);
            self.other_notes = queues.remove(0);

            for frame in pending_frames.iter() {
                self.handle_replay_frame(*frame, time, manager).await;
            }

        }
        
        for queue in [&mut self.notes, &mut self.other_notes] {
            for note in queue.notes.iter_mut() {
                note.update(time).await;
            }

            if queue.done() {
                if !manager.completed && time > self.end_time {
                    manager.completed = true;
                }

                continue;
            }

            if let Some(do_miss) = queue.check_missed(time, self.miss_window) {
                if do_miss {
                    // queue.current_note().miss(time); // done in check_missed

                    let j = &TaikoHitJudgments::Miss;
                    manager.add_judgment(j).await;
                    Self::add_hit_indicator(time, j, false, &self.taiko_settings, &self.judgement_helper, manager);
                }

                queue.next()
            }
        }

        // TODO: might move tbs to a (time, speed) tuple
        for tb in self.timing_bars.iter_mut() { tb.update(time); }

        Vec::new()
    }
    async fn draw(&mut self, manager:&mut IngameManager, list: &mut RenderableCollection) {
        let time = manager.time();
        let lifetime_time = DRUM_LIFETIME_TIME * manager.game_speed();
        
        for (hit_type, hit_time) in self.hit_cache.iter() {
            if time - hit_time > lifetime_time { continue }
            let alpha = 1.0 - (time - hit_time) / (lifetime_time * 4.0);
            let depth = -1.0;
            match hit_type {
                TaikoHit::LeftKat => {
                    if let Some(kat) = &self.left_kat_image {
                        let mut img = kat.clone();
                        img.color.a = alpha;
                        list.push(img);
                    } else {
                        list.push(HalfCircle::new(
                            self.taiko_settings.kat_color.alpha(alpha),
                            self.taiko_settings.hit_position,
                            depth,
                            self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult,
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
                            self.taiko_settings.don_color.alpha(alpha),
                            self.taiko_settings.hit_position,
                            depth,
                            self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult,
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
                            self.taiko_settings.don_color.alpha(alpha),
                            self.taiko_settings.hit_position,
                            depth,
                            self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult,
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
                            self.taiko_settings.kat_color.alpha(alpha),
                            self.taiko_settings.hit_position,
                            depth,
                            self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult,
                            false
                        ));
                    }
                }
            }
        }

        // draw the playfield
        let window_size = WindowSize::get();
        list.push(self.taiko_settings.get_playfield(window_size.x, manager.current_timing_point().kiai));

        // draw the hit area
        list.push(Circle::new(
            Color::BLACK,
            1001.0,
            self.taiko_settings.hit_position,
            self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult,
            None
        ));

        // draw notes
        for note in self.notes.iter_mut().chain(self.other_notes.iter_mut()) { 
            note.draw(list).await 
        }

        // draw timing lines
        for tb in self.timing_bars.iter_mut() { tb.draw(list) }
    }

    async fn reset(&mut self, beatmap:&Beatmap) {
        for queue in [&mut self.notes, &mut self.other_notes] {
            queue.index = 0;
            
            for note in queue.iter_mut() {
                note.reset().await;

                // set note svs
                if self.current_mods.has_mod(NoSV.name()) {
                    note.set_sv(self.taiko_settings.sv_multiplier);
                } else {
                    let sv = (beatmap.slider_velocity_at(note.time()) / SV_FACTOR) * self.taiko_settings.sv_multiplier;
                    note.set_sv(sv);
                }
            }
        }

        
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
                if !self.current_mods.has_mod(NoSV.name()) {sv = (beatmap.slider_velocity_at(time) / SV_FACTOR) * self.taiko_settings.sv_multiplier}

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

        self.healthbar_swap_pending = true;
    }

    fn skip_intro(&mut self, manager: &mut IngameManager) {
        let x_needed = (self.playfield.pos.x + self.playfield.size.x) as f32;
        let mut time = self.end_time; //manager.time();
        
        for queue in [&self.notes, &self.other_notes] {
            if queue.index > 0 { return }

            for i in queue.notes.iter().rev() {
                let time_at = i.time_at(x_needed);
                time = time.min(time_at)
            }

        }

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

    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.playfield = Arc::new(TaikoPlayfield { pos: Vector2::ZERO, size: window_size.0 });
        
        // update notes
        for note in self.notes.iter_mut().chain(self.other_notes.iter_mut()) { 
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
        for note in self.notes.iter_mut().chain(self.other_notes.iter_mut()) { 
            note.playfield_changed(self.playfield.clone());
        }
        
        for tb in self.timing_bars.iter_mut() {
            tb.playfield_changed(self.playfield.clone());
        }
    }


    async fn force_update_settings(&mut self, settings: &Settings) {
        let old_sv_mult = self.taiko_settings.sv_multiplier;
        let sv_static = self.current_mods.has_mod(NoSV.name());
        
        let mut settings = settings.taiko_settings.clone();
        // calculate the hit area
        settings.init_settings().await;
        let settings = Arc::new(settings);
        self.taiko_settings = settings.clone();


        // update notes
        for n in self.notes.iter_mut().chain(self.other_notes.iter_mut()) {
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
        let scale = Vector2::ONE * (radius * 2.0) / TAIKO_HIT_INDICATOR_TEX_SIZE.x;

        for i in [ &mut self.left_don_image, &mut self.right_kat_image ] {
            if let Some(i) = i {
                i.scale = scale;
            }
        }
        
        for i in [ &mut self.left_kat_image, &mut self.right_don_image] {
            let scale = scale * Vector2::new(-1.0, 1.0);
            if let Some(i) = i {
                i.scale = scale;
            }
        }

    }
    
    async fn reload_skin(&mut self) {
        let radius = self.taiko_settings.note_radius * self.taiko_settings.hit_area_radius_mult;
        let scale = Vector2::ONE * (radius * 2.0) / TAIKO_HIT_INDICATOR_TEX_SIZE.x;

        if let Some(don) = &mut SkinManager::get_texture("taiko-drum-inner", true).await {
            don.depth = -1.0;
            don.origin.x = (don.tex_size() / don.base_scale).x;
            don.pos = self.taiko_settings.hit_position;

            let mut ldon = don.clone();
            ldon.scale = scale;
            self.left_don_image = Some(ldon);
            
            let mut rdon = don.clone();
            rdon.scale = scale * Vector2::new(-1.0, 1.0);
            rdon.origin.x = 0.0;
            self.right_don_image = Some(rdon);
        }
        if let Some(kat) = &mut SkinManager::get_texture("taiko-drum-outer", true).await {
            kat.depth = -1.0;
            kat.origin.x = 0.0;
            kat.pos = self.taiko_settings.hit_position;
            
            let mut lkat = kat.clone();
            lkat.scale = scale * Vector2::new(-1.0, 1.0);
            lkat.origin.x = lkat.tex_size().x;
            self.left_kat_image = Some(lkat);

            let mut rkat = kat.clone();
            rkat.scale = scale;
            self.right_kat_image = Some(rkat);
        }

        self.judgement_helper = JudgmentImageHelper::new(TaikoHitJudgments::Miss).await;

        for n in self.notes.iter_mut().chain(self.other_notes.iter_mut()) {
            n.reload_skin().await;
        }
    }

    
    async fn apply_mods(&mut self, mods: Arc<ModManager>) {
        let old_sv_mult = self.taiko_settings.sv_multiplier;
        let old_mods = self.current_mods.clone();

        let old_sv_static = old_mods.has_mod(NoSV.name());
        let current_sv_static = mods.has_mod(NoSV.name());
        self.current_mods = mods;

        // let old_no_finisher = old_mods.has_mod(NoFinisher.name());
        let new_no_finisher = self.current_mods.has_mod(NoFinisher.name());
        
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


        if old_mods.has_mod(NoBattery.name()) != self.current_mods.has_mod(NoBattery.name()) {
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
}

#[async_trait]
impl GameModeInput for TaikoGame {
    async fn key_down(&mut self, key:Key) -> Option<ReplayFrame> {
        // // dont accept key input when autoplay is enabled, or a replay is being watched
        // if manager.current_mods.has_autoplay() || manager.replaying {
        //     return;
        // }

        if key == self.taiko_settings.left_kat {
            Some(ReplayFrame::Press(KeyPress::LeftKat))
        } else if key == self.taiko_settings.left_don {
            Some(ReplayFrame::Press(KeyPress::LeftDon))
        } else if key == self.taiko_settings.right_don {
            Some(ReplayFrame::Press(KeyPress::RightDon))
        } else if key == self.taiko_settings.right_kat {
            Some(ReplayFrame::Press(KeyPress::RightKat))
        } else {
            None
        }
    }
    
    async fn key_up(&mut self, key:Key) -> Option<ReplayFrame> {

        if key == self.taiko_settings.left_kat {
            Some(ReplayFrame::Release(KeyPress::LeftKat))
        } else if key == self.taiko_settings.left_don {
            Some(ReplayFrame::Release(KeyPress::LeftDon))
        } else if key == self.taiko_settings.right_don {
            Some(ReplayFrame::Release(KeyPress::RightDon))
        } else if key == self.taiko_settings.right_kat {
            Some(ReplayFrame::Release(KeyPress::RightKat))
        } else {
            None
        }
    }


    async fn mouse_down(&mut self, btn:MouseButton) -> Option<ReplayFrame> {
        if self.taiko_settings.ignore_mouse_buttons { return None }
        
        match btn {
            MouseButton::Left  => Some(ReplayFrame::Press(KeyPress::LeftDon)),
            MouseButton::Right => Some(ReplayFrame::Press(KeyPress::LeftKat)),
            _ => None
        }
    }

    async fn mouse_up(&mut self, btn:MouseButton) -> Option<ReplayFrame> {
        if self.taiko_settings.ignore_mouse_buttons { return None }
        
        match btn {
            MouseButton::Left =>  Some(ReplayFrame::Release(KeyPress::LeftDon)),
            MouseButton::Right => Some(ReplayFrame::Release(KeyPress::LeftKat)),
            _ => None
        }
    }


    async fn controller_press(&mut self, c: &Box<dyn Controller>, btn: u8) -> Option<ReplayFrame> {

        if let Some(c_config) = self.taiko_settings.controller_config.get(&*c.get_name()) {

            // skip
            if Some(ControllerButton::Y) == c.map_button(btn) {
                Some(ReplayFrame::Press(KeyPress::SkipIntro))
            } else if c_config.left_kat.check_button(btn) {
                Some(ReplayFrame::Press(KeyPress::LeftKat))
            } else if c_config.left_don.check_button(btn) {
                Some(ReplayFrame::Press(KeyPress::LeftDon))
            } else if c_config.right_don.check_button(btn) {
                Some(ReplayFrame::Press(KeyPress::RightDon))
            } else if c_config.right_kat.check_button(btn) {
                Some(ReplayFrame::Press(KeyPress::RightKat))
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
            new_settings.controller_config.insert((*c.get_name()).clone(), TaikoControllerConfig::defaults(c.get_name()));

            // update the global settings
            {
                let mut settings = get_settings_mut!();
                settings.taiko_settings = new_settings.clone();
                settings.save().await;
            }
            
            self.taiko_settings = Arc::new(new_settings);
            // rerun the handler now that the thing is setup
            self.controller_press(c, btn).await
        }
    }

    async fn controller_release(&mut self, c: &Box<dyn Controller>, btn: u8) -> Option<ReplayFrame> {
        if let Some(c_config) = self.taiko_settings.controller_config.get(&*c.get_name()) {
            if c_config.left_kat.check_button(btn) {
                Some(ReplayFrame::Release(KeyPress::LeftKat))
            } else if c_config.left_don.check_button(btn) {
                Some(ReplayFrame::Release(KeyPress::LeftDon))
            } else if c_config.right_don.check_button(btn) {
                Some(ReplayFrame::Release(KeyPress::RightDon))
            } else if c_config.right_kat.check_button(btn) {
                Some(ReplayFrame::Release(KeyPress::RightKat))
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
            new_settings.controller_config.insert((*c.get_name()).clone(), TaikoControllerConfig::defaults(c.get_name()));

            // update the global settings
            {
                let mut settings = get_settings_mut!();
                settings.taiko_settings = new_settings.clone();
                settings.save().await;
            }
            
            self.taiko_settings = Arc::new(new_settings);
            // rerun the handler now that the thing is setup
            self.controller_release(c, btn).await
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
            Vector2::ZERO,
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
            Vector2::with_y(self.taiko_settings.hit_position.y + self.taiko_settings.note_radius * self.taiko_settings.big_note_multiplier + 50.0),
            LeaderboardElement::new().await
        ).await);

        // don chan
        ui_elements.push(UIElement::new(
            &get_name("don_chan".to_owned()),
            self.taiko_settings.get_playfield(0.0, false).pos,
            DonChan::new().await
        ).await);
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
        self.pos.x = self.settings.hit_position.x + self.x_at(time) - BAR_WIDTH / 2.0;
    }

    fn x_at(&self, time: f32) -> f32 {
        ((self.time - time) / SV_OVERRIDE) * self.speed * self.playfield.size.x as f32
    }
    fn draw(&mut self, list: &mut RenderableCollection){
        if self.pos.x + BAR_WIDTH < 0.0 || self.pos.x - BAR_WIDTH > 10000.0 {return}

        list.push(Rectangle::new(
            BAR_COLOR,
            1001.5,
            self.pos,
            self.size,
            None
        ));
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




pub struct TaikoPlayfield {
    pub pos: Vector2,
    pub size: Vector2,
}
