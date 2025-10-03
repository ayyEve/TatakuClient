/*
 * Taiko game mode
 * Author: ayyEve
 * 
 * NOTE! gekis and katus are for DISPLAY ONLY!!
 * they are not factored into acc!!
*/
use crate::prelude::*;

use common::{
    replays::*,
};

use tataku::{
    Color,
    Bounds,
    Vector2,
    Alignment,
};
use engine::{
    input,
    beatmaps::{
        osu::*,
        Beatmap,
        NoteType,
        BeatmapMeta,
        TimingPoint,
        map_difficulty,
    },
    gameplay,
    gameplay::{
        mods::*,
        Gamemode,
        Hitsound,
        judgments::*,
        GameplayEvent,
        TimingPointHelper,
        PlayfieldNonsense,
        GamemodeProperties,
        gameplay_manager::*,
    },
};
use input::GamepadButton;

#[cfg(feature="graphics")]
use engine::graphics;


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

#[derive(Default2)]
pub struct TaikoGame {
    // lists
    pub notes: TaikoNoteQueue,
    pub other_notes: TaikoNoteQueue,

    end_time: f32,
    auto_helper: TaikoAutoHelper,

    metadata: Arc<BeatmapMeta>,
    taiko_settings: Arc<TaikoSettings>,
    #[cfg(feature="graphics")] timing_bars: Vec<TimingBar>,
    #[cfg(feature="graphics")] left_kat_image: Option<graphics::Image>,
    #[cfg(feature="graphics")] left_don_image: Option<graphics::Image>,
    #[cfg(feature="graphics")] right_don_image: Option<graphics::Image>,
    #[cfg(feature="graphics")] right_kat_image: Option<graphics::Image>,
    #[cfg(feature="graphics")] playfield: Arc<TaikoPlayfield>,
    
    #[cfg(feature="graphics")] 
    #[default(JudgmentImageHelper::new(TaikoHitJudgments::variants().to_vec()))]
    judgement_helper: JudgmentImageHelper,

    counter: FullAltCounter,
    
    hit_windows: Vec<(HitJudgment, std::ops::Range<f32>)>,
    #[cfg(feature="graphics")] hit_cache: HashMap<TaikoHit, f32>,
    miss_window: f32,

    #[default(TaikoHitJudgments::Miss)]
    last_judgment: HitJudgment,
    current_mods: Arc<ModManager>,
    healthbar_swap_pending: bool,
}
impl TaikoGame {
    fn get_hitsound(
        note_time: f32, 
        hit_type: HitType, 
        finisher: bool,
        timing_points: &TimingPointHelper,
    ) -> Vec<Hitsound> {
        let hitsound = match (hit_type, finisher) {
            (HitType::Don, false) => 1, // normal is don
            (HitType::Don, true)  => 4, // finish is bigdon
            (HitType::Kat, false) => 8, // clap is kat
            (HitType::Kat, true)  => 2, // whistle is bigkat
        };

        Hitsound::from_hitsamples(
            hitsound, 
            HitSamples::default(), 
            false, 
            timing_points.timing_point_at(
                note_time, 
                true
            )
        )
    }


    fn setup_hitwindows(&mut self) {
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


    #[cfg(feature="graphics")]
    fn add_hit_indicator(
        mut hit_value: &HitJudgment, 
        finisher_hit: bool, 
        game_settings: &TaikoSettings, 
        playfield: &TaikoPlayfield,
        judgment_helper: &JudgmentImageHelper, 
        state: &mut GameplayUpdateShell,
    ) {
        let pos = playfield.hit_position 
            + Vector2::with_y(game_settings.judgement_indicator_offset);

        // if finisher, upgrade to geki or katu
        if finisher_hit {
            // remove the normal hit indicator, its being replaced with a finisher
            state.add_action(gameplay::Action::RemoveLastJudgment);

            if hit_value == &TaikoHitJudgments::X100 {
                hit_value = &TaikoHitJudgments::Katu;
            } else if hit_value == &TaikoHitJudgments::X300 {
                hit_value = &TaikoHitJudgments::Geki;
            }
        }

        let mut image = if game_settings.use_skin_judgments { 
            judgment_helper.get_from_scorehit(hit_value) 
        } else { 
            None 
        };

        if let Some(image) = &mut image {
            image.pos = pos;

            let radius = game_settings.note_radius * game_settings.big_note_multiplier;
            image.scale = Vector2::ONE * (radius * 2.0) / TAIKO_JUDGEMENT_TEX_SIZE;
        }

        state.add_indicator(BasicJudgementIndicator::new(
            pos, 
            state.time,
            game_settings.note_radius 
                * 0.5 
                * if finisher_hit { game_settings.big_note_multiplier } else { 1.0 },
            hit_value.color,
            image
        ));
    }

    #[inline]
    pub fn scale_by_mods<V:std::ops::Mul<Output=V>>(
        val: V, 
        ez_scale: V, 
        hr_scale: V, 
        mods: &ModManager
    ) -> V {
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
        Self::scale_by_mods(meta.od, 0.5, 1.4, mods)
            .clamp(1.0, 10.0)
    }
    

    pub fn get_taiko_playfield(
        settings: &TaikoSettings, 
        bounds: Bounds, 
        full_window: bool
    ) -> TaikoPlayfield {
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
            bounds.size - Vector2::new(
                bounds.size.x, 
                bounds.size.y / settings.hit_position_relative_height_div
            ) 
        } else { Vector2::ZERO };

        let hit_position = bounds.pos 
            + base 
            + Vector2::new(x_offset + half_note_width, y_offset);

        TaikoPlayfield {
            bounds,
            height,
            hit_position,
            // full_window
        }
    } 

    #[allow(clippy::borrowed_box, reason = "matches sort_by signature")]
    fn sort(
        a: &Box<dyn TaikoHitObject>,
        b: &Box<dyn TaikoHitObject>,
    ) -> std::cmp::Ordering {
        a.time()
            .partial_cmp(&b.time())
            .unwrap()
    }

    fn map_gamepad_button(
        &self, 
        config: &TaikoControllerConfig,
        btn: GamepadButton,
    ) -> Option<KeyPress> {
        if self.taiko_settings.gamepad_left_kat == Some(btn) {
            Some(KeyPress::LeftKat)
        } else if self.taiko_settings.gamepad_left_don == Some(btn) {
            Some(KeyPress::LeftDon)
        } else if self.taiko_settings.gamepad_right_don == Some(btn) {
            Some(KeyPress::RightDon)
        } else if self.taiko_settings.gamepad_right_kat == Some(btn) {
            Some(KeyPress::RightKat)
        }
        
        else if config.left_kat.check_button(btn) {
            Some(KeyPress::LeftKat)
        } else if config.left_don.check_button(btn) {
            Some(KeyPress::LeftDon)
        } else if config.right_don.check_button(btn) {
            Some(KeyPress::RightDon)
        } else if config.right_kat.check_button(btn) {
            Some(KeyPress::RightKat)
        } 

        // skip
        else if GamepadButton::North == btn {
            Some(KeyPress::SkipIntro)
        }
        else {
            None
        }
    }
}

impl Gamemode for TaikoGame {
    fn new(
        beatmap: &Beatmap, 
        _diff_calc_only: bool, 
        settings: &engine::Settings
    ) -> tataku::Result<Self> {
        let metadata = beatmap.get_beatmap_meta();

        let settings = settings
            .gamemode_settings::<TaikoSettings>(GAME_INFO)
            .unwrap_or_default();
        let settings = Arc::new(settings);

        #[cfg(feature="graphics")] 
        let playfield = Arc::new(Self::get_taiko_playfield(
            &settings, 
            Bounds::new(Vector2::ZERO, Vector2::new(1920.0, 1080.0)), 
            true
        ));
        
        let timing_points = TimingPointHelper::new(
            beatmap.get_timing_points(), 
            beatmap.slider_velocity()
        );

        let mut s = Self {
            taiko_settings: settings.clone(),
            #[cfg(feature="graphics")] playfield: playfield.clone(),
            metadata,
            
            #[cfg(feature="graphics")] 
            hit_cache: TaikoHit::ALL
                .iter()
                .map(|i| (*i, -999.9))
                .collect(),

            ..Self::default()
        };

        match beatmap {
            Beatmap::Osu(beatmap) => {
                // add notes
                for note in beatmap.notes.iter() {
                    let hit_type = HitType::new((note.hitsound & (2 | 8)) > 0);
                    let finisher = (note.hitsound & 4) > 0;

                    s.notes.push(Box::new(TaikoNote::new(
                        note.time,
                        hit_type,
                        finisher,
                        settings.clone(),
                        #[cfg(feature="graphics")] playfield.clone(),
                    )));
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
                    let skip_period = (bl / beatmap.slider_tick_rate)
                        .min((end_time - time) / slides as f32);

                    if skip_period > 0.0 
                        && &*beatmap.metadata.mode != "taiko" 
                        && l / v * 1000.0 < 2.0 * bl 
                    {
                        let mut i = 0;
                        let mut j = time;

                        // load sounds
                        // let sound_list_raw = if let Some(list) = split.next() {list.split("|")} else {"".split("")};

                        // when loading, if unified just have it as sound_types with 1 index
                        let mut sound_types:Vec<(HitType, bool)> = Vec::new();

                        for hitsound in slider.edge_sounds.iter() {
                            let hit_type = HitType::new(
                                (hitsound & (2 | 8)) > 0
                            );
                            let finisher = (hitsound & 4) > 0;
                            sound_types.push((hit_type, finisher));
                        }
                        
                        let unified_sound_addition = sound_types.is_empty();
                        if unified_sound_addition {
                            sound_types.push((HitType::Don, false));
                        }

                        loop {
                            let sound_type = sound_types[i];

                            s.notes.push(Box::new(TaikoNote::new(
                                j,
                                sound_type.0,
                                sound_type.1,
                                settings.clone(),
                                #[cfg(feature="graphics")] playfield.clone(),
                            )));

                            if !unified_sound_addition { 
                                i = (i + 1) % sound_types.len();
                            }

                            j += skip_period;
                            if j >= end_time + skip_period / 8.0 { break }
                        }
                    } else {
                        s.other_notes.push(Box::new(TaikoDrumroll::new(
                            time, 
                            end_time, 
                            finisher, 
                            settings.clone(),
                            #[cfg(feature="graphics")] playfield.clone(),
                        )));
                    }
                }
                for spinner in beatmap.spinners.iter() {
                    s.other_notes.push(Box::new(TaikoSpinner::new(
                        spinner.time,
                        spinner.end_time, 
                        0, 
                        settings.clone(),
                        #[cfg(feature="graphics")] playfield.clone(),
                    )));
                }
            }

            Beatmap::Tja(beatmap) => {
                for note in beatmap.circles.iter() {
                    s.notes.push(Box::new(TaikoNote::new(
                        note.time,
                        HitType::new(!note.is_don),
                        note.is_big,
                        settings.clone(),
                        #[cfg(feature="graphics")] playfield.clone(),
                    )));
                }

                for drumroll in beatmap.drumrolls.iter() {
                    s.other_notes.push(Box::new(TaikoDrumroll::new(
                        drumroll.time, 
                        drumroll.end_time, 
                        drumroll.is_big, 
                        settings.clone(),
                        #[cfg(feature="graphics")] playfield.clone(),
                    )));
                }

                for balloon in beatmap.balloons.iter() {
                    s.other_notes.push(Box::new(TaikoSpinner::new(
                        balloon.time,
                        balloon.end_time, 
                        balloon.hits_required as u16, 
                        settings.clone(),
                        #[cfg(feature="graphics")] playfield.clone(),
                    )));
                }
            }
            _ => return Err(errors::beatmap::BeatmapError::UnsupportedMode.into()),
        };

        if s.notes.is_empty() && s.other_notes.is_empty() { 
            return Err(tataku::Error::Beatmap(errors::beatmap::BeatmapError::NoNotes)) 
        }

        s.notes.sort_by(Self::sort);
        s.other_notes.sort_by(Self::sort);

        // theres probably a better way to do this lol
        if let Some(last) = s.notes.last() {
            s.end_time = s.end_time.max(last.end_time(0.0));
        }
        if let Some(last) = s.other_notes.last() {
            s.end_time = s.end_time.max(last.end_time(0.0));
        }
        s.end_time += 1000.0;

        s.setup_hitwindows();

        Ok(s)
    }

    fn handle_replay_frame(
        &mut self, 
        frame: ReplayFrame, 
        shell: &mut GameplayUpdateShell,
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
        let is_left = taiko_hit_type == TaikoHit::LeftKat 
            || taiko_hit_type == TaikoHit::LeftDon;
        
        if is_left { 
            shell.add_stat(TaikoStatLeftPresses, 1.0);
        } else { 
            shell.add_stat(TaikoStatRightPresses, 1.0);
        }

        // check fullalt
        if shell.mods.has_mod(FullAlt) && !self.counter.add_hit(taiko_hit_type) {
            return;
        }

        let mut hit_type: HitType = key.into();
        #[cfg(feature="gameplay")] 
        let mut finisher_sound = false;
        // let mut sound = match hit_type {HitType::Don => "don", HitType::Kat => "kat"};

        let mut hit_time = frame.time;
        let has_relax = shell.mods.has_mod(Relax);

        let mut did_hit = false;
        for queue in [&mut self.notes, &mut self.other_notes] {
            // if theres no more notes to hit, return after playing the sound
            if queue.done() { continue; }

            // check for finisher 2nd hit. 
            if !did_hit && self.last_judgment != TaikoHitJudgments::Miss
            && let Some(last_note) = queue.previous_note()
            && last_note.check_finisher(hit_type, frame.time, shell.game_speed) {

                // i cant match on these contants bc i dont use the derive macro :c
                // let j = match &self.last_judgment {
                //     &TaikoHitJudgments::X300 | &TaikoHitJudgments::Geki => &TaikoHitJudgments::Geki,
                //     &TaikoHitJudgments::X100 | &TaikoHitJudgments::Katu => &TaikoHitJudgments::Katu,
                //     _ => return, // this shouldnt happen, last judgment will always be one of the above
                // };
                let j = if [
                    &TaikoHitJudgments::X300, 
                    &TaikoHitJudgments::Geki
                ].contains(&&self.last_judgment) {
                    &TaikoHitJudgments::Geki
                } else if [
                    &TaikoHitJudgments::X100, 
                    &TaikoHitJudgments::Katu
                ].contains(&&self.last_judgment) {
                    &TaikoHitJudgments::Katu
                } else {
                    return
                };

                // add whatever the last judgment was as a finisher score
                shell.add_judgment(*j);

                #[cfg(feature="graphics")] {
                    Self::add_hit_indicator(
                        j, 
                        true, 
                        &self.taiko_settings, 
                        &self.playfield, 
                        &self.judgement_helper, 
                        shell
                    );
                    
                    // draw drum
                    *self.hit_cache.get_mut(&taiko_hit_type).unwrap() = shell.time;
                }

                return; // return and not continue because we dont want the 2nd finisher press to count towards anything
            }

            // check note hit
            if let Some(note) = queue.current_note() {
                let note_time = note.time();
                match note.note_type() {
                    NoteType::Note => {
                        let hit_maybe = shell.check_judgment_condition(
                            &self.hit_windows, 
                            frame.time, 
                            note_time, 
                            || has_relax || note.hit_type() == hit_type, 
                            &TaikoHitJudgments::Miss
                        );

                        if let Some(judge) = hit_maybe {
                            // if note.finisher_sound() { sound = match hit_type { HitType::Don => "bigdon", HitType::Kat => "bigkat" } }
                            
                            #[cfg(feature="gameplay")] {
                                finisher_sound = note.finisher_sound();
                            }

                            if has_relax {
                                hit_type = note.hit_type();
                            }

                            if judge == &TaikoHitJudgments::Miss {
                                note.miss(shell.time);
                            } else {
                                note.hit(shell.time, hit_type);
                            }

                            #[cfg(feature="graphics")]
                            Self::add_hit_indicator(
                                judge, 
                                false, 
                                &self.taiko_settings, 
                                &self.playfield, 
                                &self.judgement_helper, 
                                shell
                            );
                            
                            self.last_judgment = *judge;
                            queue.next();
                        }
                    }

                    // slider or spinner, special hit stuff
                    NoteType::Slider if note.hit(shell.time, hit_type) 
                        => shell.add_judgment(TaikoHitJudgments::SliderPoint),
                    NoteType::Spinner if note.hit(shell.time, hit_type) 
                        => shell.add_judgment(TaikoHitJudgments::SpinnerPoint),
                    _ => {}
                }

                // if was hit, the sound already played
                if !did_hit {
                    hit_time = note_time;
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
        #[cfg(feature="graphics")] {
            *self.hit_cache.get_mut(&new_hit_type).unwrap() = frame.time;
        }

        // play sound
        #[cfg(feature="gameplay")] 
        shell.play_hitsounds(
            &Self::get_hitsound(
                hit_time,
                hit_type,
                finisher_sound,
                shell.timing_points,
            ), 
            false
        );
    }

    fn handle_gameplay_event(&mut self, event: GameplayEvent) {
        match event {
            GameplayEvent::ApplyMods(mods) => {
                let old_sv_mult = self.taiko_settings.sv_multiplier;
                let old_mods = self.current_mods.clone();

                let old_sv_static = old_mods.has_mod(NoSV);
                let current_sv_static = mods.has_mod(NoSV);
                self.current_mods = mods;

                // let old_no_finisher = old_mods.has_mod(NoFinisher);
                let new_no_finisher = self.current_mods.has_mod(NoFinisher);
                
                // update bars
                #[cfg(feature="graphics")] 
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
                for note in self
                    .notes.iter_mut()
                    .chain(self.other_notes.iter_mut())
                {
                    
                    // set note svs
                    #[cfg(feature="graphics")] 
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

            #[cfg(feature="graphics")] 
            GameplayEvent::SetBounds { 
                bounds, 
                full_window 
            } => {
                self.playfield = Arc::new(Self::get_taiko_playfield(
                    &self.taiko_settings, 
                    bounds, 
                    full_window
                ));

                // update notes
                for note in self.notes.iter_mut()
                    .chain(self.other_notes.iter_mut())
                { 
                    note.playfield_changed(self.playfield.clone());
                }

                // update timing bars
                #[cfg(feature="graphics")] 
                for tb in self.timing_bars.iter_mut() {
                    tb.playfield_changed(self.playfield.clone());
                }

                // update hit indicator sprite positions
                #[cfg(feature="graphics")]
                for i in [ 
                    &mut self.left_kat_image, 
                    &mut self.left_don_image, 
                    &mut self.right_don_image, 
                    &mut self.right_kat_image 
                ] {
                    let Some(i) = i else { continue };
                    i.pos = self.playfield.hit_position;
                }
            }
            
            #[cfg(feature="graphics")] 
            GameplayEvent::BeatHappened { pulse_length } => {
                self.notes
                    .iter_mut()
                    .chain(self.other_notes.iter_mut())
                    .for_each(|n| n.beat_happened(pulse_length));
            }
            #[cfg(feature="graphics")] 
            GameplayEvent::KiaiChanged { enabled } => {
                self.notes
                    .iter_mut()
                    .chain(self.other_notes.iter_mut())
                    .for_each(|n| n.kiai_changed(enabled));
            }
            
            _ => {}
        }
    }

    fn update(&mut self, shell: &mut GameplayUpdateShell) {
        // check healthbar swap
        if self.healthbar_swap_pending {
            self.healthbar_swap_pending = false;

            // reset health helper to default
            shell.add_action(gameplay::Action::ResetHealth); // manager.health = Default::default();

            // if we're using battery health
            if !self.current_mods.has_mod(NoBattery) {
                let note_count = self.notes.iter()
                    .filter(|n| n.note_type() == NoteType::Note)
                    .count() as f32;

                const MAX_HEALTH:f32 = 200.0;

                // this is essentially stolen from peppy's 2016 osu code
                let hp = map_difficulty(
                    self.metadata.hp, 
                    0.5, 
                    0.75, 
                    0.98
                );
                let normal_health = MAX_HEALTH / (0.06 * 6.0 * note_count * hp);
                
                // random fudge because osu makes no sense
                const FACTOR: f32 = 15.0;
                let normal_health = normal_health / FACTOR;

                let health_per_300 = normal_health * 6.0;
                let health_per_100 = normal_health * map_difficulty(
                    self.metadata.hp, 
                    6.0, 
                    2.2, 
                    2.2
                );
                let health_per_miss = map_difficulty(
                    self.metadata.hp, 
                    -6.0, 
                    -25.0, 
                    -40.0
                ) / FACTOR;

                shell.add_action(gameplay::Action::replace_health(TaikoBatteryHealthManager::new(
                    health_per_300,
                    health_per_100,
                    health_per_miss
                )));
            }
        }

        // do autoplay things
        if shell.mods.has_autoplay() {
            let mut pending_frames = Vec::new();
            let mut queues = vec![
                std::mem::take(&mut self.notes),
                std::mem::take(&mut self.other_notes),
            ];

            // get auto inputs
            self.auto_helper.update(shell.time, &mut queues, &mut pending_frames);

            self.notes = queues.remove(0);
            self.other_notes = queues.remove(0);

            for frame in pending_frames.into_iter() {
                self.handle_replay_frame(
                    ReplayFrame::new(shell.time, frame), 
                    shell
                );
            }

        }
        
        for queue in [&mut self.notes, &mut self.other_notes] {
            for note in queue.notes.iter_mut() {
                note.update(shell.time);
            }

            if queue.done() {
                if !shell.complete() && shell.time > self.end_time {
                    shell.add_action(gameplay::Action::MapComplete);
                    // manager.completed = true;
                }

                continue;
            }

            if let Some(do_miss) = queue.check_missed(shell.time, self.miss_window) {
                if do_miss {
                    // queue.current_note().miss(time); // done in check_missed

                    let j = TaikoHitJudgments::Miss;
                    shell.add_judgment(j);

                    #[cfg(feature="graphics")]
                    Self::add_hit_indicator(
                        &j, 
                        false, 
                        &self.taiko_settings, 
                        &self.playfield, 
                        &self.judgement_helper, 
                        shell
                    );
                }

                queue.next();
            }
        }

        #[cfg(feature="graphics")] 
        for tb in self.timing_bars.iter_mut() { 
            tb.update(shell.time); 
        }
    }
    
    #[cfg(feature = "graphics")]
    fn draw(
        &mut self, 
        shell: GameplayDrawShell, 
        list: &mut graphics::RenderableCollection
    ) {

        // draw the playfield
        list.push(self.playfield.get_rectangle(shell.current_timing_point.kiai));
        
        // draw the hit area
        list.push(graphics::Circle::new(
            self.playfield.hit_position,
            self.taiko_settings.note_radius 
                * self.taiko_settings.hit_area_radius_mult,
            Color::BLACK,
        ));

        // draw timing lines
        for tb in self.timing_bars.iter_mut() { tb.draw(list) }

        // draw notes
        // earlier notes are drawn on top of later notes
        let mut note_list = self
            .notes
            .iter_mut()
            .chain(self.other_notes.iter_mut())
            .collect::<Vec<_>>();

        note_list.sort_by(|a, b| {
            b.time()
                .partial_cmp(&a.time())
                .unwrap_or(core::cmp::Ordering::Equal)
        });

        for note in note_list { 
            note.draw(shell.time, list);
        }

        // draw hit indicators
        let lifetime_time = DRUM_LIFETIME_TIME * shell.mods.get_speed();
        for (hit_type, hit_time) in self.hit_cache.iter() {
            if shell.time - hit_time > lifetime_time { continue }
            let alpha = 1.0 - (shell.time - hit_time) / (lifetime_time * 4.0);
            let alpha = (alpha.clamp(0.0, 1.0) * 255.0) as u8;
            match hit_type {
                TaikoHit::LeftKat => {
                    if let Some(kat) = &self.left_kat_image {
                        let mut img = kat.clone();
                        img.color.a = alpha;
                        list.push(img);
                    } else {
                        list.push(graphics::HalfCircle::new(
                            self.playfield.hit_position,
                            self.taiko_settings.note_radius 
                                * self.taiko_settings.hit_area_radius_mult,
                            self.taiko_settings.kat_color.alpha8(alpha),
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
                        list.push(graphics::HalfCircle::new(
                            self.playfield.hit_position,
                            self.taiko_settings.note_radius 
                                * self.taiko_settings.hit_area_radius_mult,
                            self.taiko_settings.don_color.alpha8(alpha),
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
                        list.push(graphics::HalfCircle::new(
                            self.playfield.hit_position,
                            self.taiko_settings.note_radius 
                                * self.taiko_settings.hit_area_radius_mult,
                            self.taiko_settings.don_color.alpha8(alpha),
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
                        list.push(graphics::HalfCircle::new(
                            self.playfield.hit_position,
                            self.taiko_settings.note_radius 
                                * self.taiko_settings.hit_area_radius_mult,
                            self.taiko_settings.kat_color.alpha8(alpha),
                            false
                        ));
                    }
                }
            }
        }

        if shell.mods.has_mod(Flashlight) {
            let radius = match shell.score.combo {
                0..=99 => 125.0,
                100..=199 => 100.0,
                _ => 75.0
            } * self.taiko_settings.sv_multiplier * 2.0;
            let fade_radius = radius / 5.0;

            list.push(graphics::FlashlightDrawable::new(
                self.playfield.hit_position,
                radius - fade_radius,
                fade_radius,
                self.playfield.bounds,
                Color::BLACK
            ));
        }
    }


    fn all_notes(&self) -> Vec<&dyn engine::gameplay::HitObject> {
        self.notes.iter()
            .chain(self.other_notes.iter())
            .map(|i| &**i as &dyn engine::gameplay::HitObject)
            .collect::<Vec<&dyn engine::gameplay::HitObject>>()
    }

    fn reset(&mut self, beatmap: &Beatmap) {
        #[cfg(feature="graphics")] 
        let timing_points = TimingPointHelper::new(
            beatmap.get_timing_points(), 
            beatmap.slider_velocity()
        );

        for queue in [&mut self.notes, &mut self.other_notes] {
            queue.index = 0;
            
            for note in queue.iter_mut() {
                note.reset();

                // set note svs
                #[cfg(feature="graphics")] 
                if self.current_mods.has_mod(NoSV) {
                    note.set_sv(self.taiko_settings.sv_multiplier);
                } else {
                    let sv = timing_points
                        .slider_velocity_at(note.time()) 
                        / SV_FACTOR;

                    note.set_sv(sv* self.taiko_settings.sv_multiplier);
                }
            }
        }

        self.last_judgment = TaikoHitJudgments::Miss;
        self.counter = FullAltCounter::default();

        // setup timing bars
        #[cfg(feature="graphics")] {
            if self.timing_bars.is_empty() {
                // load timing bars
                let parent_tps = timing_points
                    .iter()
                    .filter(|t| !t.is_inherited())
                    .collect::<Vec<&TimingPoint>>();
                
                let mut sv = self.taiko_settings.sv_multiplier;
                let mut time = parent_tps[0].time;
                let mut tp_index = 0;
                let step = timing_points.beat_length_at(time, false);
                time %= step; // get the earliest bar line possible
    
                loop {
                    if !self.current_mods.has_mod(NoSV) {
                        sv = (timing_points.slider_velocity_at(time) / SV_FACTOR) 
                            * self.taiko_settings.sv_multiplier;
                    }
    
                    // if theres a bpm change, adjust the current time to that of the bpm change
                    let next_bar_time = timing_points
                        .beat_length_at(time, false)
                        * BAR_SPACING; // bar spacing is actually the timing point measure
    
                    // edge case for aspire maps
                    if next_bar_time.is_nan() || next_bar_time == 0.0 { break; }
    
                    // add timing bar at current time
                    self.timing_bars.push(TimingBar::new(
                        time, 
                        sv, 
                        self.playfield.clone()
                    ));
    
                    if tp_index < parent_tps.len() 
                        && parent_tps[tp_index].time <= time + next_bar_time 
                    {
                        time = parent_tps[tp_index].time;
                        tp_index += 1;
                        continue;
                    }
    
                    // why isnt this accounting for bpm changes? because the bpm change doesnt always happen inline with the bar idiot
                    time += next_bar_time;
                    if time >= self.end_time || time.is_nan() { break }
                } 
    
            }
            
            // reset hitcache times
            for t in self.hit_cache.values_mut() {
                *t = -999.9;
            }
        }

        self.healthbar_swap_pending = true;
    }

    #[cfg(feature="graphics")] 
    fn skip_intro(&mut self, game_time: f32) -> Option<f32> {
        let x_needed = self.playfield.pos.x + self.playfield.size.x;
        let mut time = self.end_time; //manager.time();
        
        for queue in [&self.notes, &self.other_notes] {
            if queue.index > 0 { return None }

            for i in queue.notes.iter().rev() {
                let time_at = i.time_at(x_needed);
                time = time.min(time_at);
            }
        }

        if game_time >= time { return None }
        
        if time < 0.0 { return None }
        Some(time)
    }

    fn force_update_settings(&mut self, settings: &engine::Settings) {
        let settings = settings
            .gamemode_settings::<TaikoSettings>(GAME_INFO)
            .unwrap_or_default();

        if settings == *self.taiko_settings { return }
        let settings = Arc::new(settings);
        #[cfg(feature="graphics")] 
        let playfield = {
            let playfield = Arc::new(Self::get_taiko_playfield(
                &settings, 
                self.playfield.bounds, 
                false
            ));

            self.playfield = playfield.clone();

            playfield
        };

        let old_sv_mult = self.taiko_settings.sv_multiplier;
        let sv_static = self.current_mods.has_mod(NoSV);

        self.taiko_settings = settings.clone();


        // update notes
        for n in self
            .notes.iter_mut()
            .chain(self.other_notes.iter_mut())
        {
            n.set_settings(settings.clone());

            #[cfg(feature="graphics")] {
                n.playfield_changed(self.playfield.clone());
                
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
        }


        #[cfg(feature="graphics")] {
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
    
            for i in [ 
                &mut self.left_don_image, 
                &mut self.right_kat_image 
            ] {
                let Some(i) = i else { continue };
                i.scale = scale;
                i.pos = self.playfield.hit_position;
            }
    
            for i in [ 
                &mut self.left_kat_image, 
                &mut self.right_don_image
            ] {
                let Some(i) = i else { continue };
                i.scale = scale * Vector2::new(-1.0, 1.0);
                i.pos = self.playfield.hit_position;
            }
        }

    }
    
    #[cfg(feature="graphics")]
    fn reload_skin(
        &mut self, 
        beatmap_path: &str, 
        skin_manager: &mut dyn graphics::SkinProvider
    ) -> graphics::TextureSource {
        use graphics::{
            SkinUsage,
            TextureSource,
        };
        let source = TextureSource::Beatmap(beatmap_path.to_owned()); // TODO: yeah

        let radius = self.taiko_settings.note_radius 
            * self.taiko_settings.hit_area_radius_mult;
        let scale = Vector2::ONE 
            * (radius * 2.0) 
            / TAIKO_HIT_INDICATOR_TEX_SIZE.x;

        if let Some(mut don) = skin_manager.get_texture(
            "taiko-drum-inner", 
            &source, 
            SkinUsage::Gamemode, 
            true
        ) {
            don.origin.x = (don.tex_size() / don.base_scale).x;
            don.pos = self.playfield.hit_position;
            don.scale = scale;
            self.left_don_image = Some(don.clone());
            
            let mut rdon = don;
            rdon.scale *= Vector2::new(-1.0, 1.0);
            self.right_don_image = Some(rdon);
        }
        if let Some(mut kat) = skin_manager.get_texture(
            "taiko-drum-outer", 
            &source, 
            SkinUsage::Gamemode, 
            true
        ) {
            kat.origin.x = 0.0;
            kat.pos = self.playfield.hit_position;
            kat.scale = scale;
            self.right_kat_image = Some(kat.clone());

            let mut lkat = kat; 
            lkat.scale *= Vector2::new(-1.0, 1.0);
            self.left_kat_image = Some(lkat);
        }

        self.judgement_helper = JudgmentImageHelper::new(
            TaikoHitJudgments::variants().to_vec()
        );

        for n in self
            .notes.iter_mut()
            .chain(self.other_notes.iter_mut())
        {
            n.reload_skin(&source, skin_manager);
        }

        source
    }

    
    fn time_jump(&mut self, new_time: f32, _state: &mut GameplayUpdateShell) {
        let mut latest_time = 0f32;
        #[cfg(feature="graphics")] 
        for i in self.hit_cache.values() { 
            latest_time = latest_time.max(*i);
        }
        // info!("{new_time} < {latest_time}");

        if new_time < latest_time {
            for queue in [&mut self.notes, &mut self.other_notes] {
                let mut index = 0;
                for (i, note) in queue
                    .iter_mut()
                    .enumerate() 
                {
                    note.reset();
                    if note.time() <= new_time {
                        index = i;
                    }
                }
                queue.index = index;
            }
            
            // reset hitcache times
            #[cfg(feature="graphics")] 
            for t in self.hit_cache.values_mut() {
                *t = -999.9;
            }
        }
    }

    #[cfg(feature = "graphics")]
    fn build_widgets(&self, loader: &mut dyn engine::gameplay::widgets::UiElementLoader) {
        use engine::gameplay::widgets::*;

        // combo
        loader.change_default_layout(
            "combo",
            GameplayWidgetLayout::new_default(
                GameplayWidgetAnchor::Playfield {
                    saved_size: None,
                    relative: GameplayWidgetAlign::Inside,
                },
                Alignment::CENTER_LEFT,
                None,
                None,
            ),
        );

        // Leaderboard
        loader.change_default_layout(
            "leaderboard",
            GameplayWidgetLayout::new_default(
                GameplayWidgetAnchor::Playfield {
                    saved_size: None,
                    relative: GameplayWidgetAlign::Below
                },
                Alignment::BOTTOM_LEFT,
                None,
                None,
            ),
        );

        // don chan
        loader.load(
            "don_chan",
        );
    }

    #[cfg(feature="graphics")] 
    fn get_playfield(&self) -> PlayfieldNonsense {
        PlayfieldNonsense::new_simple(self.playfield.get_playfield_bounds())
    }
    fn properties(&self, timing_points: &TimingPointHelper) -> GamemodeProperties {

        // FIXME: please god optimize this
        let mut sound_list = HashMap::new();
        for time in 
            self.notes.iter().map(|n| n.time())
            .chain(self.other_notes.iter().map(|n| n.time()))
            .chain(timing_points.iter().map(|t| t.time))
        {
            for (hit, finisher) in [
                (HitType::Don, false),
                (HitType::Kat, false),
                (HitType::Don, true),
                (HitType::Kat, true),
            ] {
                let hitsound = Self::get_hitsound(
                    time, 
                    hit, 
                    finisher, 
                    timing_points
                );

                for i in hitsound {
                    sound_list.insert(i.get_id(), i.load_data(Some("taiko-")));
                    // sound_list.insert(i.get_id(), i.load_data(None::<String>));
                }
            }
        }


        // for hitsound in [0, 1, 2, 4, 8] {
        //     let hitsound = Hitsound::from_hitsamples(
        //         hitsound, 
        //         HitSamples::default(), 
        //         false, 
        //         &TimingPoint::default(),
        //     );
        //     for i in hitsound {
        //         sound_list.insert(i.get_id(), i.load_data(Some("taiko-")));
        //         sound_list.insert(i.get_id(), i.load_data(None::<String>));
        //     }
        // }

        GamemodeProperties { 
            info: &crate::GAME_INFO, 
            keys: vec![
                (KeyPress::LeftKat, "LK"),
                (KeyPress::LeftDon, "LD"),
                (KeyPress::RightDon, "RD"),
                (KeyPress::RightKat, "RK"),
            ], 
            end_time: self.end_time, 
            show_cursor: false, 
            audio_prefix: "taiko".to_owned(),
            timing_bar_things: self.hit_windows.iter()
                .map(|(j, w)| (w.end, j.color))
                .collect(), 
            sound_list: sound_list.into_iter().collect()
        }
    }


    #[cfg(feature="gameplay")] 
    fn handle_input(&mut self, input: input::InputEvent) -> Option<ReplayAction> {
        use input::{
            InputType,
            InputEvent,
            MouseButton,
        };


        match input.event {
            InputType::KeyPress(key) => {
                let key = key.as_key()?;

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

            InputType::KeyRelease(key) => {
                let key = key.as_key()?;
                
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

            InputType::MousePress(btn) => {
                if self.taiko_settings.ignore_mouse_buttons { return None }
                
                match btn {
                    MouseButton::Left => Some(ReplayAction::Press(KeyPress::LeftDon)),
                    MouseButton::Right => Some(ReplayAction::Press(KeyPress::LeftKat)),
                    _ => None
                }
            }

            InputType::MouseRelease(btn) => {
                if self.taiko_settings.ignore_mouse_buttons { return None }
                
                match btn {
                    MouseButton::Left => Some(ReplayAction::Release(KeyPress::LeftDon)),
                    MouseButton::Right => Some(ReplayAction::Release(KeyPress::LeftKat)),
                    _ => None
                }
            }

            InputType::ControllerPress(
                btn,
                id, 
                name
            ) => {
                if let Some(config) = self
                    .taiko_settings
                    .controller_config
                    .get(&name)
                {
                    self.map_gamepad_button(config, btn)
                        .map(ReplayAction::Press)

                } else {
                    trace!("Controller with no setup");

                    // TODO: if this is slow, we should store controller configs separately
                    // but i dont think this will be an issue, as its unlikely to happen in the first place,
                    // and if there is lag, the user is likely to retry the man anyways
                    trace!("Setting up new controller {name}");
                    let mut new_settings = self.taiko_settings
                        .as_ref()
                        .clone();

                    new_settings.controller_config.insert(
                        name.clone(), 
                        TaikoControllerConfig::defaults(&name)
                    );

                    // // update the global settings
                    // {
                    //     let mut settings = Settings::get_mut();
                    //     settings.taiko_settings = new_settings.clone();
                    //     // settings.save().await;
                    // }
                    
                    self.taiko_settings = Arc::new(new_settings);
                    // rerun the handler now that the thing is setup
                    self.handle_input(
                        InputEvent {
                            event: InputType::ControllerPress(btn, id, name),
                            ..input
                        }
                    )
                }
            }

            InputType::ControllerRelease(
                btn, 
                id, 
                name
            ) => {
                if let Some(config) = self
                    .taiko_settings
                    .controller_config
                    .get(&name) 
                {
                    self.map_gamepad_button(config, btn)
                        .map(ReplayAction::Release)
                } else {
                    trace!("Controller with no setup");

                    // TODO: if this is slow, we should store controller configs separately
                    // but i dont think this will be an issue, as its unlikely to happen in the first place,
                    // and if there is lag, the user is likely to retry the map anyways
                    trace!("Setting up new controller");
                    let mut new_settings = self.taiko_settings
                        .as_ref()
                        .clone();

                    new_settings.controller_config.insert(
                        name.clone(), 
                        TaikoControllerConfig::defaults(&name)
                    );

                    // // update the global settings
                    // {
                    //     let mut settings = Settings::get_mut();
                    //     settings.taiko_settings = new_settings.clone();
                    //     // settings.save(&mut self.actions);
                    // }
                    
                    self.taiko_settings = Arc::new(new_settings);

                    // rerun the handler now that the thing is setup
                    self.handle_input(
                        InputEvent {
                            event: InputType::ControllerRelease(btn, id, name),
                            ..input
                        }
                    )
                }
            }
            

            _ => None
        }
    }
}
