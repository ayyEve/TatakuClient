use crate::prelude::*;


/// how much time should pass at beatmap start before audio begins playing (and the map "starts")
pub const LEAD_IN_TIME:f32 = 1000.0;
/// how long should center text be drawn for?
const CENTER_TEXT_DRAW_TIME:f32 = 2_000.0;
/// how tall is the duration bar
pub const DURATION_HEIGHT:f32 = 35.0;

/// ms between spectator score sync packets
const SPECTATOR_SCORE_SYNC_INTERVAL:f32 = 1000.0;

const HIT_DIFF_FACTOR:f32 = 1.0;


// bc im lazy
macro_rules! add_timing {
    ($self:ident, $time:expr, $note_time:expr) => {{
        let diff = ($time - $note_time) / HIT_DIFF_FACTOR;
        $self.add_stat(HitVarianceStat, diff);
        // $self.score.hit_timings.push(diff);
        $self.hitbar_timings.push(($time, diff));
    }}
}

pub struct IngameManager {
    pub beatmap: Beatmap,
    pub metadata: Arc<BeatmapMeta>,
    pub gamemode: Box<dyn GameMode>,
    pub current_mods: Arc<ModManager>,
    pub beatmap_preferences: BeatmapPreferences,

    pub score: IngameScore,
    pub replay: Replay,
    pub score_multiplier: f32,

    pub health: HealthHelper,
    pub judgment_type: Box<dyn HitJudgments>,
    pub key_counter: KeyCounter,
    ui_elements: Vec<UIElement>,

    animation: Box<dyn BeatmapAnimation>,

    pub score_list: Vec<IngameScore>,
    score_loader: Option<Arc<AsyncRwLock<ScoreLoaderHelper>>>,

    pub started: bool,
    pub completed: bool,
    pub replaying: bool,
    pub failed: bool,
    pub failed_time: f32,

    /// has something about the ui been changed? 
    /// this will make the play unrankable and should not be saved
    pub ui_changed: bool,

    /// should the manager be paused?
    pub should_pause: bool,
    /// is a pause pending?
    /// used for breaks. if the user tabs out during a break, a pause is pending, but we shouldnt pause until the break is over (or almost over i guess)
    pause_pending: bool,
    pause_start: Option<i64>,

    /// is this playing in the background of the main menu?
    pub menu_background: bool,
    pub multiplayer: bool,

    pub end_time: f32,
    pub lead_in_time: f32,
    pub lead_in_timer: Instant,

    // pub timing_points: Vec<TimingPoint>,
    // pub timing_point_index: usize,
    // next_beat: f32,
    pub timing_points: TimingPointHelper,

    pub song: Arc<dyn AudioInstance>,
    pub hitsound_manager: HitsoundManager,

    /// center text helper (ie, for offset and global offset)
    pub center_text_helper: CenteredTextHelper,

    /// (map.time, note.time - hit.time)
    pub hitbar_timings: Vec<(f32, f32)>,

    /// list of judgement indicators to draw
    pub judgement_indicators: Vec<Box<dyn JudgementIndicator>>,

    /// if in replay mode, what replay frame are we at?
    replay_frame: u64,

    pub common_game_settings: Arc<CommonGameplaySettings>,
    settings: SettingsHelper,
    window_size: Arc<WindowSize>,
    fit_to_bounds: Option<Bounds>,

    // spectator info
    pub spectator_info: IngameSpectatorInfo,


    /// what should the game do on start?
    /// mainly a helper for spectator
    pub on_start: Box<dyn FnOnce(&mut Self) + Send + Sync>,

    pub events: Vec<InGameEvent>,
    ui_editor: Option<GameUIEditorDialog>,

    pending_time_jump: Option<f32>,
    skin_helper: CurrentSkinHelper,

    restart_key_hold_start: Option<Instant>,

    map_diff: f32,

    // used for discord rich presence
    pub start_time: i64,
}

impl IngameManager {
    pub async fn new(beatmap: Beatmap, mut gamemode: Box<dyn GameMode>) -> Self {
        let playmode = gamemode.playmode();
        let metadata = beatmap.get_beatmap_meta();

        let settings = SettingsHelper::new();
        let beatmap_preferences = Database::get_beatmap_prefs(&metadata.beatmap_hash).await;

        let timing_points = beatmap.get_timing_points();


        let mut current_mods = ModManager::get_cloned();
        if current_mods.get_speed() == 0.0 { current_mods.set_speed(1.0); }
        let current_mods = Arc::new(current_mods);

        let common_game_settings = Arc::new(settings.common_game_settings.clone());

        let mut score =  Score::new(beatmap.hash().clone(), settings.username.clone(), playmode.clone());
        score.speed = current_mods.get_speed();


        let health = HealthHelper::new();
        let score_loader = Some(SCORE_HELPER.read().await.get_scores(&metadata.beatmap_hash, &playmode).await);
        let key_counter = KeyCounter::new(gamemode.get_possible_keys().into_iter().map(|a| (a.0, a.1.to_owned())).collect());

        let song = AudioManager::get_song().await.unwrap_or(AudioManager::empty_stream()); // temp until we get the audio file path

        let center_text_helper = CenteredTextHelper::new(CENTER_TEXT_DRAW_TIME).await;

        // hardcode for now
        let audio_playmode_prefix = match &*playmode {
            "taiko" => "taiko".to_owned(),
            "mania" => "mania".to_owned(),
            // "taiko" => "taiko".to_owned(),

            _ => String::new(),
        };

        let mut hitsound_manager = HitsoundManager::new(audio_playmode_prefix);
        hitsound_manager.init(&metadata).await;

        let events = beatmap.get_events();
        // println!("loaded events {events:?}");
        let gamemode_info = get_gamemode_info(&score.playmode).unwrap();

        #[cfg(feature="storyboards")]
        let animation = beatmap.get_animation().await.unwrap_or_else(||Box::new(EmptyAnimation));
        #[cfg(not(feature="storyboards"))]
        let animation = Box::new(EmptyAnimation);

        // make sure the game has loaded its skin stuff
        gamemode.reload_skin().await;
        // make sure the gamemode has the correct mods applied
        gamemode.apply_mods(current_mods.clone()).await;

        Self {
            metadata,
            timing_points: TimingPointHelper::new(timing_points),
            // hitsound_cache,
            current_mods,
            health,
            key_counter,

            lead_in_timer: Instant::now(),
            judgment_type: gamemode_info.get_judgments(),
            score: IngameScore::new(score, true, false),

            replay: Replay::new(),
            beatmap,
            animation,

            hitsound_manager,
            song,

            lead_in_time: LEAD_IN_TIME,
            end_time: gamemode.end_time(),

            center_text_helper,
            beatmap_preferences,

            common_game_settings,
            skin_helper: CurrentSkinHelper::new(),

            gamemode,
            score_list: Vec::new(),
            score_loader,
            settings,
            window_size: WindowSize::get(),
            start_time: chrono::Utc::now().timestamp(),

            events,
            // initialize defaults for anything else not specified
            ..Self::default()
        }
    }

    async fn init_ui(&mut self) {
        if self.ui_editor.is_some() { return }
        
        let playmode = self.gamemode.playmode();
        let get_name = |name| {
            format!("{playmode}_{name}")
        };

        // score
        self.ui_elements.push(UIElement::new(
            &get_name("score"),
            Vector2::new(self.window_size.x, 0.0),
            ScoreElement::new().await
        ).await);

        // Acc
        self.ui_elements.push(UIElement::new(
            &get_name("acc"),
            Vector2::new(self.window_size.x, 40.0),
            AccuracyElement::new().await
        ).await);

        // Performance
        // TODO: calc diff before starting somehow?
        self.ui_elements.push(UIElement::new(
            &get_name("perf"),
            Vector2::new(self.window_size.x, 80.0),
            PerformanceElement::new().await
        ).await);

        // Healthbar
        self.ui_elements.push(UIElement::new(
            &get_name("healthbar"),
            Vector2::ZERO,
            HealthBarElement::new(self.common_game_settings.clone()).await
        ).await);

        // Duration Bar
        self.ui_elements.push(UIElement::new(
            &get_name("durationbar"),
            Vector2::new(0.0, self.window_size.y),
            DurationBarElement::new(self.common_game_settings.clone())
        ).await);

        // Judgement Bar
        self.ui_elements.push(UIElement::new(
            &get_name("judgementbar"),
            Vector2::new(self.window_size.x/2.0, self.window_size.y),
            JudgementBarElement::new(self.gamemode.timing_bar_things())
        ).await);

        // Key Counter
        self.ui_elements.push(UIElement::new(
            &get_name("key_counter"),
            Vector2::new(self.window_size.x, self.window_size.y/2.0),
            KeyCounterElement::new().await
        ).await);

        // Spectators
        self.ui_elements.push(UIElement::new(
            &get_name("spectators"),
            Vector2::new(0.0, self.window_size.y/3.0),
            SpectatorsElement::new()
        ).await);

        // judgement counter
        self.ui_elements.push(UIElement::new(
            &get_name("judgement_counter"),
            Vector2::new(self.window_size.x, self.window_size.y * (2.0/3.0)),
            JudgementCounterElement::new().await
        ).await);


        
        // elapsed timer
        self.ui_elements.push(UIElement::new(
            &get_name("elapsed_timer"),
            Vector2::new(30.0, self.window_size.y - 150.0),
            ElapsedElement::new().await
        ).await);

        // remaining timer
        self.ui_elements.push(UIElement::new(
            &get_name("remaining_timer"),
            Vector2::new(self.window_size.x - 300.0, self.window_size.y - 150.0),
            RemainingElement::new().await
        ).await);



        // anything in the gamemode itself
        self.gamemode.get_ui_elements(self.window_size.0, &mut self.ui_elements).await;
    }

    pub async fn apply_mods(&mut self, mut mods: ModManager) {
        if self.menu_background {
            mods.add_mod(Autoplay.name());
        }

        self.current_mods = Arc::new(mods);
        self.gamemode.apply_mods(self.current_mods.clone()).await;
    }

    pub async fn update(&mut self) {
        // update settings
        self.settings.update();
        if self.skin_helper.update() {
            self.reload_skin().await;
        }
        
        // make sure we jump to the time we're supposed to be at
        if let Some(time) = self.pending_time_jump {
            self.gamemode.time_jump(time).await;
            self.pending_time_jump = None;
        }

        // check map restart
        if let Some(press_time) = self.restart_key_hold_start {
            if press_time.as_millis() >= self.common_game_settings.map_restart_delay {
                self.reset().await;
                return;
            }
        }

        // check pause
        if self.pause_pending && !self.in_break() {
            info!("pausing");
            self.pause();
            self.pause_pending = false;
            self.should_pause = true;
        }
        // // i'm not sure whats happening here?
        // if self.should_pause && self.in_break() {
        //     info!("pausing");
        //     self.pause();
        //     self.should_pause = false;
        // }


        // update ui elements
        if !self.menu_background {
            let mut ui_elements = std::mem::take(&mut self.ui_elements);
            ui_elements.iter_mut().for_each(|ui|ui.update(self));
            self.ui_elements = ui_elements;
        }

        // update ui editor
        let mut ui_editor = std::mem::take(&mut self.ui_editor);
        let mut should_close = false;
        if let Some(ui_editor) = &mut ui_editor {
            ui_editor.update(&mut ()).await;
            ui_editor.update_elements(self);

            if ui_editor.should_close() {
                self.ui_elements = std::mem::take(&mut ui_editor.elements);
                should_close = true
            }
        }
        if !should_close {
            self.ui_editor = ui_editor;
        }
        

        // check lead-in time
        if self.lead_in_time > 0.0 {
            let elapsed = self.lead_in_timer.elapsed().as_micros() as f32 / 1000.0;
            self.lead_in_timer = Instant::now();
            self.lead_in_time -= elapsed * self.game_speed();

            if self.lead_in_time <= 0.0 {
                self.song.set_position(-self.lead_in_time);
                self.song.set_volume(self.settings.get_music_vol());
                self.song.set_rate(self.game_speed());
                self.song.play(true);
                
                self.lead_in_time = 0.0;
            }
        }
        let time = self.time();

        let tp_updates = self.timing_points.update(time);


        // check if scores have been loaded
        if let Some(loader) = self.score_loader.clone() {
            let loader = loader.read().await;
            if loader.done {
                self.score_list = loader.scores.iter().map(|s| { let mut s = s.clone(); s.is_previous = s.username == self.score.username; s }).collect();
                self.score_loader = None;
            }
        }

        
        let mut gamemode = std::mem::take(&mut self.gamemode);
        for tp_update in tp_updates {
            match tp_update {
                TimingPointUpdate::BeatHappened(pulse_length) => gamemode.beat_happened(pulse_length).await,
                TimingPointUpdate::KiaiChanged(kiai) => gamemode.kiai_changed(kiai).await,
            }
        }

        let mut pending_frames = Vec::new();

        // read inputs from replay if replaying
        if self.replaying && !self.current_mods.has_autoplay() {

            // read any frames that need to be read
            loop {
                if self.replay_frame as usize >= self.replay.frames.len() { break }
                
                let ReplayFrame {time:frame_time, action} = self.replay.frames[self.replay_frame as usize];
                if frame_time > time { break }

                pending_frames.push((action, frame_time));
                // gamemode.handle_replay_frame(frame, frame_time, self).await;
                
                self.replay_frame += 1;
            }
        }

        // update hit timings bar
        self.hitbar_timings.retain(|(hit_time, _)| {time - hit_time < HIT_TIMING_DURATION});
        
        // update judgement indicators
        self.judgement_indicators.retain(|a| a.should_keep(time));

        // update gamemode
        let update_frames = gamemode.update(self, time).await.into_iter().map(|f|(f, time));
        pending_frames.extend(update_frames);

        if self.song.is_stopped() {
            trace!("Song over, saying map is complete");
            self.completed = true;
        }

        // update score stuff now that gamemode has been updated
        self.score.accuracy = calc_acc(&self.score);
        self.score.performance = perfcalc_for_playmode(&self.gamemode.playmode())(self.map_diff, self.score.accuracy as f32);
        // self.score.take_snapshot(time, self.health.get_ratio());

        // do fail things
        // TODO: handle edge cases, like replays, spec, autoplay, etc
        if self.failed && !self.multiplayer {
            let new_rate = f32::lerp(self.game_speed(), 0.0, (self.time() - self.failed_time) / 1000.0);

            if new_rate <= 0.05 {
                self.song.pause();

                self.completed = true;
                // self.outgoing_spectator_frame_force((self.end_time + 10.0, SpectatorAction::Failed));
                trace!("show fail menu");
            } else {
                self.song.set_rate(new_rate);
            }

            // put it back
            self.gamemode = gamemode;
            return;
        }

        // send map completed packets
        if self.completed {
            self.outgoing_spectator_frame_force(SpectatorFrame::new(self.end_time + 10.0, SpectatorAction::ScoreSync {score: self.score.score.clone()}));
            self.outgoing_spectator_frame_force(SpectatorFrame::new(self.end_time + 10.0, SpectatorAction::Buffer));

            // check if we failed
            if self.health.check_fail_at_end && self.health.is_dead() && !self.failed {
                self.fail();
            }
        }

        // update our spectator list if we can
        if let Some(mut manager) = OnlineManager::try_get_mut() {
            if let Some(our_list) = manager.spectator_info.our_spectator_list() {
                if our_list.updated || self.spectator_info.spectators.list.len() != our_list.list.len() {
                    info!("updated ingame spectator list");
                    self.spectator_info.spectators = our_list.clone();
                    self.spectator_info.spectators.updated = true; // make sure this update gets propogated to the spectator element
                    our_list.updated = false
                }
            }
        }

        // if its time to send another score sync packet
        if self.spectator_info.last_score_sync + SPECTATOR_SCORE_SYNC_INTERVAL <= time {
            self.spectator_info.last_score_sync = time;
            
            // create and send the packet
            self.outgoing_spectator_frame(SpectatorFrame::new(time, SpectatorAction::ScoreSync {score: self.score.score.clone()}))
        }

        // handle any frames
        for (frame, time) in pending_frames {
            self.handle_frame(frame, true, Some(time), &mut gamemode).await;
        }

        // put it back
        self.gamemode = gamemode;

        // handle animation
        let mut anim = std::mem::replace(&mut self.animation, Box::new(EmptyAnimation));
        anim.update(time, self).await;
        self.animation = anim;
    }

    pub async fn draw(&mut self, list: &mut RenderableCollection) {
        let time = self.time();

        // draw animation
        self.animation.draw(list).await;


        // draw gamemode
        if let Some(bounds) = self.fit_to_bounds { list.push_scissor([bounds.pos.x, bounds.pos.y, bounds.size.x, bounds.size.y]); }
        
        let mut gamemode = std::mem::take(&mut self.gamemode);
        gamemode.draw(self, list).await;
        self.gamemode = gamemode;

        if self.fit_to_bounds.is_some() { list.pop_scissor(); }


        // dont draw score, combo, etc if this is a menu bg
        if self.menu_background { return }


        // judgement indicators
        for indicator in self.judgement_indicators.iter_mut() {
            indicator.draw(time, list);
        }


        // ui element editor
        if let Some(ui_editor) = &mut self.ui_editor {
            ui_editor.draw(Vector2::ZERO, list).await;
        } 


        // ui elements
        for i in self.ui_elements.iter_mut() {
            i.draw(list)
        }

        // draw center text
        self.center_text_helper.draw(time, list);
    }
}

// judgment stuff
impl IngameManager {

    pub fn add_hit_timning(&mut self, time: f32, note_time: f32) {
        let diff = (time - note_time) / HIT_DIFF_FACTOR;
        self.add_stat(HitVarianceStat, diff);
        // $self.score.hit_timings.push(diff);
        self.hitbar_timings.push((time, diff));
    }

    // have a hitsound manager trait and hitsound_type trait, and have this pass the hitsound trait to a fn to get a sound, then play it
    // essentially the same thing as judgments
    pub async fn play_note_sound(&mut self, hitsounds: &Vec<Hitsound>) {
        // let timing_point = self.beatmap.control_point_at(note_time);


        // get volume
        let mut vol = self.settings.get_effect_vol();
        if self.menu_background { vol *= self.settings.background_game_settings.hitsound_volume };

        self.hitsound_manager.play_sound(hitsounds, vol);
    }

    /// add judgment, affects health and score, but not hit timings
    pub async fn add_judgment<HJ:HitJudgments>(&mut self, judgment: &HJ) {
        // increment judgment, if applicable
        if let Some(count) = self.score.judgments.get_mut(judgment.as_str_internal()) {
            *count += 1;
        }

        // do score 
        let combo_mult = (self.score.combo as f32 * self.score_multiplier).floor() as u16;
        match judgment.get_score(combo_mult) {
            score @ i32::MIN..=0 => self.score.score.score -= score.abs() as u64,
            score @ 1.. => self.score.score.score += score as u64,
        }

        // do combo
        match judgment.affects_combo() {
            AffectsCombo::Increment => {
                self.score.combo += 1;
                self.score.max_combo = self.score.max_combo.max(self.score.combo);
            },
            AffectsCombo::Reset => self.combo_break().await,
            AffectsCombo::Ignore => {},
        }
        
        // do health
        (self.health.do_health.clone())(&mut self.health, judgment, &self.score);

        // check health
        if !self.health.check_fail_at_end && self.health.is_dead() {
            self.fail()
        }

        // check sd/pf mods
        //TODO: if this happens, change the judgment to a miss
        if self.current_mods.has_sudden_death() && judgment.fails_sudden_death() {
            self.fail()
        }
        if self.current_mods.has_perfect() && judgment.fails_perfect() {
            self.fail()
        }

    }

    /// check and add to hit timings if found
    pub async fn check_judgment<'a, HJ:HitJudgments>(&mut self, windows: &'a Vec<(HJ, Range<f32>)>, time: f32, note_time: f32) -> Option<&'a HJ> {
        if let Some(hj) = self.check_judgment_only(windows, time, note_time) {
            self.add_judgment(hj).await;
            add_timing!(self, time, note_time);

            // return the hit judgment we got
            Some(hj)
        } else {
            None
        }

        // let diff = (time - note_time).abs() / HIT_DIFF_FACTOR / self.game_speed();
        // for (hj, window) in windows.iter() {
        //     if window.contains(&diff) {
        //         self.add_judgment(hj).await;
        //         add_timing!(self, time, note_time);

        //         // return the hit judgment we got
        //         return Some(hj)
        //     }
        // }

        // None
    }
    
    pub async fn check_judgment_condition<
        'a,
        HJ:HitJudgments,
        F:Fn() -> bool,
    >(&mut self, windows: &'a Vec<(HJ, Range<f32>)>, time: f32, note_time: f32, cond: F, if_bad: &'a HJ) -> Option<&'a HJ> {
        if let Some(hj) = self.check_judgment_only(windows, time, note_time) {
            if cond() {
                self.add_judgment(hj).await;
                add_timing!(self, time, note_time);
                // return the hit judgment we got
                Some(hj)
            } else {
                self.add_judgment(if_bad).await;
                // return the hit judgment we got
                Some(if_bad)
            }
        } else {
            None
        }

        // let diff = (time - note_time).abs() / HIT_DIFF_FACTOR / self.game_speed();
        // for (hj, window) in windows.iter() {
        //     if window.contains(&diff) {
        //         let is_okay = cond();
        //         if is_okay {
        //             self.add_judgment(hj).await;
        //             add_timing!(self, time, note_time);
        //             // return the hit judgment we got
        //             return Some(hj)
        //         } else {
        //             self.add_judgment(if_bad).await;
        //             // return the hit judgment we got
        //             return Some(if_bad)
        //         }

        //     }
        // }

        // info!("no judgment");
        // None
    }

    /// only check if the note + hit fit into a window, and if so, return the corresponding judgment
    pub fn check_judgment_only<'a, HJ:HitJudgments>(&self, windows: &'a Vec<(HJ, Range<f32>)>, time: f32, note_time: f32) -> Option<&'a HJ> {
        let diff = (time - note_time).abs() / HIT_DIFF_FACTOR / self.game_speed();
        for (hj, window) in windows.iter() {
            if window.contains(&diff) {
                // return the hit judgment we got
                return Some(hj)
            }
        }

        None
    }


    pub fn add_judgement_indicator<HI:JudgementIndicator+'static>(&mut self, mut indicator: HI) {
        indicator.set_start_time(self.time());
        indicator.set_draw_duration(self.common_game_settings.hit_indicator_draw_duration, &self.settings);
        self.judgement_indicators.push(Box::new(indicator))
    }


    pub fn add_stat(&mut self, stat: impl GameModeStat, value: f32) {
        self.score.insert_stat(stat, value)
    }

}

// getters, setters, properties
impl IngameManager {
    pub fn all_scores(&self) -> Vec<&IngameScore> {
        let mut list = Vec::new();
        for score in self.score_list.iter() {
            list.push(score)
        }

        list.push(&self.score);

        // sort by points
        list.sort_by(|a,b| b.score.score.cmp(&a.score.score));

        list
    }

    pub fn time(&self) -> f32 {
        let t = self.song.get_position();

        t - (self.lead_in_time + self.beatmap_preferences.audio_offset + self.settings.global_offset)
    }

    pub fn should_save_score(&self) -> bool {
        let should = !(self.replaying || self.current_mods.has_autoplay() || self.ui_changed);
        should
    }

    // is this game pausable
    pub fn can_pause(&mut self) -> bool {
        if self.multiplayer { return false; }
        self.should_pause || !(self.current_mods.has_autoplay() || self.replaying || self.failed)
    }

    #[inline]
    pub fn game_speed(&self) -> f32 {
        if self.menu_background {
            1.0 // TODO: 
        } else {
            self.current_mods.get_speed()
        }
    }


    pub fn current_timing_point(&self) -> &TimingPoint {
        self.timing_points.timing_point()
        // &self.timing_points[self.timing_point_index]
    }
    pub fn timing_point_at(&self, time: f32, allow_inherited: bool) -> &TimingPoint {
        self.timing_points.timing_point_at(time, allow_inherited)
    }

}

// Events and States
impl IngameManager {
    // can be from either paused or new
    pub async fn start(&mut self) {
        if self.settings.allow_gamemode_cursor_ripple_override {
            CursorManager::set_ripple_override(self.gamemode.ripple_size());
        }

        if self.gamemode.show_cursor() || self.menu_background {
            CursorManager::set_visible(true)
        } else {
            CursorManager::set_visible(false)
        }

        // if !self.gamemode.show_cursor() {
        //     if !self.menu_background {
        //         CursorManager::set_visible(false)
        //     } else {
        //         CursorManager::set_visible(true);
        //     }
        // } else if self.replaying || self.current_mods.has_autoplay() {
        //     CursorManager::show_system_cursor(true)
        // } else {
        //     CursorManager::set_visible(true);
        //     CursorManager::show_system_cursor(false);
        // }

        self.pause_pending = false;
        self.should_pause = false;

        // offset our start time by the duration of the pause
        if let Some(pause_time) = std::mem::take(&mut self.pause_start) {
            self.start_time += chrono::Utc::now().timestamp() - pause_time
        }

        // re init ui because pointers may not be valid anymore
        self.ui_elements.clear();
        self.init_ui().await;

        if !self.started {
            self.reset().await;

            if !self.replaying {
                self.outgoing_spectator_frame(SpectatorFrame::new(0.0, SpectatorAction::Play {
                    beatmap_hash: self.beatmap.hash(),
                    mode: self.gamemode.playmode(),
                    mods: self.score.mods_string_sorted(),
                    speed: self.current_mods.speed,
                    map_game: self.metadata.beatmap_type.into(),
                    map_link: None
                }));
                
                // self.outgoing_spectator_frame(SpectatorFrame::new(0.0, SpectatorAction::MapInfo {
                //     beatmap_hash: self.beatmap.hash(),
                //     game: format!("{:?}", self.metadata.beatmap_type).to_lowercase(),
                //     download_link: None
                // }));
            }

            if self.menu_background {
                // dont reset the song, and dont do lead in
                self.lead_in_time = 0.0;
            } else {
                self.song.set_position(0.0);
                if self.song.is_stopped() { self.song.play(true); }
                self.song.pause();
                self.song.set_rate(self.current_mods.get_speed());
                
                self.lead_in_timer = Instant::now();
                self.lead_in_time = LEAD_IN_TIME;
            }

            // volume is set when the song is actually started (when lead_in_time is <= 0)
            self.started = true;

            // run the startup code
            let mut on_start:Box<dyn FnOnce(&mut Self) + Send + Sync> = Box::new(|_|{});
            std::mem::swap(&mut self.on_start, &mut on_start);
            on_start(self);

        } else if self.lead_in_time <= 0.0 {
            // if this is the menu, dont do anything
            if self.menu_background {return}
            
            let frame = SpectatorAction::UnPause;
            let time = self.time();
            self.outgoing_spectator_frame(SpectatorFrame::new(time, frame));
            self.song.play(false);

            let mut gamemode = std::mem::take(&mut self.gamemode);
            gamemode.unpause(self);
            self.gamemode = gamemode;
        }
    }
    pub fn pause(&mut self) {
        // make sure the cursor is visible
        CursorManager::set_visible(true);
        // undo any cursor override
        CursorManager::set_ripple_override(None);

        self.song.pause();
        self.pause_start = Some(chrono::Utc::now().timestamp());

        // is there anything else we need to do?

        // might mess with lead-in but meh

        let time = self.time();
        self.outgoing_spectator_frame_force(SpectatorFrame::new(time, SpectatorAction::Pause));

        let mut gamemode = std::mem::take(&mut self.gamemode);
        gamemode.pause(self);
        self.gamemode = gamemode;
    }
    pub async fn reset(&mut self) {
        self.gamemode.reset(&self.beatmap).await;
        self.health.reset();
        self.key_counter.reset();
        self.hitbar_timings.clear();
        self.judgement_indicators.clear();
        self.restart_key_hold_start = None;

        if self.menu_background {
            self.gamemode.apply_mods(self.current_mods.clone()).await;
        } else {
            // reset song
            self.song.set_rate(self.game_speed());
            self.song.set_position(0.0);
            if self.song.is_stopped() { self.song.play(true); }
            self.song.pause();
        }

        self.completed = false;
        self.started = false;
        self.failed = false;
        self.lead_in_time = LEAD_IN_TIME / self.current_mods.get_speed();
        self.lead_in_timer = Instant::now();
        self.map_diff = get_diff(&self.beatmap.get_beatmap_meta(), &self.gamemode.playmode(), &self.current_mods).unwrap_or_default();
        self.score = IngameScore::new(Score::new(self.beatmap.hash(), self.settings.username.clone(), self.gamemode.playmode()), true, false);
        self.score.speed = self.current_mods.get_speed();
        self.score_multiplier = 1.0;
        self.timing_points.reset();

        {
            *self.score.mods_mut() = self.current_mods.mods.clone();
            let playmode = self.gamemode.playmode();

            // get all available mods for this playmode
            let ok_mods = ModManager::mods_for_playmode_as_hashmap(&playmode);
            
            // purge any non-gamemode mods, and get the score multiplier for mods that are enabled
            self.score.mods_mut().retain(|m| {
                if let Some(m) = ok_mods.get(m) {
                    self.score_multiplier *= m.score_multiplier();
                    true
                } else {
                    false
                }
            });
        }


        self.replay_frame = 0;
        if !self.replaying {
            // only reset the replay if we arent replaying
            self.replay = Replay::new();
            self.score.speed = self.current_mods.get_speed();
        } else {
            if let Some(score) = &self.replay.score_data {
                self.score.username = score.username.clone();
            }
        }

        // reset elements
        self.ui_elements.iter_mut().for_each(|e|e.reset_element());

        // re-add judgments to score
        for j in self.judgment_type.variants() {
            let id = j.as_str_internal();
            self.score.judgments.insert(id.to_owned(), 0);
        }
        
    }
    pub fn fail(&mut self) {
        if self.failed || self.current_mods.has_nofail() || self.current_mods.has_autoplay() || self.menu_background { return }
        self.failed = true;
        self.failed_time = self.time();
    }

    pub async fn combo_break(&mut self) {
        // play hitsound
        if self.score.combo >= 20 && !self.menu_background {
            let combobreak = Hitsound::new_simple("combobreak");
            // index of 1 because we want to try beatmap sounds
            self.hitsound_manager.play_sound_single(&combobreak, None, self.settings.get_effect_vol());
        }

        // reset combo to 0
        self.score.combo = 0;
    }

    /// the time set here will be properly applied next update call, as async is required
    pub fn jump_to_time(&mut self, time: f32, skip_intro: bool) {
        if skip_intro {
            self.lead_in_time = 0.0;
        }
        
        self.song.set_position(time);

        self.pending_time_jump = Some(time);
    }

    pub fn on_complete(&mut self) {
        // make sure the cursor is visible
        CursorManager::set_visible(true);
        // undo any cursor override
        CursorManager::set_ripple_override(None);

        // let info = get_gamemode_info(&self.score.playmode).unwrap();
        // let groups = self.score.stats.into_groups(&info.get_stat_groups());
        // println!("{groups:#?}")
    }
    
    pub fn make_menu_background(&mut self) {
        self.menu_background = true;

        self.lead_in_time = 0.0;
        self.pending_time_jump = Some(self.time());

        let mut mods = self.current_mods.as_ref().clone();
        mods.add_mod(Autoplay.name());
        self.current_mods = Arc::new(mods);
    }

    pub fn make_multiplayer(&mut self) {
        self.multiplayer = true;
        self.score_loader = None;
    }

    pub async fn fit_to_area(&mut self, bounds: Bounds) {
        self.fit_to_bounds = Some(bounds);
        self.gamemode.fit_to_area(bounds.pos, bounds.size).await;
    }
}

// Input Handlers
impl IngameManager {
    async fn handle_frame(&mut self, frame: ReplayAction, force: bool, force_time: Option<f32>, gamemode: &mut Box<dyn GameMode>) {
        // note to self: force is used when the frames are from the gamemode's update function
        if let ReplayAction::Press(KeyPress::SkipIntro) = frame {
            gamemode.skip_intro(self);
            // more to do?
            return;
        }
        
        let add_frames = !(self.current_mods.has_autoplay() || self.replaying);

        if force || add_frames {
            match frame {
                ReplayAction::Press(k) => self.key_counter.key_down(k),
                ReplayAction::Release(k) => self.key_counter.key_up(k),
                _ => {}
            }

            let time = force_time.unwrap_or_else(||self.time());
            gamemode.handle_replay_frame(frame, time, self).await;

            if add_frames {
                self.replay.frames.push(ReplayFrame::new(time, frame));
                self.outgoing_spectator_frame(SpectatorFrame::new(time, SpectatorAction::ReplayAction{ action: frame }));
            }
        }
    }

    async fn handle_input(&mut self, frame: Option<ReplayAction>) {
        let Some(frame) = frame else { return };

        let mut gamemode = std::mem::take(&mut self.gamemode);
        self.handle_frame(frame, false, None, &mut gamemode).await;
        self.gamemode = gamemode;
    }

    pub async fn key_down(&mut self, key:Key, mods: KeyModifiers) {
        if (self.replaying || self.current_mods.has_autoplay()) && !self.menu_background {
            // check replay-only keys
            if key == Key::Escape {
                self.started = false;
                self.completed = true;
                return;
            }
        }

        // check map restart key
        if key == self.common_game_settings.map_restart_key {
            self.restart_key_hold_start = Some(Instant::now());
            return;
        }

        if self.failed && key == Key::Escape {
            // set the failed time to negative, so it triggers the end
            self.failed_time = -1000.0;
        }
        if self.failed && !self.multiplayer { return }
        

        if key == Key::Escape && self.can_pause() {
            self.should_pause = true;
        }


        
        if let Some(ui_editor) = &mut self.ui_editor {
            ui_editor.on_key_press(key, &mods, &mut ()).await;
            if key == Key::F9 {
                ui_editor.should_close = true;

                // re-disable cursor
                CursorManager::set_visible(false);
            }
        } else if key == Key::F9 {
            self.ui_editor = Some(GameUIEditorDialog::new(std::mem::take(&mut self.ui_elements)));
            self.ui_changed = true;

            if !self.replaying {
                // start autoplay
                self.replaying = true;

                let mut new_mods = self.current_mods.as_ref().clone();
                new_mods.add_mod(Autoplay.name());
                self.current_mods = Arc::new(new_mods);
            }
            
            CursorManager::set_visible(true);
        }

        // check for offset changing keys
        if mods.shift {
            let mut t = 0.0;
            if key == self.common_game_settings.key_offset_up { t = 5.0 }
            if key == self.common_game_settings.key_offset_down { t = -5.0 }

            if t != 0.0 {
                self.increment_global_offset(t).await;
            }
        } else {
            if key == self.common_game_settings.key_offset_up { self.increment_offset(5.0).await; }
            if key == self.common_game_settings.key_offset_down { self.increment_offset(-5.0).await; }
        }


        // skip intro
        let frame = if key == Key::Space {
            Some(ReplayAction::Press(KeyPress::SkipIntro))
        } else {
            self.gamemode.key_down(key).await
        };

        self.handle_input(frame).await;
    }
    pub async fn key_up(&mut self, key:Key) {
        if self.failed && !self.multiplayer { return }
        
        // check map restart key
        if key == self.common_game_settings.map_restart_key {
            self.restart_key_hold_start = None;
            return;
        }

        let frame = self.gamemode.key_up(key).await;
        self.handle_input(frame).await;
    }
    pub async fn on_text(&mut self, text:&String, mods: &KeyModifiers) {
        if self.failed && !self.multiplayer { return }
        let frame = self.gamemode.on_text(text, mods).await;
        self.handle_input(frame).await;
    }
    
    
    pub async fn mouse_move(&mut self, pos:Vector2) {
        if let Some(ui_editor) = &mut self.ui_editor {
            ui_editor.on_mouse_move(pos, &mut ()).await;
        }

        // if self.failed && !self.multiplayer { return }

        let frame = self.gamemode.mouse_move(pos).await;
        self.handle_input(frame).await;
    }
    pub async fn mouse_down(&mut self, btn:MouseButton) {
        if let Some(ui_editor) = &mut self.ui_editor {
            ui_editor.on_mouse_down(Vector2::ZERO, btn, &KeyModifiers::default(), &mut ()).await;
            return
        }

        if self.failed && !self.multiplayer { return }
        let frame = self.gamemode.mouse_down(btn).await;
        self.handle_input(frame).await;
    }
    pub async fn mouse_up(&mut self, btn:MouseButton) {
        if let Some(ui_editor) = &mut self.ui_editor {
            ui_editor.on_mouse_up(Vector2::ZERO, btn, &KeyModifiers::default(), &mut ()).await;
            return
        }

        if self.failed && !self.multiplayer { return }
        let frame = self.gamemode.mouse_up(btn).await;
        self.handle_input(frame).await;
    }
    pub async fn mouse_scroll(&mut self, delta:f32) {
        if let Some(ui_editor) = &mut self.ui_editor {
            ui_editor.on_mouse_scroll(delta, &mut ()).await;
        } 

        if self.failed && !self.multiplayer { return }
        let frame = self.gamemode.mouse_scroll(delta).await;
        self.handle_input(frame).await;
    }


    pub async fn controller_press(&mut self, c: &GamepadInfo, btn: ControllerButton) {
        if self.failed && !self.multiplayer { return }
        let frame = self.gamemode.controller_press(c, btn).await;
        self.handle_input(frame).await;
    }
    pub async fn controller_release(&mut self, c: &GamepadInfo, btn: ControllerButton) {
        if self.failed && !self.multiplayer { return }
        let frame = self.gamemode.controller_release(c, btn).await;
        self.handle_input(frame).await;
    }
    pub async fn controller_axis(&mut self, c: &GamepadInfo, axis_data:HashMap<Axis, (bool, f32)>) {
        if self.failed && !self.multiplayer { return }
        let frame = self.gamemode.controller_axis(c, axis_data).await;
        self.handle_input(frame).await;
    }

    pub fn window_focus_lost(&mut self, got_focus: bool) {
        // info!("window focus changed");
        if got_focus {
            self.pause_pending = false
        } else {
            if self.can_pause() {
                if self.in_break() { self.pause_pending = true } else { self.should_pause = true }
            }
        }
    }


    pub async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.window_size = window_size.clone();
        self.gamemode.window_size_changed(window_size).await;
        self.animation.window_size_changed(self.window_size.0);

        // TODO: relocate ui elements properly
        if let Some(mut editor) = std::mem::take(&mut self.ui_editor) {
            self.init_ui().await;
            editor.elements = std::mem::take(&mut self.ui_elements);
            self.ui_editor = Some(editor);
        } else {
            self.ui_elements.clear();
            self.init_ui().await;
        }
    }
}

// other misc stuff that isnt touched often and i just wanted it out of the way
impl IngameManager {
    pub fn set_replay(&mut self, replay: Replay) {
        self.replaying = true;
        self.replay = replay;
        
        // load speed from score
        if let Some(score) = &self.replay.score_data {
            let mut mods = ModManager::new();
            mods.mods = score.mods();
            mods.set_speed(score.speed);
            self.current_mods = Arc::new(mods);
            *self.score.mods_mut() = self.current_mods.mods.clone();

            self.score.username = score.username.clone()

        } else {
            self.score.username = "User".to_owned();
        }
    }
    
    pub async fn increment_offset(&mut self, delta:f32) {
        let time = self.time();
        self.beatmap_preferences.audio_offset += delta;
        self.center_text_helper.set_value(format!("Offset: {:.2}ms", self.beatmap_preferences.audio_offset), time);

        // update the beatmap offset
        let new_prefs = self.beatmap_preferences.clone();
        let hash = self.beatmap.hash();
        tokio::spawn(async move { Database::save_beatmap_prefs(&hash, &new_prefs); });
    }
    
    pub async fn increment_global_offset(&mut self, delta:f32) {
        let time = self.time();
        let mut settings = Settings::get_mut();
        settings.global_offset += delta;

        self.center_text_helper.set_value(format!("Global Offset: {:.2}ms", settings.global_offset), time);
    }

    pub async fn force_update_settings(&mut self) {
        self.settings.update();
        self.gamemode.force_update_settings(&self.settings).await;
    }

    pub async fn reload_skin(&mut self) {
        info!("reloading skin");
        self.gamemode.reload_skin().await;
        self.hitsound_manager.reload_skin(&self.settings).await;
    }


    /// helper since most texture loads will look something like this
    pub async fn load_texture_maybe(name: impl AsRef<str> + Send + Sync, grayscale:bool, mut on_loaded: impl FnMut(&mut Image)) -> Option<Image> {
        SkinManager::get_texture_grayscale(name, true, grayscale).await.map(|mut i| {on_loaded(&mut i); i})
    }


    fn in_break(&self) -> bool {
        let time = self.time();
        #[allow(irrefutable_let_patterns)]
        self.events.iter().find(|f| if let InGameEvent::Break { start, end } = f { time >= *start && time < *end } else { false }).is_some()
    }

}

// Spectator Stuff
impl IngameManager {
    pub fn outgoing_spectator_frame(&mut self, frame: SpectatorFrame) {
        if self.menu_background || self.replaying { return }
        OnlineManager::send_spec_frames(vec![frame], false)
    }
    pub fn outgoing_spectator_frame_force(&mut self, frame: SpectatorFrame) {
        if self.menu_background || self.replaying { return }
        OnlineManager::send_spec_frames(vec![frame], true);
    }
}

// default
impl Default for IngameManager {
    fn default() -> Self {
        Self { 
            song: AudioManager::empty_stream(),
            judgement_indicators: Vec::new(),
            hitsound_manager: HitsoundManager::new(String::new()),

            failed: false,
            failed_time: 0.0,
            health: HealthHelper::default(),
            beatmap: Default::default(),
            metadata: Default::default(),
            gamemode: Default::default(),
            current_mods: Default::default(),
            score: IngameScore::new(Default::default(), true, false),
            score_multiplier: 1.0,
            replay: Default::default(),
            started: Default::default(),
            completed: Default::default(),
            replaying: Default::default(),
            menu_background: Default::default(),
            multiplayer: false,
            end_time: Default::default(),
            lead_in_time: Default::default(),
            lead_in_timer: Instant::now(),
            timing_points: Default::default(),
            // timing_point_index: Default::default(),
            center_text_helper: Default::default(),
            hitbar_timings: Default::default(),
            replay_frame: Default::default(),
            spectator_info: Default::default(),
            on_start: Box::new(|_|{}),
            animation: Box::new(EmptyAnimation),
            fit_to_bounds: None,

            common_game_settings: Default::default(),

            score_list: Vec::new(),
            score_loader: None,
            beatmap_preferences: Default::default(),
            should_pause: false,
            pause_pending: false,
            events: Vec::new(),
            ui_elements: Vec::new(),
            ui_editor: None,
            key_counter: KeyCounter::default(),

            ui_changed: false,

            judgment_type: Box::new(DefaultHitJudgments::None),

            settings: SettingsHelper::new(),
            window_size: WindowSize::get(),
            pending_time_jump: None,
            skin_helper: CurrentSkinHelper::new(),

            restart_key_hold_start: None,
            map_diff: 0.0,
            start_time: 0,
            pause_start: None,
        }
    }
}


#[derive(Clone, Debug)]
pub enum InGameEvent {
    Break { start: f32, end: f32 }
}

#[derive(Default)]
pub struct IngameSpectatorInfo {
    /// when was the last time the score was synchronized?
    pub last_score_sync: f32,   

    /// who is currently spectating us?
    pub spectators: SpectatorList
}
