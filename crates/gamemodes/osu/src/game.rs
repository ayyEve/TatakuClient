use std::ops::Range;
use crate::prelude::*;
use common::replays::*;
use std::f32::consts::PI;
#[cfg(feature="graphics")] use engine::graphics;

use tataku::{
    Color,
    Border,
    Easing,
    Vector2,
    EmitterVal,
};
use engine::{
    actions,
    beatmaps::{
        Beatmap,
        BeatmapMeta,
        map_difficulty,
    },
    gameplay,
    gameplay::{
        mods::*,
        Gamemode,
        Hitsound,
        judgments::*,
        GameplayEvent,
        TimingPointProgress,
        PlayfieldNonsense,
        GamemodeProperties,
        gameplay_manager::*,
    },
};
use input::gilrs::Axis;

const STACK_LENIENCY:u32 = 3;
pub const PREEMPT_MIN:f32 = 450.0;

/// how long a single shake is
const SHAKE_TIME:f32 = 40.0;
/// how many shakes to perform
const SHAKE_COUNT:usize = 3;
/// can a shake request inturrupt another shake?
const SHAKE_INTURRUPT: bool = true;

pub struct OsuGame {
    // lists
    pub notes: Vec<HitObject>,
    pub notes_skip: usize,

    actions: actions::ActionQueue,

    // hit timing bar stuff
    hit_windows: Vec<(HitJudgment, Range<f32>)>,
    miss_window: f32,

    mouse_pos: Vector2,
    window_mouse_pos: Vector2,

    /// original, mouse_start
    move_playfield: Option<(Vector2, Vector2)>,

    /// how many keys are being held?
    hold_count: u16,

    coords: OsuCoords,
    /// needed for scaling recalc
    cs: f32,
    stack_leniency: f32,

    fade_in_time: f32,
    fade_out_time: f32,

    last_beat: f32,
    beat_next_update: bool,
    pulse_length: f32,

    kiai: bool,

    settings: Settings,

    auto_replay: AutoReplay,
    relax_play: RelaxPlay,

    /// list of note_indices which are new_combos
    #[cfg(feature="graphics")] new_combos: Vec<usize>,
    #[cfg(feature="graphics")] beatmap_combo_colors: Vec<Color>,
    #[cfg(feature="graphics")] skin_combo_colors: Vec<Color>,

    use_controller_cursor: bool,
    start_time: f32,
    end_time: f32,

    #[cfg(feature="graphics")] cursor: OsuCursor,
    #[cfg(feature="graphics")] smoke_emitter: Option<tataku_graphics::Emitter>,
    #[cfg(feature="graphics")] follow_point_image: Option<tataku_graphics::Image>,
    #[cfg(feature="graphics")] judgement_images: JudgmentImages,

    metadata: Arc<BeatmapMeta>,
    mods: Arc<Mods>,

    new_playfield_pending: bool,
}
impl OsuGame {

    #[cfg(feature="graphics")]
    fn recalculate_playfield(&mut self, window_size: Vector2) {
        let new_coords = OsuCoords::new_with_settings(
            &self.settings,
            self.cs,
            window_size,
            self.mods.has_mod(HardRock)
        );

        self.new_playfield_pending = true;
        self.coords = new_coords;
    }

    // TODO: finish this
    #[allow(dead_code, unused_variables)]
    fn apply_stacking(&mut self) {
        let stack_offset = self.coords.cs / 10.0;

        let stack_vector = Vector2::ONE * stack_offset;

        // let stack_threshhold = self.preempt * self.beatmap.stack_leniency

        // // reset stack counters
        // for note in self.notes.iter_mut() {
        //     note.set_stack_count(0)
        // }


        // Extend the end index to include objects they are stacked on
        // let mut extended_end_index = self.notes.len();

        // let mut stack_base_index = self.notes.len();
        // loop {
        //     for n in (stack_base_index + 1)..self.notes.len() {
        //         let obj = &self.notes[stack_base_index];
        //         if obj.note_type() == NoteType::Spinner { break }

        //         let obj_n = &self.notes[n];
        //         if obj_n.note_type() == NoteType::Spinner { break }

        //         let stack_threshhold = obj_n.get_preempt() * self.stack_leniency;

        //         if obj_n.time() - obj.time() > stack_threshhold {
        //             // outside stack threshhold
        //             break;
        //         }

        //         let obj_pos = obj.pos_at(obj.time());
        //         let obj_n_pos = obj.pos_at(obj.time());
        //         let obj_is_slider = obj.note_type() == NoteType::Slider;
        //         let obj_end_pos = obj.pos_at(obj.end_time(0.0));

        //         if obj_pos.distance(obj_n_pos) < STACK_LENIENCY as f32 || (obj_is_slider && obj_end_pos.distance(obj_n_pos) < STACK_LENIENCY as f32) {
        //             stack_base_index = n;

        //             // self.notes[n].set_stack_count(0)
        //         }
        //     }


        //     if stack_base_index > extended_end_index {
        //         extended_end_index = stack_base_index;
        //         if extended_end_index == self.notes.len() - 1 {break}
        //     }

        //     // check loop
        //     // if stack_base_index == 0 {
        //     //     stack_base_index -= 1
        //     // } else {
        //     //     break
        //     // }
        // }



        // Reverse pass for stack calculation.
        let extended_start_index = self.notes.len() - 1;

    }

    fn setup_hitwindows(&mut self) {
        // windows
        let od = Self::get_od(&self.metadata, &self.mods);
        let w_miss = map_difficulty(od, 225.0, 175.0, 125.0); // idk
        let w_50   = map_difficulty(od, 200.0, 150.0, 100.0);
        let w_100  = map_difficulty(od, 140.0, 100.0, 60.0);
        let w_300  = map_difficulty(od, 80.0, 50.0, 20.0);
        self.miss_window = w_miss;

        self.hit_windows = vec![
            (OsuHitJudgments::X300, 0.0..w_300),
            (OsuHitJudgments::X100, w_300..w_100),
            (OsuHitJudgments::X50, w_100..w_50),
            (OsuHitJudgments::Miss, w_50..w_miss),
        ];
    }

    #[cfg(feature="graphics")]
    fn add_judgement_indicator(
        pos: Vector2,
        hit_value: &HitJudgment,
        coords: &OsuCoords,
        judgement_images: &JudgmentImages,
        settings: &Settings,
        state: &mut GameplayUpdateShell<'_>
    ) {
        if hit_value.tex_name.is_empty() { return }

        let pos = coords.to_window(pos);

        let color = hit_value.color;
        let mut image = settings.use_skin_judgments.then_some(())
            .and_then(|_| judgement_images.get_from_scorehit(hit_value));

        if let Some(image) = &mut image {
            let transform = graphics::Transform {
                pos,
                scale: Vector2::ONE * coords.cs,
                ..graphics::Transform::IDENTITY
            };

            state.add_indicator(ImageJudgementIndicator::new(
                image.clone(),
                transform
            ));
        } else {
            let transform = graphics::Transform {
                pos,
                scale: Vector2::ONE * CIRCLE_RADIUS_BASE * coords.cs * (1.0 / 3.0),
                ..graphics::Transform::IDENTITY
            };

            state.add_indicator(BasicJudgementIndicator::new(
                color,
                transform
            ));
        }

    }


    #[inline]
    fn scale_by_mods<V:std::ops::Mul<Output=V>>(
        val:V,
        ez_scale: V,
        hr_scale: V,
        mods: &Mods
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
    pub fn get_ar(meta: &BeatmapMeta, mods: &Mods) -> f32 {
        Self::scale_by_mods(meta.ar, 0.5, 1.4, mods).clamp(1.0, 11.0)
    }

    #[inline]
    pub fn get_od(meta: &BeatmapMeta, mods: &Mods) -> f32 {
        Self::scale_by_mods(meta.od, 0.5, 1.4, mods).clamp(1.0, 10.0)
    }

    #[inline]
    pub fn get_cs(meta: &BeatmapMeta, mods: &Mods) -> f32 {
        Self::scale_by_mods(meta.cs, 0.5, 1.3, mods).clamp(1.0, 10.0)
    }

    #[cfg(feature="graphics")]
    fn draw_follow_points(
        &mut self,
        time: f32,
        list: &mut engine::graphics::RenderableCollection,
    ) {
        if !self.settings.draw_follow_points { return; }
        if self.notes.is_empty() { return }

        let follow_dot_size = 3.0 * self.coords.scale;
        let follow_dot_distance = 20.0 * self.coords.scale;

        for i in self.notes_skip..self.notes.len() - 1 {
            if self.new_combos.contains(&(i + 1)) { continue }

            let n1 = &self.notes[i];
            let n2 = &self.notes[i + 1];

            let n1_time = n1.time();
            let n1_pos = match n1 {
                HitObject::Circle(note) => note.pos,
                HitObject::Slider(slider) => slider.pos,
                HitObject::Spinner(_) => continue,
            };

            let n2_time = n2.time();
            let n2_pos = match n2 {
                HitObject::Circle(note) => note.pos,
                HitObject::Slider(slider) => slider.pos,
                HitObject::Spinner(_) => continue,
            };

            if time < n1_time - self.fade_in_time { break; }
            if time >= n2_time { continue; }

            // setup follow points and the time they should exist at
            let distance = n1_pos.distance(n2_pos);
            if distance < follow_dot_distance { continue }

            let direction = PI * 2.0 - Vector2::atan2(n2_pos - n1_pos);
            let follow_dot_count = distance / follow_dot_distance;
            for i in 1..(follow_dot_count.min(1.0) as u64 - 1) {
                let lerp_amount = i as f32 / follow_dot_count;
                let time_at_this_point = f32::lerp(n1_time, n2_time, lerp_amount);
                let point = Vector2::lerp(n1_pos, n2_pos, lerp_amount);

                // get the alpha
                let alpha_lerp_amount = (time_at_this_point - time) / (n2_time - n1_time);
                let alpha = if !(0.0..=2.0).contains(&alpha_lerp_amount) {
                    0.0
                } else if alpha_lerp_amount > 1.0 {
                    f32::easeout_sine(1.0, 0.0, alpha_lerp_amount - 1.0)
                } else {
                    f32::easein_sine(0.0, 1.0, alpha_lerp_amount)
                };

                if alpha == 0.0 { continue }

                // add point
                if let Some(i) = self.follow_point_image.clone() {
                    let transform = graphics::Transform {
                        pos: point,
                        rotation: direction,
                        // scale: Vector2::ONE * self.coords.scale,
                        ..graphics::Transform::IDENTITY
                    };

                    list.push(i.with_transform(transform.matrix()));
                } else {
                    let transform = graphics::Transform {
                        pos: point,
                        scale: Vector2::ONE * follow_dot_size,
                        ..graphics::Transform::IDENTITY
                    };

                    list.push(engine::graphics::Circle::new(
                        Color::WHITE.with_alpha_f32(alpha),
                    ).with_transform(transform.matrix()));
                }
            }
        }
    }

    fn map_key(&self, key: &input::Key) -> Option<KeyPress> {
        if key == &self.settings.left_key {
            Some(KeyPress::Left)
        } else if key == &self.settings.right_key {
            Some(KeyPress::Right)
        } else if key == &self.settings.smoke_key {
            Some(KeyPress::Dash)
        } else {
            None
        }
    }

    fn map_btn(&self, btn: &input::MouseButton) -> Option<KeyPress> {
        if btn == &input::MouseButton::Left {
            Some(KeyPress::LeftMouse)
        } else if btn == &input::MouseButton::Right {
            Some(KeyPress::RightMouse)
        } else {
            None
        }
    }
}
impl Gamemode for OsuGame {
    fn new(
        map: &Beatmap,
        _diff_calc_only: bool,
        engine_settings: &engine::Settings,
    ) -> tataku::Result<Self> {
        let metadata = map.get_beatmap_meta();
        let mods = Arc::default();
        let effective_window_size = super::diff_calc::WINDOW_SIZE;

        let settings = engine_settings.gamemode_settings(crate::GAME_INFO).unwrap_or_default();
        // settings.osu_settings.clone();

        let cs = Self::get_cs(&metadata, &mods);
        let ar = Self::get_ar(&metadata, &mods);
        let od = Self::get_od(&metadata, &mods);
        let coords = OsuCoords::new_with_settings(&settings, cs, effective_window_size, mods.has_mod(HardRock));

        let fade_time = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);

        let timing_points = TimingPointProgress::new_from_beatmap(map);

        let parent_dir = map.get_parent_dir().unwrap_or_default().to_string_lossy().to_string();
        let mut actions = actions::ActionQueue::new();

        #[cfg(feature="graphics")]
        let cursor = {
            let cursor = OsuCursor::new(
                graphics::SkinSettings::default(),
                parent_dir,
                engine_settings.cursor_settings.clone(),
            );

            cursor.init(&mut actions);
            cursor
        };

        let mut s = match map {
            Beatmap::Osu(beatmap) => {
                use engine::beatmaps::osu::*;
                let stack_leniency = beatmap.stack_leniency;

                let mut s = Self {
                    actions,
                    notes: Vec::new(),
                    notes_skip: 0,
                    mouse_pos: Vector2::ZERO,
                    window_mouse_pos: Vector2::ZERO,
                    hit_windows: Vec::new(),
                    miss_window: 0.0,

                    hold_count: 0,
                    start_time: 0.0,
                    end_time: 0.0,

                    move_playfield: None,
                    coords,
                    cs,

                    fade_in_time: fade_time,
                    fade_out_time: 100.0,

                    last_beat: -1000.0,
                    beat_next_update: false,
                    pulse_length: 1.0,

                    kiai: false,

                    use_controller_cursor: false,

                    settings,
                    auto_replay: AutoReplay::default(),
                    relax_play: RelaxPlay::default(),

                    #[cfg(feature="graphics")] new_combos: Vec::new(),
                    stack_leniency,
                    // window_size,
                    #[cfg(feature="graphics")] follow_point_image: None,
                    #[cfg(feature="graphics")] judgement_images: JudgmentImages::new(OsuHitJudgments::variants().to_vec()),
                    metadata,
                    mods,
                    #[cfg(feature="graphics")] smoke_emitter: None,
                    #[cfg(feature="graphics")] cursor,

                    #[cfg(feature="graphics")] beatmap_combo_colors: beatmap.combo_colors.clone(),
                    #[cfg(feature="graphics")] skin_combo_colors: Vec::new(),
                    new_playfield_pending: false,
                };


                enum HitObjectRef<'a> {
                    Note(&'a NoteDef),
                    Slider(&'a SliderDef, Option<Box<Curve>>),
                    Spinner(&'a SpinnerDef),
                }
                impl HitObjectRef<'_> {
                    fn time(&self) -> f32 {
                        match self {
                            Self::Note(n) => n.time,
                            Self::Slider(s, _) => s.time,
                            Self::Spinner(s) => s.time,
                        }
                    }
                    fn end_time(&self) -> f32 {
                        match self {
                            Self::Note(n) => n.time,
                            Self::Slider(s, None) => s.time, // invisible note
                            Self::Slider(_, Some(c)) => c.end_time,
                            Self::Spinner(s) => s.end_time,
                        }
                    }
                    fn new_combo(&self) -> bool {
                        match self {
                            Self::Note(n) => n.new_combo,
                            Self::Slider(s, _) => s.new_combo,
                            Self::Spinner(_) => true,
                        }
                    }
                    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                        self
                            .time()
                            .partial_cmp(&other.time())
                            .unwrap_or(std::cmp::Ordering::Equal)
                    }
                }

                let mut all_items =
                    beatmap.notes.iter().map(HitObjectRef::Note)
                    .chain(beatmap.sliders.iter().map(|s| HitObjectRef::Slider(
                        s,
                        if s.curve_points.is_empty() || s.length == 0.0 {
                            None
                        } else {
                            Some(Box::new(get_curve(s, map, &timing_points)))
                        }
                    )))
                    .chain(beatmap.spinners.iter().map(HitObjectRef::Spinner))
                    .collect::<Vec<_>>();

                // sort all the notes
                all_items.sort_by(|a, b| a.cmp(b));

                // add notes
                let mut combo_num = 0;
                let mut color_index = 0;

                for (counter, i) in all_items.into_iter().enumerate() {

                    // update end time
                    s.end_time = s.end_time.max(i.end_time());

                    // reset combo if hitobject says so
                    if i.new_combo() {
                        combo_num = 0;

                        color_index += 1;

                        #[cfg(feature="graphics")]
                        s.new_combos.push(counter);
                    }

                    // update combo number
                    combo_num += 1;

                    // add the hitobject
                    match i {
                        HitObjectRef::Note(note) => {
                            s.notes.push(Note::new(
                                note,
                                combo_num as u16,
                                color_index,
                            ).into());
                        }
                        HitObjectRef::Slider(slider, None) => {
                            let note = NoteDef {
                                pos: slider.pos,
                                time: slider.time,
                                hitsound: slider.hitsound,
                                hitsamples: slider.hitsamples.clone(),
                                new_combo: slider.new_combo,
                                color_skip: slider.color_skip,
                            };

                            s.notes.push(Note::new(
                                &note,
                                combo_num as u16,
                                color_index,
                            ).into());
                        }
                        HitObjectRef::Slider(slider, Some(curve)) => {
                            s.notes.push(Slider::new(
                                slider,
                                *curve,
                                combo_num as u16,
                                color_index,
                                timing_points.slider_velocity_at(slider.time)
                            ).into());
                        }
                        HitObjectRef::Spinner(spinner) => {
                            let duration = spinner.end_time - spinner.time;
                            let min_rps = engine::beatmaps::map_difficulty(od, 2.0, 4.0, 6.0) * 0.6;

                            let mut spins_required = (duration / 1000.0 * min_rps) as u16;
                            // fudge until we can properly calculate
                            if spins_required < 10 {
                                if spins_required > 2 {
                                    spins_required = 2;
                                } else {
                                    spins_required = 0;
                                }
                            }

                            s.notes.push(Spinner::new(
                                spinner,
                                spins_required
                            ).into());
                        }
                    }
                }

                s
            }

            _ => return Err(errors::beatmap::BeatmapError::UnsupportedMode.into()),
        };

        s.start_time = s.notes.first()
            .map(HitObject::time)
            .unwrap_or_default()
            - 5000.0;

        // wait an extra sec
        s.end_time += 1000.0;

        s.setup_hitwindows();

        Ok(s)
    }

    fn handle_replay_frame(
        &mut self,
        frame: ReplayFrame,
        shell: &mut GameplayUpdateShell
    ) {
        const ALLOWED_PRESSES:&[KeyPress] = &[
            KeyPress::Left,
            KeyPress::Right,
            KeyPress::Dash,
            KeyPress::LeftMouse,
            KeyPress::RightMouse,
        ];

        match frame.action {
            ReplayAction::Press(key) if ALLOWED_PRESSES.contains(&key) => {
                self.hold_count += 1;

                #[cfg(feature="graphics")]
                match key {
                    KeyPress::Left | KeyPress::LeftMouse => self.cursor.left_pressed(true),
                    KeyPress::Right | KeyPress::RightMouse => self.cursor.right_pressed(true),
                    KeyPress::Dash => {
                        for i in self.smoke_emitter.iter_mut() {
                            i.should_emit = true;
                        }
                        return;
                    }
                    _ => {}
                }

                let mouse_pos = self.coords.to_osu(self.mouse_pos);
                let cs = CIRCLE_RADIUS_BASE * self.coords.cs / self.coords.scale;

                let iter = self.notes.iter_mut()
                    .skip(self.notes_skip);

                for note in iter {
                    let note_time = note.time();

                    if frame.time < note_time - self.fade_in_time {
                        break;
                    }

                    let shake = (note_time - frame.time).abs() > self.miss_window;

                    match note {
                        HitObject::Circle(note) => {
                            if note.hit_time.is_some() || note.missed { continue; }

                            let was_hit = note.pos.distance_squared(mouse_pos) <= cs * cs;

                            if was_hit {
                                if shake {
                                    if SHAKE_INTURRUPT || note.shake.animation.is_none() {
                                        note.shake.animation = Some(shake_animation(
                                            frame.time,
                                            SHAKE_TIME,
                                            SHAKE_COUNT,
                                            Easing::EaseInOutSine,
                                        ));
                                    }
                                } else if let Some(judge) = shell.check_judgment(&self.hit_windows, frame.time, note_time) {
                                    if judge == &OsuHitJudgments::X300 && !self.settings.show_300s {
                                        // dont show the judgment
                                    } else {
                                        #[cfg(feature="graphics")]
                                        Self::add_judgement_indicator(
                                            note.pos,
                                            judge,
                                            &self.coords,
                                            &self.judgement_images,
                                            &self.settings,
                                            shell
                                        );
                                    }

                                    if judge == &OsuHitJudgments::Miss {
                                        // tell the note it was missed
                                        note.missed = true;
                                    } else {
                                        // tell the note it was hit
                                        note.hit_time = Some(frame.time);

                                        let timing_point = shell.timing_points.timing_point_at(note.time, true);

                                        let hitsounds = Hitsound::from_hitsamples(
                                            note.hitsound,
                                            note.hitsamples.clone(),
                                            true,
                                            timing_point,
                                        );

                                        // play the sound
                                        #[cfg(feature="gameplay")]
                                        shell.play_hitsounds(&hitsounds, false);
                                    }
                                }

                                break;
                            }
                        },
                        HitObject::Slider(slider) => {
                            if slider.initial_hit.is_some() { continue; }

                            let in_hit_window = frame.time < slider.time + self.miss_window;

                            let was_hit = slider.pos.distance_squared(mouse_pos) <= cs * cs;

                            if in_hit_window && was_hit {
                                if shake {
                                    if SHAKE_INTURRUPT || slider.shake.animation.is_none() {
                                        slider.shake.animation = Some(shake_animation(
                                            frame.time,
                                            SHAKE_TIME,
                                            SHAKE_COUNT,
                                            Easing::Linear,
                                        ));
                                    }
                                } else if let Some(judge) = shell.check_judgment(&self.hit_windows, frame.time, note_time) {
                                    slider.initial_hit = Some(*judge);

                                    if judge == &OsuHitJudgments::X300 && !self.settings.show_300s {
                                        // dont show the judgment
                                    } else {
                                        #[cfg(feature="graphics")]
                                        Self::add_judgement_indicator(
                                            slider.pos,
                                            judge,
                                            &self.coords,
                                            &self.judgement_images,
                                            &self.settings,
                                            shell
                                        );
                                    }

                                    let timing_point = shell.timing_points.timing_point_at(slider.time, true);

                                    let hitsounds = Hitsound::from_hitsamples(
                                        slider.hitsound,
                                        slider.hitsamples.clone(),
                                        true,
                                        timing_point,
                                    );

                                    // play the sound
                                    #[cfg(feature="gameplay")]
                                    shell.play_hitsounds(&hitsounds, false);
                                }

                                break;
                            }
                        },
                        HitObject::Spinner(_) => {},
                    }
                }
            }
            // dont continue if no keys were being held (happens when leaving a menu)
            ReplayAction::Release(key) if ALLOWED_PRESSES.contains(&key) && self.hold_count > 0 => {
                self.hold_count -= 1;

                #[cfg(feature="graphics")]
                match key {
                    KeyPress::Left | KeyPress::LeftMouse => self.cursor.left_pressed(false),
                    KeyPress::Right | KeyPress::RightMouse => self.cursor.right_pressed(false),
                    KeyPress::Dash => {
                        if let Some(i) = self.smoke_emitter.as_mut() {
                            i.should_emit = false;
                        }
                    }
                    _ => {}
                }
            }
            ReplayAction::MousePos(x, y) => {
                // scale the coords from playfield to window
                let pos = self.coords.to_window(Vector2::new(x, y));
                self.mouse_pos = pos;
                #[cfg(feature="graphics")]
                if let Some(emitter) = &mut self.smoke_emitter {
                    emitter.position = pos;
                }
                #[cfg(feature="graphics")] {
                    self.cursor.pos = pos;
                }
            }
            _ => {}
        }
    }

    fn handle_gameplay_event(&mut self, shell: &mut GameplayCustomShell<'_, GameplayEvent>) {
        let event = shell.custom.clone();

        match event {
            GameplayEvent::SetBounds { bounds, full_window } => {
                #[cfg(feature="graphics")]
                if full_window {
                    // self.window_size = window_size;
                    self.recalculate_playfield(bounds.size);
                } else {
                    self.coords = OsuCoords::new_offset_scale(
                        self.cs,
                        bounds.size,
                        bounds.pos,
                        0.80,
                        self.mods.has_mod(HardRock)
                    );
                }
            }

            GameplayEvent::ApplyMods(mods) => {
                let had_easy_or_hr = self.mods.has_mod(Easy) || self.mods.has_mod(HardRock);

                let has_hr = mods.has_mod(HardRock);
                let has_easy_or_hr = mods.has_mod(Easy) || has_hr;

                let had_otb = self.mods.has_mod(OnTheBeat);
                let has_otb = mods.has_mod(OnTheBeat);

                // check easing type
                let easing_type_names = ["in", "out", "inout"];
                let mut last_easing_type = "";
                let mut new_easing_type = "";
                for i in easing_type_names {
                    if self.mods.has_mod(i) { last_easing_type = i }
                    if mods.has_mod(i) { new_easing_type = i }
                }

                // check easing
                let easing_names = ["sine", "quad", "cube", "quart", "quint", "exp", "circ", "back"];
                let mut last_easing = "";
                let mut new_easing = "";
                for i in easing_names {
                    if self.mods.has_mod(i) { last_easing = i }
                    if mods.has_mod(i) { new_easing = i }
                }

                self.mods = mods;

                let mut set_easing = None;

                if has_easy_or_hr || had_easy_or_hr != has_easy_or_hr {
                    self.cs = Self::get_cs(&self.metadata, &self.mods);
                    let ar = Self::get_ar(&self.metadata, &self.mods);

                    let fade_time = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);

                    self.fade_in_time = fade_time;
                    self.fade_out_time = fade_time;

                    #[cfg(feature="graphics")]
                    self.recalculate_playfield(self.coords.window_size);
                    self.setup_hitwindows();
                }

                if last_easing != new_easing || last_easing_type != new_easing_type {
                    // use out as default easing type
                    if new_easing_type.is_empty() && !new_easing.is_empty() {
                        new_easing_type = "out";
                    }

                    let easing = match (new_easing_type, new_easing) {
                        // sine
                        ("in", "sine") => Easing::EaseInSine,
                        ("out", "sine") => Easing::EaseOutSine,
                        ("inout", "sine") => Easing::EaseInOutSine,
                        // quadratic
                        ("in", "quad") => Easing::EaseInQuadratic,
                        ("out", "quad") => Easing::EaseOutQuadratic,
                        ("inout", "quad") => Easing::EaseInOutQuadratic,
                        // cubic
                        ("in", "cube") => Easing::EaseInCubic,
                        ("out", "cube") => Easing::EaseOutCubic,
                        ("inout", "cube") => Easing::EaseInOutCubic,
                        // quartic
                        ("in", "quart") => Easing::EaseInQuartic,
                        ("out", "quart") => Easing::EaseOutQuartic,
                        ("inout", "quart") => Easing::EaseInOutQuartic,
                        // quintic
                        ("in", "quint") => Easing::EaseInQuintic,
                        ("out", "quint") => Easing::EaseOutQuintic,
                        ("inout", "quint") => Easing::EaseInOutQuintic,
                        // exponential
                        ("in", "exp") => Easing::EaseInExponential,
                        ("out", "exp") => Easing::EaseOutExponential,
                        ("inout", "exp") => Easing::EaseInOutExponential,
                        // // circular
                        // ("in", "circ") => Easing::EaseInCircular,
                        // ("out", "circ") => Easing::EaseOutCircular,
                        // // back
                        // ("in", "back") => Easing::EaseInBack      (1.7, 1.7 * 1.525),
                        // ("out", "back") => Easing::EaseOutBack    (1.7, 1.7 * 1.525),
                        // ("inout", "back") => Easing::EaseInOutBack(1.7, 1.7 * 1.525),
                        _ => Easing::Linear
                    };

                    set_easing = Some(easing);
                }

                #[cfg(feature="graphics")]
                if has_otb != had_otb {
                    if has_otb {
                        let timing_points = shell.timing_points
                            .iter()
                            .filter(|t| !t.is_inherited())
                            .copied()
                            .collect::<Vec<_>>();
                        let mut index = 0;
                        // info!("tp: {} -> {}", timing_points[index].time, timing_points[index].beat_length);

                        for note in self.notes.iter_mut() {
                            let note_time = note.time();

                            let approach_circle = match note {
                                HitObject::Circle(note) => &mut note.approach_circle,
                                HitObject::Slider(slider) => &mut slider.approach_circle,
                                HitObject::Spinner(_) => continue,
                            };

                            // check next timing point
                            if let Some(next) = timing_points.get(index + 1)
                            && next.time <= note_time {
                                index += 1;
                                // info!("tp: {} -> {}", timing_points[index].time, timing_points[index].beat_length);
                            }

                            // get the beat length of the current timing point
                            let beat_length = timing_points[index].beat_length;

                            // normalize the note time to "align" with the control point time offset
                            let normalized_time = note_time - timing_points[index].time; // beat lengths with decimal points
                            let m = beat_length - (normalized_time % beat_length); // beat lengths without a decimal point
                            let m2 = normalized_time % beat_length;
                            // info!("{normalized_time}, {m}, {m2}");

                            // if this note lands on a beat, or within 10ms of a beat, make it ~funky~
                            if m < 10.0 || m2 < 10.0 {
                                approach_circle.easing_type = Easing::EaseOutExponential;
                            } else {
                                approach_circle.easing_type = Easing::Linear;
                            }

                        }

                        set_easing = None;
                    } else {
                        set_easing = Some(Easing::Linear);
                    }

                }

                if set_easing.is_some() {
                    for note in self.notes.iter_mut() {
                        #[cfg(feature="graphics")]
                        if let Some(easing) = set_easing {
                            let approach_circle = match note {
                                HitObject::Circle(note) => &mut note.approach_circle,
                                HitObject::Slider(slider) => &mut slider.approach_circle,
                                HitObject::Spinner(_) => continue,
                            };

                            approach_circle.easing_type = easing;
                        }
                    }
                }

            }

            GameplayEvent::BeatHappened { pulse_length } => {
                self.beat_next_update = true;
                self.pulse_length = pulse_length;
            }
            GameplayEvent::KiaiChanged { enabled } => {
                self.kiai = enabled;
            }

            _ => {}
        }
    }

    fn update(
        &mut self,
        shell: &mut GameplayUpdateShell,
    ) {
        shell.action_queue.extend(self.actions.take());

        if self.new_playfield_pending {
            self.new_playfield_pending = false;
            shell.add_action(gameplay::Action::PlayfieldChanged);
        }

        if self.beat_next_update.take() {
            self.last_beat = shell.time;
        }

        // disable the cursor particle emitter if this is a menu game
        // the emitter nukes perf so its best to keep it off unless needed
        #[cfg(feature="graphics")]
        if shell.gameplay_type.is_preview() && self.cursor.emitter_enabled {
            self.cursor.emitter_enabled = false;
        }
        #[cfg(feature="graphics")]
        self.cursor.update();

        let has_autoplay: bool = shell.mods.has_autoplay();
        let has_relax = shell.mods.has_mod(Relax);

        // do autoplay things
        if has_autoplay {
            let mut pending_frames = Vec::new();
            self.auto_replay.update(
                shell.time,
                &self.notes,
                self.notes_skip,
                &mut pending_frames
            );

            // // handle presses and mouse movements now, and releases later
            for action in pending_frames {
                shell.add_replay_action(action);
            }
        }

        if has_relax {
            self.relax_play.update(shell.time);
        }

        // update emitter
        #[cfg(feature="graphics")]
        if let Some(e) = self.smoke_emitter.as_mut() { e.update(shell.time) }

        // if the map is over, say it is
        if shell.time >= self.end_time {
            if !shell.complete() {
                shell.add_action(gameplay::Action::MapComplete);
            }
            return;
        }

        let mut hitsounds = Vec::new();

        // update notes
        let iter = self.notes.iter_mut()
            .enumerate()
            .skip(self.notes_skip);

        for (note_index, note) in iter {
            let mut update_shell = UpdateShell {
                time: shell.time,
                hitwindow_miss: self.miss_window,
                cursor_pos: self.mouse_pos,
                holding: self.hold_count > 0,
                settings: &self.settings,
                coords: &self.coords,
                timing_points: shell.timing_points,
                hitsounds: &mut hitsounds,
                font_contexts: shell.font_contexts,
            };

            note.update(&mut update_shell);

            let end_time = match note {
                HitObject::Circle(note) => note.time + self.miss_window,
                HitObject::Slider(slider) => slider.curve.end_time,
                HitObject::Spinner(spinner) => spinner.end_time,
            };

            // check relax stuff
            if has_relax && !has_autoplay {
                self.relax_play.check_note(
                    self.mouse_pos,
                    end_time,
                    note,
                    note_index,
                    &self.coords,
                    shell,
                );
            }

            match note {
                HitObject::Circle(_) => {},
                HitObject::Slider(slider) => {
                    let iter = slider.pending_combo.iter()
                        .skip(slider.judgements_added)
                        .copied();

                    for (judgement, pos) in iter {
                        shell.add_judgment(judgement);
                        #[cfg(feature="graphics")]
                        Self::add_judgement_indicator(
                            pos,
                            &judgement,
                            &self.coords,
                            &self.judgement_images,
                            &self.settings,
                            shell
                        );
                    }

                    slider.judgements_added = slider.pending_combo.len();
                },
                HitObject::Spinner(spinner) => {
                    for judgement in spinner.points_queue.take() {
                        shell.add_judgment(judgement);
                        #[cfg(feature="graphics")]
                        Self::add_judgement_indicator(
                            FIELD_SIZE / 2.0,
                            &judgement,
                            &self.coords,
                            &self.judgement_images,
                            &self.settings,
                            shell
                        );
                    }
                }
            }

            // check if note was missed
            if shell.time >= end_time + self.miss_window {
                if self.notes_skip + 1 == note_index {
                    self.notes_skip = note_index;
                }

                match note {
                    HitObject::Circle(note) => {
                        if note.hit_time.is_none() && !note.missed {
                            note.missed = true;

                            let j = OsuHitJudgments::Miss;
                            shell.add_judgment(j);

                            #[cfg(feature="graphics")]
                            Self::add_judgement_indicator(
                                note.pos,
                                &j,
                                &self.coords,
                                &self.judgement_images,
                                &self.settings,
                                shell
                            );
                        }
                    },
                    HitObject::Spinner(spinner) => {
                        if !spinner.judged {
                            spinner.judged = true;

                            // todo: other judgements
                            let judge = if spinner.rotations_completed >= spinner.rotations_required {
                                OsuHitJudgments::X300
                            } else {
                                OsuHitJudgments::SpinnerMiss
                            };

                            shell.add_judgment(judge);
                            #[cfg(feature="graphics")]
                            Self::add_judgement_indicator(
                                FIELD_SIZE / 2.0,
                                &judge,
                                &self.coords,
                                &self.judgement_images,
                                &self.settings,
                                shell
                            );
                        }
                    },
                    // checked above
                    HitObject::Slider(_) => {},
                }
            }
        }

        shell.play_hitsounds(&hitsounds, false);

        // handle note releases
        // required because autoplay frames are checked after the frame is processed
        // so if the key is released on the same frame its checked, it will count as not held
        // which makes sense, but we dont want that
        if has_autoplay {
            for action in self.auto_replay.get_release_queue() {
                shell.add_replay_action(action);
            }
        }

    }

    #[cfg(feature="graphics")]
    fn draw(
        &mut self,
        shell: GameplayDrawShell,
        list: &mut tataku_graphics::RenderableCollection
    ) {
        use engine::graphics;
        let window_size = shell.window_size;
        // draw the playfield
        if !shell.gameplay_type.is_preview() {
            let alpha = self.settings.playfield_alpha;

            let mut playfield_border = shell.current_timing_point.kiai
                .then_some(Border::new(Color::YELLOW.with_alpha_f32(alpha), 2.0));

            let playfield = self.coords.playfield_with_padding;

            if self.move_playfield.is_some() {
                let line_size = self.settings.playfield_movelines_thickness;
                // draw x and y center lines
                let px_line = graphics::Line::new(
                    Vector2::with_x(playfield.size.x),
                    line_size,
                    Color::WHITE
                ).with_transform(tataku::Matrix::IDENTITY
                    .trans(playfield.pos + Vector2::new(0.0, playfield.size.y/2.0))
                );
                let py_line = graphics::Line::new(
                    Vector2::with_y(playfield.size.y),
                    line_size,
                    Color::WHITE
                ).with_transform(tataku::Matrix::IDENTITY
                    .trans(playfield.pos + Vector2::new(playfield.size.x/2.0, 0.0))
                );

                let wx_line = graphics::Line::new(
                    Vector2::with_x(window_size.x),
                    line_size,
                    Color::WHITE
                ).with_transform(tataku::Matrix::IDENTITY
                    .trans(Vector2::new(0.0, window_size.y/2.0))
                );
                let wy_line = graphics::Line::new(
                    Vector2::with_y(window_size.y),
                    line_size,
                    Color::WHITE
                ).with_transform(tataku::Matrix::IDENTITY
                    .trans(
                    Vector2::new(window_size.x/2.0, 0.0))
                );

                playfield_border = Some(Border::new(
                    Color::WHITE,
                    line_size
                ));

                list.push(wx_line);
                list.push(wy_line);
                list.push(px_line);
                list.push(py_line);
            }

            let playfield = graphics::Rectangle::new(
                playfield.size,
                Color::BLACK.with_alpha_f32(alpha),
            ).border_maybe(playfield_border)
            .with_transform(tataku::Matrix::IDENTITY
                .trans(playfield.pos)
            );

            list.push(playfield);
        }

        let has_flashlight = self.mods.has_mod(Flashlight);
        // if flashlight is enabled, we want to scissor all items by the playfield
        // this prevents things like approach circles and ripples from showing up outside the flashlight radius
        if has_flashlight {
            // list.push_scissor(self.coords.playfield_with_padding.into_scissor());
            // todo: fix this
        }

        // draw cursor ripples
        self.cursor.draw_below(list);

        // draw follow points
        self.draw_follow_points(shell.time, list);

        {
            let combo_colors = if self.settings.use_beatmap_combo_colors && !self.beatmap_combo_colors.is_empty() {
                Cow::Borrowed(self.beatmap_combo_colors.as_slice())
            } else if !self.skin_combo_colors.is_empty() {
                Cow::Borrowed(self.skin_combo_colors.as_slice())
            } else {
                let colors = self.settings.combo_colors.iter()
                    .map(|c| c.color)
                    .collect::<Vec<_>>();

                Cow::Owned(colors)
            };

            let mut shell = DrawShell {
                time: shell.time,
                fade_in_time: self.fade_in_time,
                fade_out_time: self.fade_out_time,
                hitwindow_miss: self.miss_window,
                alpha: tataku::ColorField::MAX,
                combo_colors: &combo_colors,
                last_beat: self.last_beat,
                pulse_length: self.pulse_length,
                list,
                transform: tataku::Matrix::IDENTITY,
                settings: &self.settings,
                coords: &self.coords,
                slider_track_override: None, // todo:
            };

            // draw notes
            // spinners should be drawn last since they should be on top of everything
            // (we dont want notes or sliders drawn on top of the spinners)
            // todo: do this in a more efficient way
            let mut spinners = Vec::new();
            for note in self.notes.iter_mut().rev() {
                match note {
                    HitObject::Circle(note) => note.draw(&mut shell),
                    HitObject::Slider(slider) => slider.draw(&mut shell),
                    HitObject::Spinner(spinner) => spinners.push(spinner),
                }
            }

            for spinner in spinners {
                spinner.draw(&mut shell);
            }
        }

        // draw flashlight
        if has_flashlight {
            // list.pop_scissor();

            let radius = match shell.score.combo {
                0..=99 => 125.0,
                100..=199 => 100.0,
                _ => 75.0
            } * self.coords.scale;
            let fade_radius = radius / 5.0;

            list.push(graphics::FlashlightDrawable::new(
                self.mouse_pos,
                radius - fade_radius,
                fade_radius,
                self.coords.playfield_with_padding,
                Color::BLACK
            ));
        }

        // need to draw the smoke particles on top of everything
        if let Some(e) = self.smoke_emitter.as_ref() {
            e.draw(graphics::DrawOptions::default(), list);
        }

        // draw the cursor on top of smoke tho
        self.cursor.draw_above(list);
    }

    fn reset(&mut self, _: &GameplayCustomShell<'_, &Beatmap>) {
        self.notes_skip = 0;

        self.auto_replay.get_release_queue();

        // reset notes
        for note in self.notes.iter_mut() {
            note.reset();
        }

        // reset the smoke particles
        #[cfg(feature="graphics")] {
            if let Some(e) = self.smoke_emitter.as_mut() {
                e.reset(0.0);
            }
            self.cursor.reset();
        }
    }

    #[cfg(feature="gameplay")]
    fn skip_intro(&mut self, game_time: f32) -> Option<f32> {
        if self.start_time > 0.0 && game_time < self.start_time {
            Some(self.start_time)
        } else {
            None
        }
    }

    fn time_jump(
        &mut self,
        new_time: f32,
        _state: &mut GameplayUpdateShell
    ) {
        self.notes_skip = 0;

        for (i, note) in self.notes.iter_mut().enumerate() {
            note.reset();

            if note.time() < new_time {
                match note {
                    HitObject::Circle(note) => {
                        note.missed = true;
                    },
                    HitObject::Slider(slider) => {
                        slider.initial_hit = Some(OsuHitJudgments::Miss);
                    },
                    HitObject::Spinner(spinner) => {
                        spinner.judged = true;
                    },
                }

                if self.notes_skip + 1 == i {
                    self.notes_skip = i;
                }
            }
        }
    }

    fn force_update_settings(&mut self, settings: &engine::Settings) {
        let settings = settings.gamemode_settings::<Settings>(crate::GAME_INFO).unwrap_or_default();

        if self.settings == settings { return }

        self.settings = settings;
    }

    #[cfg(feature="graphics")]
    fn reload_skin(
        &mut self,
        beatmap_path: &str,
        skin_manager: &mut dyn engine::graphics::SkinProvider
    ) -> engine::graphics::TextureSource {
        use engine::graphics::{
            SkinUsage,
            TextureSource,
            EmitterBuilder,
        };

        let source = if self.settings.beatmap_skin {
            TextureSource::Beatmap(beatmap_path.to_owned())
        } else {
            TextureSource::Skin
        };

        self.cursor.reload_skin(skin_manager);
        self.judgement_images.reload_skin(skin_manager);
        self.follow_point_image = skin_manager.get_texture(
            Path::new("followpoint"),
            &source,
            SkinUsage::Gamemode,
            false
        );

        self.skin_combo_colors = skin_manager.skin().combo_colors.clone();

        for n in self.notes.iter_mut() {
            n.reload_skin(&source, skin_manager);
        }

        let smoke = skin_manager.get_texture(
            Path::new("cursor-smoke"),
            &source,
            SkinUsage::Gamemode,
            false
        ).map(|i| i.tex).unwrap_or_default();

        if let Some(emitter) = &mut self.smoke_emitter {
            emitter.image = smoke;
        } else {
            // create the emitter
            let emitter = EmitterBuilder::default()
                .should_emit(false)
                .spawn_delay(10.0_f32)
                .life(500.0..2000.0)
                .image(smoke)
                .scale(EmitterVal::init_only(0.8..1.5))
                .opacity(EmitterVal::new(1.0..1.0, 1.0..0.0))
                .rotation(EmitterVal::init_only(0.0..PI*2.0))
                .color(Color::WHITE)
                .build(0.0);

            self.smoke_emitter = Some(emitter);
        }

        source
    }

    #[cfg(feature="gameplay")]
    fn handle_input(&mut self, input: input::InputEvent) -> Option<ReplayAction> {
        use input::{
            Key,
            InputType,
            GamepadButton,
        };
        match input.event {
            InputType::KeyPress(press) => {
                let key = press.key?;

                // playfield adjustment
                if key == Key::LControl {
                    let old = self.settings.get_playfield();
                    self.move_playfield = Some((old.1, self.window_mouse_pos));
                    return None;
                }

                let key = self.map_key(&key)?;

                // if relax is enabled, and the user doesn't want manual input, return
                if self.mods.has_mod(Relax) {
                    if !self.settings.manual_input_with_relax { return None; }
                    self.relax_play.key_pressed(key);
                }

                Some(ReplayAction::Press(key))
            }

            InputType::KeyRelease(release) => {
                let key = release.key?;

                // playfield adjustment
                if key == Key::LControl {
                    self.move_playfield = None;
                    return None;
                }

                let key = self.map_key(&key)?;

                // if relax is enabled, and the user doesn't want manual input, return
                if self.mods.has_mod(Relax) {
                    if !self.settings.manual_input_with_relax { return None; }
                    self.relax_play.key_released(key);
                }

                Some(ReplayAction::Release(key))
            }

            InputType::MouseMove(pos) => {
                if self.use_controller_cursor {
                    // info!("switched to mouse");
                    self.use_controller_cursor = false;
                }
                self.window_mouse_pos = pos;

                if let Some((original, mouse_start)) = self.move_playfield {

                    let mut change = original + (pos - mouse_start);

                    // check playfield snapping
                    // TODO: can this be simplified?
                    let playfield_size = self.coords.playfield_with_padding.size;

                    // what the offset should be if playfield is centered
                    let center_offset = (self.coords.window_size - FIELD_SIZE * self.coords.scale) / 2.0 - (self.coords.window_size - playfield_size) / 2.0;

                    let snap_threshold = self.settings.playfield_snap;
                    if (center_offset.x - change.x).abs() < snap_threshold {
                        change.x = center_offset.x;
                    }
                    if (center_offset.y - change.y).abs() < snap_threshold {
                        change.y = center_offset.y;
                    }

                    self.settings.playfield_x_offset = change.x;
                    self.settings.playfield_y_offset = change.y;

                    let settings = self.settings.clone();

                    self.actions.push(actions::game::GameAction::UpdateSettings(Arc::new(
                        move |s|
                        s.update_gamemode_settings(
                            GAME_INFO,
                            settings.clone() // todo: remove clone
                        )
                    )).into());

                    self.recalculate_playfield(self.coords.window_size);
                    return None;
                }


                // convert window pos to playfield pos
                let pos = self.coords.to_osu(pos);
                Some(ReplayAction::MousePos(pos.x, pos.y))
            }

            InputType::MousePress(btn) => {
                // if the user has mouse input disabled, return
                if self.settings.ignore_mouse_buttons { return None }

                let button = self.map_btn(&btn)?;

                // if relax is enabled, and the user doesn't want manual input, return
                if self.mods.has_mod(Relax) {
                    if !self.settings.manual_input_with_relax { return None; }
                    self.relax_play.key_pressed(button);
                }

                Some(ReplayAction::Press(button))
            }

            InputType::MouseRelease(btn) => {
                // if the user has mouse input disabled, return
                if self.settings.ignore_mouse_buttons { return None }

                let button = self.map_btn(&btn)?;

                // if relax is enabled, and the user doesn't want manual input, return
                if self.mods.has_mod(Relax) {
                    if !self.settings.manual_input_with_relax { return None; }
                    self.relax_play.key_released(button);
                }

                Some(ReplayAction::Release(button))
            }

            InputType::MouseScroll(scroll) => {
                if self.move_playfield.is_some() {
                    let delta = scroll.get_delta(10.0).y;

                    self.settings.playfield_scale += delta;

                    let settings = self.settings.clone();
                    self.actions.push(actions::game::GameAction::UpdateSettings(Arc::new(
                        move |s|
                        s.update_gamemode_settings(
                            GAME_INFO,
                            settings.clone()
                        )
                    )).into());

                    self.recalculate_playfield(self.coords.window_size);
                }

                None
            }

            InputType::ControllerPress(btn, _id, _name) => {
                // if relax is enabled, and the user doesn't want manual input, return
                if self.mods.has_mod(Relax) && !self.settings.manual_input_with_relax { return None; }

                match btn {
                    GamepadButton::LeftTrigger => Some(ReplayAction::Press(KeyPress::Left)),
                    GamepadButton::RightTrigger => Some(ReplayAction::Press(KeyPress::Right)),
                    _ => None
                }
            }

            InputType::ControllerRelease(btn, _id, _name) => {
                // if relax is enabled, and the user doesn't want manual input, return
                if self.mods.has_mod(Relax) && !self.settings.manual_input_with_relax { return None; }

                match btn {
                    GamepadButton::LeftTrigger => Some(ReplayAction::Release(KeyPress::Left)),
                    GamepadButton::RightTrigger => Some(ReplayAction::Release(KeyPress::Right)),
                    _ => None
                }
            }

            InputType::ControllerAxis(Axis::LeftStickX, value, _id, _name) => {
                // -1.0 to 1.0
                // where -1 is 0, and 1 is coords.playfield_scaled_with_cs_border.whatever

                if !self.use_controller_cursor {
                    // info!("switched to controller input");
                    self.use_controller_cursor = true;
                }

                let mut new_pos = self.mouse_pos;
                let playfield = self.coords.playfield_with_padding;

                let normalized = (value + 1.0) / 2.0;
                new_pos.x = playfield.pos.x + f32::lerp(0.0, playfield.size.x, normalized);

                let new_pos = self.coords.to_osu(new_pos);
                Some(ReplayAction::MousePos(new_pos.x, new_pos.y))
            }

            InputType::ControllerAxis(Axis::LeftStickY, value, _id, _name) => {
                // y is upside down in gilrs i guess?

                if !self.use_controller_cursor {
                    // info!("switched to controller input");
                    self.use_controller_cursor = true;
                }


                let mut new_pos = self.mouse_pos;
                let playfield = self.coords.playfield_with_padding;

                let normalized = (value + 1.0) / 2.0;
                new_pos.y = playfield.pos.y + f32::lerp(playfield.size.y, 0.0, normalized);

                let new_pos = self.coords.to_osu(new_pos);
                Some(ReplayAction::MousePos(new_pos.x, new_pos.y))
            }

            _ => None
        }
    }

    #[cfg(feature="graphics")] fn get_playfield(&self) -> PlayfieldNonsense {
        PlayfieldNonsense::new(
            self.coords.playfield,
            self.coords.scale,
            self.coords.circle_size,
            self.mods.has_mod(HardRock)
        )
    }
    fn properties(&self, timing_points: &TimingPointProgress) -> GamemodeProperties {
        let mut sound_list = HashMap::new();
        #[cfg(feature="gameplay")]
        for note in self.notes.iter() {
            let hitsounds = match note {
                HitObject::Circle(note) => {
                    let timing_point = timing_points.timing_point_at(note.time, true);

                    Hitsound::from_hitsamples(
                        note.hitsound,
                        note.hitsamples.clone(),
                        true,
                        timing_point,
                    )
                },
                HitObject::Slider(slider) => {
                    let timing_point = timing_points.timing_point_at(slider.time, true);

                    let mut hitsounds = Hitsound::from_hitsamples(
                        slider.hitsound,
                        slider.hitsamples.clone(),
                        true,
                        timing_point,
                    );

                    const SAMPLE_SETS:[&str; 4] = ["normal", "normal", "soft", "drum"];

                    let sliderdot_hitsound = Hitsound::new_simple(format!("{}-slidertick", SAMPLE_SETS[slider.hitsamples.addition_set as usize]));

                    hitsounds.push(sliderdot_hitsound);

                    for (n, &[normal_set, addition_set]) in slider.edge_sets.iter().enumerate() {
                        let mut samples = slider.hitsamples.clone();
                        samples.normal_set = normal_set;
                        samples.addition_set = addition_set;

                        let hitsound = slider.edge_sounds[n.min(slider.edge_sounds.len() - 1)];

                        hitsounds.extend(Hitsound::from_hitsamples(
                            hitsound,
                            samples,
                            true,
                            timing_point,
                        ));
                    }

                    hitsounds
                },
                HitObject::Spinner(_) => continue,
            };

            for hitsound in hitsounds {
                #[expect(clippy::map_entry, reason = "never used")]
                if !sound_list.contains_key(&hitsound.get_id()) {
                    sound_list.insert(
                        hitsound.get_id(),
                        hitsound.load_data(None::<String>)
                    );
                }
            }
        }

        GamemodeProperties {
            info: &crate::GAME_INFO,
            keys: vec![
                (KeyPress::Left, "L"),
                (KeyPress::Right, "R"),
                (KeyPress::LeftMouse, "M1"),
                (KeyPress::RightMouse, "M2"),
            ],
            end_time: self.end_time,
            show_cursor: false,
            audio_prefix: String::new(),
            timing_bar_things: self.hit_windows
                .iter()
                .map(|(j, w)| (w.end, j.color))
                .collect(),

            sound_list: sound_list.into_iter().collect(),
        }
    }
}
