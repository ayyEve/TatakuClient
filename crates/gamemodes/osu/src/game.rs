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
        NoteType,
        BeatmapMeta,
        TimingPoint,
        map_difficulty,
    },
    gameplay,
    gameplay::{
        mods::*,
        Gamemode,
        judgments::*,
        GameplayEvent,
        TimingPointHelper,
        PlayfieldNonsense,
        GamemodeProperties,
        gameplay_manager::*,
    },
};
use input::gilrs::Axis;


const STACK_LENIENCY:u32 = 3;
pub const PREEMPT_MIN:f32 = 450.0;

pub struct OsuGame {
    // lists
    pub notes: Vec<Box<dyn OsuHitObject>>,
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

    /// scaling helper to help with scaling
    coords: Arc<OsuCoords>,
    /// needed for scaling recalc
    cs: f32,
    stack_leniency: f32,

    /// cached settings
    game_settings: Arc<OsuSettings>,

    auto_replay: AutoReplay,
    relax_play: RelaxPlay,

    /// list of note_indices which are new_combos
    #[cfg(feature="graphics")] new_combos: Vec<usize>,
    #[cfg(feature="graphics")] beatmap_combo_colors: Vec<Color>,

    use_controller_cursor: bool,
    end_time: f32,

    #[cfg(feature="graphics")] cursor: OsuCursor,
    #[cfg(feature="graphics")] smoke_emitter: Option<tataku_graphics::Emitter>,
    #[cfg(feature="graphics")] follow_point_image: Option<tataku_graphics::Image>,
    #[cfg(feature="graphics")] judgment_helper: JudgmentImages,

    metadata: Arc<BeatmapMeta>,
    mods: Arc<ModManager>,
    timing_points: Vec<TimingPoint>,

    new_playfield_pending: bool,
}
impl OsuGame {

    #[cfg(feature="graphics")]
    fn recalculate_playfield(&mut self, window_size: Vector2) {
        let new_scale = OsuCoords::new_with_settings(
            &self.game_settings,
            self.cs,
            window_size,
            self.mods.has_mod(HardRock)
        );

        self.new_playfield_pending = true;
        self.apply_playfield(Arc::new(new_scale));
    }

    #[cfg(feature="graphics")]
    fn apply_playfield(&mut self, playfield: Arc<OsuCoords>) {
        self.coords = playfield;
        self.cursor.note_radius = self.coords.circle_size.x / 2.0;

        // update playfield for notes
        for note in self.notes.iter_mut() {
            note.playfield_changed(self.coords.clone());
        }
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
        let mut extended_end_index = self.notes.len();

        let mut stack_base_index = self.notes.len();
        loop {
            for n in (stack_base_index + 1)..self.notes.len() {
                let obj = &self.notes[stack_base_index];
                if obj.note_type() == NoteType::Spinner { break }

                let obj_n = &self.notes[n];
                if obj_n.note_type() == NoteType::Spinner { break }

                let stack_threshhold = obj_n.get_preempt() * self.stack_leniency;

                if obj_n.time() - obj.time() > stack_threshhold {
                    // outside stack threshhold
                    break;
                }

                let obj_pos = obj.pos_at(obj.time());
                let obj_n_pos = obj.pos_at(obj.time());
                let obj_is_slider = obj.note_type() == NoteType::Slider;
                let obj_end_pos = obj.pos_at(obj.end_time(0.0));

                if obj_pos.distance(obj_n_pos) < STACK_LENIENCY as f32 || (obj_is_slider && obj_end_pos.distance(obj_n_pos) < STACK_LENIENCY as f32) {
                    stack_base_index = n;

                    // self.notes[n].set_stack_count(0)
                }
            }


            if stack_base_index > extended_end_index {
                extended_end_index = stack_base_index;
                if extended_end_index == self.notes.len() - 1 {break}
            }

            // check loop
            // if stack_base_index == 0 {
            //     stack_base_index -= 1
            // } else {
            //     break
            // }
        }



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
        coords: &Arc<OsuCoords>,
        judgment_helper: &JudgmentImages,
        settings: &OsuSettings,
        state: &mut GameplayUpdateShell<'_>
    ) {
        if hit_value.tex_name.is_empty() { return }

        let color = hit_value.color;
        let mut image = settings.use_skin_judgments.then_some(())
            .and_then(|_| judgment_helper.get_from_scorehit(hit_value));

        if let Some(image) = &mut image {
            let transform = graphics::Transform {
                pos,
                scale: Vector2::ONE * coords.cs,
                ..graphics::Transform::identity()
            };

            state.add_indicator(ImageJudgementIndicator::new(
                image.clone(),
                transform
            ));
        } else {
            let transform = graphics::Transform {
                pos,
                scale: Vector2::ONE * CIRCLE_RADIUS_BASE * coords.cs * (1.0 / 3.0),
                ..graphics::Transform::identity()
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
    pub fn get_ar(meta: &BeatmapMeta, mods: &ModManager) -> f32 {
        Self::scale_by_mods(meta.ar, 0.5, 1.4, mods).clamp(1.0, 11.0)
    }

    #[inline]
    pub fn get_od(meta: &BeatmapMeta, mods: &ModManager) -> f32 {
        Self::scale_by_mods(meta.od, 0.5, 1.4, mods).clamp(1.0, 10.0)
    }

    #[inline]
    pub fn get_cs(meta: &BeatmapMeta, mods: &ModManager) -> f32 {
        Self::scale_by_mods(meta.cs, 0.5, 1.3, mods).clamp(1.0, 10.0)
    }

    #[cfg(feature="graphics")]
    fn draw_follow_points(
        &mut self,
        time: f32,
        list: &mut engine::graphics::RenderableCollection,
    ) {
        if !self.game_settings.draw_follow_points { return; }
        if self.notes.is_empty() { return }

        let follow_dot_size = 3.0 * self.coords.scale;
        let follow_dot_distance = 20.0 * self.coords.scale;

        for i in 0..self.notes.len() - 1 {
            if self.new_combos.contains(&(i + 1)) { continue }

            let n1 = &self.notes[i];
            let n2 = &self.notes[i + 1];

            // skip if either note is a spinner
            if n1.note_type() == NoteType::Spinner { continue }
            if n2.note_type() == NoteType::Spinner { continue }

            let preempt = n2.get_preempt();
            let n1_time = n1.time();
            if time < n1_time - preempt { continue } //|| time > n2.end_time(0.0) {continue}
            let n2_time = n2.end_time(0.0);
            if time >= n2_time { continue }//|| time <= n1_time {continue}

            // setup follow points and the time they should exist at
            let n1_pos = n1.pos_at(n2_time);
            let n2_pos = n2.pos_at(n1_time);
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
                        ..graphics::Transform::identity()
                    };

                    list.push(i.with_transform(transform.matrix()));
                } else {
                    let transform = graphics::Transform {
                        pos: point,
                        scale: Vector2::ONE * follow_dot_size,
                        ..graphics::Transform::identity()
                    };

                    list.push(engine::graphics::Circle::new(
                        Color::WHITE.alpha(alpha),
                    ).with_transform(transform.matrix()));
                }
            }
        }
    }

    #[cfg(feature="graphics")]
    fn apply_combo_colors(&mut self, colors: &[Color]) {
        let mut combo_num = 0;
        let mut combo_change = 0;

        for note in self.notes.iter_mut() {
            if note.new_combo() { combo_num = 0 }

            // if new combo, increment new combo counter
            if combo_num == 0 {
                combo_change += 1;
            }

            // get color
            let color = colors[(combo_change - 1) % colors.len()];
            note.set_combo_color(color);

            // update combo number
            combo_num += 1;
        }
    }

    fn map_key(&self, key: &input::Key) -> Option<KeyPress> {
        if key == &self.game_settings.left_key {
            Some(KeyPress::Left)
        } else if key == &self.game_settings.right_key {
            Some(KeyPress::Right)
        } else if key == &self.game_settings.smoke_key {
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
        settings: &engine::Settings,
    ) -> tataku::Result<Self> {
        let metadata = map.get_beatmap_meta();
        let mods = Arc::default();
        let effective_window_size = super::diff_calc::WINDOW_SIZE;

        let game_settings = settings.gamemode_settings(crate::GAME_INFO).unwrap_or_default();
        // settings.osu_settings.clone();

        let cs = Self::get_cs(&metadata, &mods);
        let ar = Self::get_ar(&metadata, &mods);
        let od = Self::get_od(&metadata, &mods);
        let coords = Arc::new(OsuCoords::new_with_settings(&game_settings, cs, effective_window_size, mods.has_mod(HardRock)));

        let timing_points = TimingPointHelper::new(
            map.get_timing_points(),
            map.slider_velocity(),
        );

        let parent_dir = map.get_parent_dir().unwrap_or_default().to_string_lossy().to_string();
        let mut actions = actions::ActionQueue::new();

        #[cfg(feature="graphics")]
        let cursor = {
            let cursor = OsuCursor::new(
                coords.circle_size.x / 2.0,
                graphics::SkinSettings::default(),
                parent_dir,
                settings
            );

            cursor.init(&mut actions);
            cursor
        };

        let mut s = match map {
            Beatmap::Osu(beatmap) => {
                use engine::beatmaps::osu::*;
                let stack_leniency = beatmap.stack_leniency;
                let std_settings = Arc::new(game_settings);

                let get_hitsounds = |time, hitsound, hitsamples| {
                    let tp = timing_points.timing_point_at(time, true);
                    engine::gameplay::Hitsound::from_hitsamples(hitsound, hitsamples, true, tp)
                };

                let mut s = Self {
                    actions,
                    notes: Vec::new(),
                    mouse_pos: Vector2::ZERO,
                    window_mouse_pos: Vector2::ZERO,
                    hit_windows: Vec::new(),
                    miss_window: 0.0,

                    hold_count: 0,
                    end_time: 0.0,

                    move_playfield: None,
                    coords: coords.clone(),
                    cs,

                    use_controller_cursor: false,

                    game_settings: std_settings.clone(),
                    auto_replay: AutoReplay::default(),
                    relax_play: RelaxPlay::default(),

                    #[cfg(feature="graphics")] new_combos: Vec::new(),
                    stack_leniency,
                    // window_size,
                    #[cfg(feature="graphics")] follow_point_image: None,
                    #[cfg(feature="graphics")] judgment_helper: JudgmentImages::new(OsuHitJudgments::variants().to_vec()),
                    metadata,
                    mods,
                    timing_points: map.get_timing_points(),
                    #[cfg(feature="graphics")] smoke_emitter: None,
                    #[cfg(feature="graphics")] cursor,

                    #[cfg(feature="graphics")] beatmap_combo_colors: beatmap.combo_colors.clone(),
                    new_playfield_pending: false,
                };


                enum Thing<'a> {
                    Note(&'a NoteDef),
                    Slider(&'a SliderDef, Option<Box<Curve>>),
                    Spinner(&'a SpinnerDef),
                }
                impl Thing<'_> {
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
                    beatmap.notes.iter().map(Thing::Note)
                    .chain(beatmap.sliders.iter().map(|s| Thing::Slider(s, if s.curve_points.is_empty() || s.length == 0.0 { None } else { Some(Box::new(get_curve(s, map, &timing_points))) } )))
                    .chain(beatmap.spinners.iter().map(Thing::Spinner))
                    .collect::<Vec<_>>()
                ;

                // sort all the notes
                all_items.sort_by(|a, b| a.cmp(b));

                // add notes
                let mut combo_num = 0;
                for (counter, i) in all_items.into_iter().enumerate() {

                    // update end time
                    s.end_time = s.end_time.max(i.end_time());

                    // reset combo if hitobject says so
                    if i.new_combo() { combo_num = 0 }

                    // if new combo, add counter to combo
                    #[cfg(feature="graphics")]
                    if combo_num == 0 {
                        s.new_combos.push(counter);
                    }

                    // update combo number
                    combo_num += 1;

                    // add the hitobject
                    match i {
                        Thing::Note(note) => {
                            s.notes.push(Box::new(OsuNote::new(
                                note.clone(),
                                ar,
                                combo_num as u16,
                                coords.clone(),
                                std_settings.clone(),
                                get_hitsounds(note.time, note.hitsound, note.hitsamples.clone())
                            )));
                        }

                        Thing::Slider(slider, None) => {
                            let note = NoteDef {
                                pos: slider.pos,
                                time: slider.time,
                                hitsound: slider.hitsound,
                                hitsamples: slider.hitsamples.clone(),
                                new_combo: slider.new_combo,
                                color_skip: slider.color_skip,
                            };

                            let hitsounds = get_hitsounds(note.time, note.hitsound, note.hitsamples.clone());
                            s.notes.push(Box::new(OsuNote::new(
                                note,
                                ar,
                                combo_num as u16,
                                coords.clone(),
                                std_settings.clone(),
                                hitsounds,
                            )));
                        }

                        Thing::Slider(slider, Some(curve)) => {
                            s.notes.push(Box::new(OsuSlider::new(
                                slider.clone(),
                                *curve,
                                ar,
                                combo_num as u16,
                                coords.clone(),
                                std_settings.clone(),
                                get_hitsounds,
                                timing_points.slider_velocity_at(slider.time)
                            )));
                        }


                        Thing::Spinner(spinner) => {
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

                            s.notes.push(Box::new(OsuSpinner::new(
                                spinner,
                                coords.clone(),
                                spins_required
                            )));
                        }
                    }
                }

                s
            }

            _ => return Err(errors::beatmap::BeatmapError::UnsupportedMode.into()),
        };

        // wait an extra sec
        s.end_time += 1000.0;

        s.setup_hitwindows();

        Ok(s)
    }

    fn handle_replay_frame(
        &mut self,
        frame: ReplayFrame,
        state: &mut GameplayUpdateShell
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

                let mut hittable_notes = Vec::new();
                let mut visible_notes = Vec::new();

                for note in self.notes.iter_mut() {
                    note.press(frame.time);
                    // check if note is in hitwindow, has not yet been hit, and is not a spinner
                    let note_time = note.time();
                    let in_hitwindow = (frame.time - note_time).abs() <= self.miss_window;
                    let is_visible = frame.time > note_time - note.get_preempt() && frame.time < note_time;

                    if (in_hitwindow || is_visible) && !note.was_hit() && note.note_type() != NoteType::Spinner {
                        if in_hitwindow {
                            hittable_notes.push(note);
                        } else {
                            visible_notes.push(note);
                        }
                    }
                }

                if hittable_notes.is_empty() && visible_notes.is_empty() { return } // no notes to check
                hittable_notes.sort_by(|a, b| a.time().partial_cmp(&b.time()).unwrap());

                for note in hittable_notes {
                    if !note.check_distance(self.mouse_pos) { continue }
                    let note_time = note.time();

                    if let Some(judge) = state.check_judgment(&self.hit_windows, frame.time, note_time) {
                        note.set_judgment(judge);

                        if judge == &OsuHitJudgments::X300 && !self.game_settings.show_300s {
                            // dont show the judgment
                        } else {
                            #[cfg(feature="graphics")]
                            Self::add_judgement_indicator(
                                note.point_draw_pos(frame.time),
                                judge,
                                &self.coords,
                                &self.judgment_helper,
                                &self.game_settings,
                                state
                            );
                        }

                        if judge == &OsuHitJudgments::Miss {
                            // tell the note it was missed
                            note.miss();
                        } else {
                            // tell the note it was hit
                            note.hit(frame.time);

                            // play the sound
                            #[cfg(feature="gameplay")]
                            state.play_hitsounds(&note.get_hitsound(), false);
                        }

                        return;
                    }
                }

                // no notes were hit, check visible notes
                visible_notes.sort_by(|a, b| a.time().partial_cmp(&b.time()).unwrap());
                for note in visible_notes {
                    if !note.check_distance(self.mouse_pos) { continue }

                    #[cfg(feature="graphics")]
                    note.shake(frame.time);
                    break
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
                        if let Some(i) = self.smoke_emitter.as_mut() { i.should_emit = false; }
                        return;
                    }
                    _ => {}
                }

                for note in self.notes.iter_mut() {
                    // if this is the last key to be released
                    if self.hold_count == 0 {
                        note.release(frame.time);
                    }
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
                #[cfg(feature="graphics")]
                self.cursor.cursor_pos(pos);

                for note in self.notes.iter_mut() {
                    note.mouse_move(pos);
                }
            }
            _ => {}
        }
    }

    fn handle_gameplay_event(&mut self, event: GameplayEvent) {
        match event {
            GameplayEvent::SetBounds { bounds, full_window } => {

                #[cfg(feature="graphics")]
                if full_window {
                    // self.window_size = window_size;
                    self.recalculate_playfield(bounds.size);
                } else {
                    self.apply_playfield(Arc::new(OsuCoords::new_offset_scale(
                        self.cs,
                        bounds.size,
                        bounds.pos,
                        0.80,
                        self.mods.has_mod(HardRock)
                    )));
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

                let mut set_ar = None;
                let mut set_easing = None;

                if has_easy_or_hr || had_easy_or_hr != has_easy_or_hr {
                    self.cs = Self::get_cs(&self.metadata, &self.mods);
                    let ar = Self::get_ar(&self.metadata, &self.mods);

                    #[cfg(feature="graphics")]
                    self.recalculate_playfield(self.coords.window_size);
                    self.setup_hitwindows();

                    set_ar = Some(ar);
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
                        let timing_points = self.timing_points
                            .iter()
                            .filter(|t| !t.is_inherited())
                            .copied()
                            .collect::<Vec<_>>();
                        let mut index = 0;
                        // info!("tp: {} -> {}", timing_points[index].time, timing_points[index].beat_length);

                        for note in self.notes.iter_mut() {
                            // check next timing point
                            if let Some(next) = timing_points.get(index + 1)
                            && next.time <= note.time() {
                                index += 1;
                                // info!("tp: {} -> {}", timing_points[index].time, timing_points[index].beat_length);
                            }

                            // get the beat length of the current timing point
                            let beat_length = timing_points[index].beat_length;

                            // normalize the note time to "align" with the control point time offset
                            let normalized_time = note.time() - timing_points[index].time; // beat lengths with decimal points
                            let m = beat_length - (normalized_time % beat_length); // beat lengths without a decimal point
                            let m2 = normalized_time % beat_length;
                            // info!("{normalized_time}, {m}, {m2}");

                            // if this note lands on a beat, or within 10ms of a beat, make it ~funky~
                            if m < 10.0 || m2 < 10.0 {
                                note.set_approach_easing(Easing::EaseOutExponential);
                            } else {
                                note.set_approach_easing(Easing::Linear);
                            }

                        }

                        set_easing = None;
                    } else {
                        set_easing = Some(Easing::Linear);
                    }

                }

                if set_ar.is_some() || set_easing.is_some() {
                    for note in self.notes.iter_mut() {
                        #[cfg(feature="graphics")]
                        if let Some(easing) = set_easing {
                            note.set_approach_easing(easing);
                        }
                        if let Some(ar) = set_ar {
                            note.set_ar(ar);
                        }
                    }
                }

            }

            GameplayEvent::BeatHappened { pulse_length } => {
                for i in self.notes.iter_mut() {
                    i.beat_happened(pulse_length);
                }
            }
            GameplayEvent::KiaiChanged { enabled } => {
                for i in self.notes.iter_mut() {
                    i.kiai_changed(enabled);
                }
            }

            _ => {}
        }
    }

    fn update(
        &mut self,
        state: &mut GameplayUpdateShell,
    ) {
        state.action_queue.extend(self.actions.take());

        if self.new_playfield_pending {
            self.new_playfield_pending = false;
            state.add_action(gameplay::Action::PlayfieldChanged);
        }

        // disable the cursor particle emitter if this is a menu game
        // the emitter nukes perf so its best to keep it off unless needed
        #[cfg(feature="graphics")]
        if state.gameplay_type.is_preview() && self.cursor.emitter_enabled {
            self.cursor.emitter_enabled = false;
        }
        #[cfg(feature="graphics")]
        self.cursor.update();

        let has_autoplay = state.mods.has_autoplay();
        let has_relax = state.mods.has_mod(Relax);

        // do autoplay things
        if has_autoplay {
            let mut pending_frames = Vec::new();
            self.auto_replay.update(
                state.time,
                &self.notes,
                &self.coords,
                &mut pending_frames
            );

            // // handle presses and mouse movements now, and releases later
            for action in pending_frames {
                state.add_replay_action(action);
            }
        }

        if has_relax {
            self.relax_play.update(state.time);
        }

        // update emitter
        #[cfg(feature="graphics")]
        if let Some(e) = self.smoke_emitter.as_mut() { e.update(state.time) }

        // if the map is over, say it is
        if state.time >= self.end_time {
            if !state.complete() {
                state.add_action(gameplay::Action::MapComplete);
            }
            return;
        }

        // update notes
        for (note_index, note) in self.notes.iter_mut().enumerate() {
            note.update(state.time);
            let end_time = note.end_time(self.miss_window);

            // play queued sounds
            #[cfg(feature="gameplay")]
            for hitsound in note.get_sound_queue() {
                state.play_hitsounds(&hitsound, false);
            }

            for (judgment, pos) in note.pending_combo() {
                state.add_judgment(judgment);
                #[cfg(feature="graphics")]
                Self::add_judgement_indicator(
                    pos,
                    &judgment,
                    &self.coords,
                    &self.judgment_helper,
                    &self.game_settings,
                    state
                );
            }

            // check relax stuff
            if has_relax && !has_autoplay {
                self.relax_play.check_note(
                    self.mouse_pos,
                    end_time,
                    note,
                    note_index,
                    state,
                );
            }

            // check if note was missed

            // if the time is leading in, we dont want to check if any notes have been missed
            if state.time < 0.0 { continue }

            // check if note is in hitwindow
            if state.time >= end_time && !note.was_hit() {

                // check if we missed the current note
                match note.note_type() {
                    NoteType::Note => {
                        let j = OsuHitJudgments::Miss;
                        state.add_judgment(j);

                        #[cfg(feature="graphics")]
                        Self::add_judgement_indicator(
                            note.point_draw_pos(state.time),
                            &j,
                            &self.coords,
                            &self.judgment_helper,
                            &self.game_settings,
                            state
                        );
                    }
                    NoteType::Slider => {
                        // check slider release points
                        // internally checks distance
                        let judge = note.check_release_points(state.time);
                        state.add_judgment(judge);

                        if judge != OsuHitJudgments::X300 || self.game_settings.show_300s {
                            #[cfg(feature="graphics")]
                            Self::add_judgement_indicator(
                                note.point_draw_pos(state.time),
                                &judge,
                                &self.coords,
                                &self.judgment_helper,
                                &self.game_settings,
                                state
                            );
                        }

                        if judge != OsuHitJudgments::Miss {
                            // tell the note it was hit
                            note.hit(state.time);

                            // play the sound
                            #[cfg(feature="gameplay")]
                            state.play_hitsounds(&note.get_hitsound(), false);
                        }
                    }

                    NoteType::Spinner => {
                        let j = OsuHitJudgments::SpinnerMiss;
                        state.add_judgment(j);
                        #[cfg(feature="graphics")]
                        Self::add_judgement_indicator(
                            note.point_draw_pos(state.time),
                            &j,
                            &self.coords,
                            &self.judgment_helper,
                            &self.game_settings,
                            state
                        );
                    }

                    _ => {},
                }

                // force the note to be misssed
                note.miss();
            }
        }

        // handle note releases
        // required because autoplay frames are checked after the frame is processed
        // so if the key is released on the same frame its checked, it will count as not held
        // which makes sense, but we dont want that
        if has_autoplay {
            for action in self.auto_replay.get_release_queue() {
                state.add_replay_action(action);
            }
        }

    }

    #[cfg(feature="graphics")]
    fn draw(
        &mut self,
        state: GameplayDrawShell,
        list: &mut tataku_graphics::RenderableCollection
    ) {
        use engine::graphics;
        let window_size = state.window_size;
        // draw the playfield
        if !state.gameplay_type.is_preview() {
            let alpha = self.game_settings.playfield_alpha;

            let mut playfield_border = state.current_timing_point.kiai
                .then_some(Border::new(Color::YELLOW.alpha(alpha), 2.0));

            let playfield = self.coords.playfield_with_padding;

            if self.move_playfield.is_some() {
                let line_size = self.game_settings.playfield_movelines_thickness;
                // draw x and y center lines
                let px_line = graphics::Line::new(
                    Vector2::with_x(playfield.size.x),
                    line_size,
                    Color::WHITE
                ).with_transform(tataku::Matrix::identity()
                    .trans(playfield.pos + Vector2::new(0.0, playfield.size.y/2.0))
                );
                let py_line = graphics::Line::new(
                    Vector2::with_y(playfield.size.y),
                    line_size,
                    Color::WHITE
                ).with_transform(tataku::Matrix::identity()
                    .trans(playfield.pos + Vector2::new(playfield.size.x/2.0, 0.0))
                );

                let wx_line = graphics::Line::new(
                    Vector2::with_x(window_size.x),
                    line_size,
                    Color::WHITE
                ).with_transform(tataku::Matrix::identity()
                    .trans(Vector2::new(0.0, window_size.y/2.0))
                );
                let wy_line = graphics::Line::new(
                    Vector2::with_y(window_size.y),
                    line_size,
                    Color::WHITE
                ).with_transform(tataku::Matrix::identity()
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
                Color::BLACK.alpha(alpha),
            ).border_maybe(playfield_border)
            .with_transform(tataku::Matrix::identity()
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
        self.draw_follow_points(state.time, list);

        // draw notes
        let mut spinners = Vec::new();
        for note in self.notes.iter_mut().rev() {
            match note.note_type() {
                NoteType::Spinner => spinners.push(note),
                _ => note.draw(state.time, list),
            }
        }

        // draw flashlight
        if has_flashlight {
            // list.pop_scissor();

            let radius = match state.score.combo {
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

        // spinners should be drawn last since they should be on top of everything
        // (we dont want notes or sliders drawn on top of the spinners)
        for i in spinners {
            i.draw(state.time, list);
        }

        // need to draw the smoke particles on top of everything
        if let Some(e) = self.smoke_emitter.as_ref() {
            e.draw(list);
        }

        // draw the cursor on top of smoke tho
        self.cursor.draw_above(list);
    }


    fn all_notes(&self) -> Vec<&dyn engine::gameplay::HitObject> {
        self.notes.iter()
            .map(|i| &**i as &dyn engine::gameplay::HitObject)
            .collect::<Vec<&dyn engine::gameplay::HitObject>>()
    }

    fn reset(&mut self, _beatmap: &Beatmap) {
        // let ar = scale_by_mods(self.metadata.ar, 0.5, 1.4, &self.mods).clamp(1.0, 11.0);

        // reset notes
        let hwm = self.miss_window;
        for note in self.notes.iter_mut() {
            note.reset();
            note.set_hitwindow_miss(hwm);
            // note.set_ar(ar)
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
        if self.notes.is_empty() { return None }

        let time = self.notes[0].time() - self.notes[0].get_preempt();
        if time < game_time || time < 0.0 { return None }

        Some(time)
    }

    fn time_jump(
        &mut self,
        new_time: f32,
        state: &mut GameplayUpdateShell
    ) {
        for n in self.notes.iter_mut() {
            n.time_jump(new_time);
        }

        let mut pending_frames = Vec::new();
        self.auto_replay.time_skip(
            new_time,
            &self.notes,
            &self.coords,
            &mut pending_frames
        );

        for i in pending_frames {
            state.add_replay_action(i);
        }
    }

    fn force_update_settings(&mut self, settings: &engine::Settings) {
        let settings = settings.gamemode_settings::<OsuSettings>(crate::GAME_INFO).unwrap_or_default();
        // let settings = settings.osu_settings.clone();
        let settings = Arc::new(settings);

        if self.game_settings == settings { return }

        self.game_settings = settings.clone();
        for n in self.notes.iter_mut() {
            n.set_settings(settings.clone());
        }
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

        let source = if self.game_settings.beatmap_skin {
            TextureSource::Beatmap(beatmap_path.to_owned())
        } else {
            TextureSource::Skin
        };

        self.cursor.reload_skin(skin_manager);
        self.judgment_helper.reload_skin(skin_manager);
        self.follow_point_image = skin_manager.get_texture(
            Path::new("followpoint"),
            &source,
            SkinUsage::Gamemode,
            false
        );

        let combo_colors = if self.game_settings.use_beatmap_combo_colors && !self.beatmap_combo_colors.is_empty() {
            self.beatmap_combo_colors.clone()
        } else if !skin_manager.skin().combo_colors.is_empty() {
            skin_manager.skin().combo_colors.clone()
        } else {
            self.game_settings.combo_colors.iter().map(Color::from_hex).collect()
        };

        self.apply_combo_colors(&combo_colors);

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
                .spawn_delay(10.0)
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
                let key = press.as_key()?;

                // playfield adjustment
                if key == Key::LControl {
                    let old = self.game_settings.get_playfield();
                    self.move_playfield = Some((old.1, self.window_mouse_pos));
                    return None;
                }

                let key = self.map_key(&key)?;

                // if relax is enabled, and the user doesn't want manual input, return
                if self.mods.has_mod(Relax) {
                    if !self.game_settings.manual_input_with_relax { return None; }
                    self.relax_play.key_pressed(key);
                }

                Some(ReplayAction::Press(key))
            }

            InputType::KeyRelease(release) => {
                let key = release.as_key()?;

                // playfield adjustment
                if key == Key::LControl {
                    self.move_playfield = None;
                    return None;
                }

                let key = self.map_key(&key)?;

                // if relax is enabled, and the user doesn't want manual input, return
                if self.mods.has_mod(Relax) {
                    if !self.game_settings.manual_input_with_relax { return None; }
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

                    let mut settings = (*self.game_settings).clone();
                    let mut change = original + (pos - mouse_start);

                    // check playfield snapping
                    // TODO: can this be simplified?
                    let playfield_size = self.coords.playfield_with_padding.size;

                    // what the offset should be if playfield is centered
                    let center_offset = (self.coords.window_size - FIELD_SIZE * self.coords.scale) / 2.0 - (self.coords.window_size - playfield_size) / 2.0;

                    let snap_threshold = settings.playfield_snap;
                    if (center_offset.x - change.x).abs() < snap_threshold {
                        change.x = center_offset.x;
                    }
                    if (center_offset.y - change.y).abs() < snap_threshold {
                        change.y = center_offset.y;
                    }

                    settings.playfield_x_offset = change.x;
                    settings.playfield_y_offset = change.y;


                    let settings2 = settings.clone();
                    self.actions.push(actions::game::GameAction::UpdateSettings(Arc::new(
                        move |settings|
                        settings.update_gamemode_settings(
                            GAME_INFO,
                            settings2.clone()
                        )
                    )).into());

                    self.game_settings = Arc::new(settings);
                    self.recalculate_playfield(self.coords.window_size);
                    return None;
                }


                // convert window pos to playfield pos
                let pos = self.coords.to_osu(pos);
                Some(ReplayAction::MousePos(pos.x, pos.y))
            }

            InputType::MousePress(btn) => {
                // if the user has mouse input disabled, return
                if self.game_settings.ignore_mouse_buttons { return None }

                let button = self.map_btn(&btn)?;

                // if relax is enabled, and the user doesn't want manual input, return
                if self.mods.has_mod(Relax) {
                    if !self.game_settings.manual_input_with_relax { return None; }
                    self.relax_play.key_pressed(button);
                }

                Some(ReplayAction::Press(button))
            }

            InputType::MouseRelease(btn) => {
                // if the user has mouse input disabled, return
                if self.game_settings.ignore_mouse_buttons { return None }

                let button = self.map_btn(&btn)?;

                // if relax is enabled, and the user doesn't want manual input, return
                if self.mods.has_mod(Relax) {
                    if !self.game_settings.manual_input_with_relax { return None; }
                    self.relax_play.key_released(button);
                }

                Some(ReplayAction::Release(button))
            }

            InputType::MouseScroll { raw: delta, .. } => {
                if self.move_playfield.is_some() {
                    let delta = delta / 40.0;
                    let mut a = (*self.game_settings).clone();
                    a.playfield_scale += delta.y;
                    self.game_settings = Arc::new(a.clone());

                    self.actions.push(actions::game::GameAction::UpdateSettings(Arc::new(
                        move |settings|
                        settings.update_gamemode_settings(
                            GAME_INFO,
                            a.clone()
                        )
                    )).into());

                    self.recalculate_playfield(self.coords.window_size);
                }

                None
            }

            InputType::ControllerPress(btn, _id, _name) => {
                // if relax is enabled, and the user doesn't want manual input, return
                if self.mods.has_mod(Relax) && !self.game_settings.manual_input_with_relax { return None; }

                match btn {
                    GamepadButton::LeftTrigger => Some(ReplayAction::Press(KeyPress::Left)),
                    GamepadButton::RightTrigger => Some(ReplayAction::Press(KeyPress::Right)),
                    _ => None
                }
            }

            InputType::ControllerRelease(btn, _id, _name) => {
                // if relax is enabled, and the user doesn't want manual input, return
                if self.mods.has_mod(Relax) && !self.game_settings.manual_input_with_relax { return None; }

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
                let coords = self.coords.clone();
                let playfield = coords.playfield_with_padding;

                let normalized = (value + 1.0) / 2.0;
                new_pos.x = playfield.pos.x + f32::lerp(0.0, playfield.size.x, normalized);

                let new_pos = coords.to_osu(new_pos);
                Some(ReplayAction::MousePos(new_pos.x, new_pos.y))
            }

            InputType::ControllerAxis(Axis::LeftStickY, value, _id, _name) => {
                // y is upside down in gilrs i guess?

                if !self.use_controller_cursor {
                    // info!("switched to controller input");
                    self.use_controller_cursor = true;
                }


                let mut new_pos = self.mouse_pos;
                let coords = self.coords.clone();
                let playfield = coords.playfield_with_padding;

                let normalized = (value + 1.0) / 2.0;
                new_pos.y = playfield.pos.y + f32::lerp(playfield.size.y, 0.0, normalized);

                let new_pos = coords.to_osu(new_pos);
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
    fn properties(&self, _timing_points: &TimingPointHelper) -> GamemodeProperties {
        let mut sound_list = HashMap::new();
        #[cfg(feature="gameplay")]
        for note in self.notes.iter() {
            for hitsound in note.get_all_hitsounds().iter().flatten() {
                sound_list.insert(
                    hitsound.get_id(),
                    hitsound.load_data(None::<String>)
                );
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
