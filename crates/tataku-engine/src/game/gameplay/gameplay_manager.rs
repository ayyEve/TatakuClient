use crate::prelude::*;


/// how much time should pass at beatmap start before audio begins playing (and the map "starts")
pub const LEAD_IN_TIME:f32 = 1000.0;
/// how long should center text be drawn for?
const CENTER_TEXT_DRAW_TIME:f32 = 2_000.0;

/// ms between spectator score sync packets
const SPECTATOR_SCORE_SYNC_INTERVAL:f32 = 1000.0;

const HIT_DIFF_FACTOR:f32 = 1.0;

pub const SCORE_SEND_TIME:f32 = 1_000.0;

/// how long of a buffer should we have? (ms)
pub const SPECTATOR_BUFFER_OK_DURATION:f32 = 500.0;

// // bc im lazy
// macro_rules! add_timing {
//     ($self:ident, $time:expr, $note_time:expr) => {{
//         let diff = ($time - $note_time) / HIT_DIFF_FACTOR;
//         $self.add_stat(HitVarianceStat, diff);
//         // $self.score.hit_timings.push(diff);
//         $self.hitbar_timings.push(($time, diff));
//     }}
// }

macro_rules! create_update_state {
    ($self: ident, $time: expr, $settings: expr) => {
        GameplayStateForUpdate {
            time: $time,
            game_speed: $self.game_speed(),
            completed: $self.completed,

            mods: &$self.current_mods,
            current_timing_point: $self.timing_points.timing_point(),
            timing_points: &$self.timing_points,
            gameplay_mode: &$self.gameplay_mode,
            score: &$self.score,
            actions: Vec::new(),
            settings: $settings
        }
    }
}

pub struct GameplayManager {
    pub actions: ActionQueue,


    pub beatmap: Beatmap,
    pub metadata: Arc<BeatmapMeta>,
    pub gamemode: Box<dyn GameMode>,
    pub gamemode_info: GameModeInfo,
    pub current_mods: Arc<ModManager>,
    pub beatmap_preferences: BeatmapPreferences,

    pub gameplay_mode: Box<GameplayModeInner>,
    gameplay_actions: Vec<GameplayAction>,


    pub score: IngameScore,
    pub score_multiplier: f32,

    pub health: Box<dyn HealthManager>,
    pub judgments: Vec<HitJudgment>,
    pub key_counter: KeyCounter,
    ui_elements: Vec<UIElement>,

    #[cfg(feature="graphics")]
    animation: Box<dyn BeatmapAnimation>,

    pub score_list: Vec<IngameScore>,
    scores_loaded: bool,
    // score_loader: Option<Arc<AsyncRwLock<ScoreLoaderHelper>>>,

    // used for discord rich presence
    pub start_time: i64,
    pub started: bool,
    pub completed: bool,
    pub failed: bool,
    pub failed_time: f32,
    pub end_time: f32,
    pub lead_in_time: f32,
    pub lead_in_timer: Instant,
    
    global_offset: f32,

    /// has something about the ui been changed?
    /// this will make the play unrankable and should not be saved
    pub ui_changed: bool,

    /// should the manager be paused?
    pub should_pause: bool,
    /// is a pause pending?
    /// used for breaks. if the user tabs out during a break, a pause is pending, but we shouldnt pause until the break is over (or almost over i guess)
    pause_pending: bool,
    pause_start: Option<i64>,
    restart_key_hold_start: Option<Instant>,


    // pub timing_points: Vec<TimingPoint>,
    // pub timing_point_index: usize,
    // next_beat: f32,
    pub timing_points: TimingPointHelper,

    // pub song: Arc<dyn AudioInstance>,
    pub hitsound_manager: HitsoundManager,

    /// center text helper (ie, for offset and global offset)
    pub center_text_helper: CenteredTextHelper,

    /// (map.time, note.time - hit.time)
    pub hitbar_timings: Vec<(f32, f32)>,

    /// list of judgement indicators to draw
    pub judgement_indicators: Vec<Box<dyn JudgementIndicator>>,

    pub common_game_settings: Arc<CommonGameplaySettings>,
    // settings: SettingsHelper,
    window_size: Arc<WindowSize>,
    fit_to_bounds: Option<Bounds>,

    // spectator info
    pub spectator_info: GameplaySpectatorInfo,

    frame_sender: Box<dyn GameplayManagerOnline>,
    diff_provider: Box<dyn DifficultyProvider>,

    /// what should the game do on start?
    /// mainly a helper for spectator
    pub on_start: Box<dyn FnOnce(&mut Self) + Send + Sync>,

    pub events: Vec<IngameEvent>,
    // #[cfg(feature="graphics")]
    // ui_editor: Option<GameUIEditorDialog>,

    pending_time_jump: Option<f32>,
    pending_frames: Vec<ReplayFrame>,

    map_diff: f32,
    song_time: f32,
}

impl GameplayManager {
    pub async fn new(
        beatmap: Beatmap,
        mut gamemode: Box<dyn GameMode>,
        mut current_mods: ModManager,
        settings: &Settings,
    ) -> Self {
        let playmode = gamemode.playmode();
        let metadata = beatmap.get_beatmap_meta();

        let beatmap_preferences = Database::get_beatmap_prefs(metadata.beatmap_hash).await;

        let timing_points = beatmap.get_timing_points();


        if current_mods.get_speed() == 0.0 { current_mods.set_speed(1.0); }
        let current_mods = Arc::new(current_mods);

        let mut score =  Score::new(beatmap.hash(), settings.username.clone(), playmode.clone().into_owned());
        score.speed = current_mods.speed;


        // let score_loader = Some(SCORE_HELPER.read().await.get_scores(metadata.beatmap_hash, &playmode).await);
        let key_counter = KeyCounter::new(gamemode.get_possible_keys().into_iter().map(|a| (a.0, a.1.to_owned())).collect());

        // hardcode for now
        let audio_playmode_prefix = match &*playmode {
            "taiko" => "taiko".to_owned(),
            "mania" => "mania".to_owned(),
            _ => String::new(),
        };

        let mut actions = ActionQueue::new();
        let gamemode_info = gamemode.get_info();

        let mut hitsound_manager = HitsoundManager::new(audio_playmode_prefix);
        hitsound_manager.init(&metadata, &mut actions, settings).await;

        let events = beatmap.get_events();
        // println!("loaded events {events:?}");

        // make sure the gamemode has the correct mods applied
        gamemode.apply_mods(current_mods.clone()).await;

        Self {
            actions,
            frame_sender: Box::new(DummyOnlineThing),
            diff_provider: Box::new(DummyDiffProvider),
            
            metadata,
            timing_points: TimingPointHelper::new(timing_points, beatmap.slider_velocity()),
            // hitsound_cache,
            current_mods,
            health: Box::new(DefaultHealthManager::new()),
            key_counter,

            judgments: gamemode_info.judgments.into_iter().copied().collect(),
            score: IngameScore::new(score, true, false),

            beatmap,
            #[cfg(feature="graphics")]
            animation: Box::new(EmptyAnimation),

            hitsound_manager,
            gamemode_info,
            // song,

            lead_in_time: LEAD_IN_TIME,
            lead_in_timer: Instant::now(),
            end_time: gamemode.end_time(),
            global_offset: settings.global_offset,

            center_text_helper: CenteredTextHelper::new(CENTER_TEXT_DRAW_TIME).await,
            beatmap_preferences,

            common_game_settings: Arc::new(settings.common_game_settings.clone()),

            gamemode,

            scores_loaded: false,
            score_list: Vec::new(),
            // score_loader, values: &mut dyn Reflec
            window_size: WindowSize::get(),
            start_time: chrono::Utc::now().timestamp(),

            events,


            judgement_indicators: Vec::new(),
            gameplay_mode: Box::new(GameplayModeInner::Normal),
            gameplay_actions: Vec::new(),

            failed: false,
            failed_time: 0.0,
            score_multiplier: 1.0,
            started: false,
            completed: false,

            hitbar_timings: Vec::new(),
            spectator_info: GameplaySpectatorInfo::default(),
            on_start: Box::new(|_|{}),
            fit_to_bounds: None,
            should_pause: false,
            pause_pending: false,
            ui_elements: Vec::new(),
            // #[cfg(feature="graphics")]
            // ui_editor: None,
            ui_changed: false,

            pending_time_jump: None,
            pending_frames: Vec::new(),

            restart_key_hold_start: None,
            map_diff: 0.0,
            pause_start: None,
            song_time: 0.0,
        }
    }

    #[cfg(feature="graphics")]
    async fn init_ui(&mut self) {
        let mut loader = DefaultUiElementLoader;
        // if self.ui_editor.is_some() { return }

        let playmode = self.gamemode.playmode();
        let get_name = |name| {
            format!("{playmode}_{name}")
        };

        // score
        self.ui_elements.push(loader.load(
            &get_name("score"),
            Vector2::new(self.window_size.x, 0.0),
            Box::new(ScoreElement::new().await)
        ).await);

        // Acc
        self.ui_elements.push(loader.load(
            &get_name("acc"),
            Vector2::new(self.window_size.x, 40.0),
            Box::new(AccuracyElement::new().await)
        ).await);

        // Performance
        // TODO: calc diff before starting somehow?
        self.ui_elements.push(loader.load(
            &get_name("perf"),
            Vector2::new(self.window_size.x, 80.0),
            Box::new(PerformanceElement::new().await)
        ).await);

        // Healthbar
        self.ui_elements.push(loader.load(
            &get_name("healthbar"),
            Vector2::ZERO,
            Box::new(HealthBarElement::new(self.common_game_settings.clone()).await)
        ).await);

        // Duration Bar
        self.ui_elements.push(loader.load(
            &get_name("durationbar"),
            Vector2::new(0.0, self.window_size.y),
            Box::new(DurationBarElement::new(self.common_game_settings.clone()))
        ).await);

        // Judgement Bar
        self.ui_elements.push(loader.load(
            &get_name("judgementbar"),
            Vector2::new(self.window_size.x/2.0, self.window_size.y),
            Box::new(JudgementBarElement::new(self.gamemode.timing_bar_things()))
        ).await);

        // Key Counter
        self.ui_elements.push(loader.load(
            &get_name("key_counter"),
            Vector2::new(self.window_size.x, self.window_size.y/2.0),
            Box::new(KeyCounterElement::new().await)
        ).await);

        // Spectators
        self.ui_elements.push(loader.load(
            &get_name("spectators"),
            Vector2::new(0.0, self.window_size.y/3.0),
            Box::new(SpectatorsElement::new())
        ).await);

        // judgement counter
        self.ui_elements.push(loader.load(
            &get_name("judgement_counter"),
            Vector2::new(self.window_size.x, self.window_size.y * (2.0/3.0)),
            Box::new(JudgementCounterElement::new().await)
        ).await);



        // elapsed timer
        self.ui_elements.push(loader.load(
            &get_name("elapsed_timer"),
            Vector2::new(30.0, self.window_size.y - 150.0),
            Box::new(ElapsedElement::new().await)
        ).await);

        // remaining timer
        self.ui_elements.push(loader.load(
            &get_name("remaining_timer"),
            Vector2::new(self.window_size.x - 300.0, self.window_size.y - 150.0),
            Box::new(RemainingElement::new().await)
        ).await);



        // anything in the gamemode itself
        self.gamemode.get_ui_elements(
            self.window_size.0, 
            &mut self.ui_elements,
            &mut loader,
        ).await;
    }

    pub async fn apply_mods(&mut self, mut mods: ModManager) {
        if self.gameplay_mode.is_preview() {
            mods.add_mod(Autoplay);
        }

        self.current_mods = Arc::new(mods);
        self.gamemode.apply_mods(self.current_mods.clone()).await;
    }

    pub async fn update(&mut self, values: &mut dyn Reflect) -> Vec<TatakuAction> {
        let new_time = *values.reflect_get::<f32>("song.position").unwrap();
        let settings = values.reflect_get::<Settings>("settings").unwrap();

        // if theres a time difference of over a second from when the last update was, pause the hitsound manager because there might be audio spam
        if new_time - self.song_time > 1000.0 {
            self.hitsound_manager.enabled = false;
        }

        self.song_time = new_time;

        // // update settings
        // self.settings.update();

        // make sure we jump to the time we're supposed to be at
        if let Some(time) = self.pending_time_jump {
            self.hitsound_manager.enabled = false; // try to mitigate spamming the user's ears with hitsounds
            self.pending_time_jump = None;
            self.gamemode.time_jump(time).await;
        }

        // check map restart
        if let Some(press_time) = self.restart_key_hold_start {
            if press_time.as_millis() >= self.common_game_settings.map_restart_delay {
                self.reset().await;
                return self.actions.take();
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
        if !self.gameplay_mode.is_preview() {
            let mut ui_elements = std::mem::take(&mut self.ui_elements);
            ui_elements.iter_mut().for_each(|ui| ui.update(self));
            self.ui_elements = ui_elements;
        }

        // // update ui editor
        // #[cfg(feature="graphics")] {
        //     let mut ui_editor = std::mem::take(&mut self.ui_editor);
        //     let mut should_close = false;
        //     if let Some(ui_editor) = &mut ui_editor {
        //         ui_editor.update().await;
        //         ui_editor.update_elements(self);

        //         if ui_editor.should_close() {
        //             self.ui_elements = std::mem::take(&mut ui_editor.elements);
        //             should_close = true
        //         }
        //     }
        //     if !should_close {
        //         self.ui_editor = ui_editor;
        //     }
        // }
        // get the time with offsets
        let time = self.time();

        // check lead-in time
        if self.lead_in_time > 0.0 {
            let elapsed = self.lead_in_timer.elapsed().as_micros() as f32 / 1000.0;
            self.lead_in_timer = Instant::now();
            self.lead_in_time -= elapsed * self.game_speed();

            if self.lead_in_time <= 0.0 {
                self.actions.push(SongAction::SetRate(self.game_speed()));
                self.actions.push(SongAction::SetVolume(settings.get_music_vol()));
                self.actions.push(SongAction::SetPosition(-self.lead_in_time));
                self.actions.push(SongAction::Play);

                // self.song.set_position(-self.lead_in_time);
                // self.song.set_volume(self.settings.get_music_vol());
                // self.song.set_rate(self.game_speed());
                // self.song.play(true);

                self.lead_in_time = 0.0;
            }
        }


        // check if scores have been loaded
        // if let Some(loader) = self.score_loader.clone() {
        //     let loader = loader.read().await;
        //     if loader.done {
        //         self.score_list = loader.scores.iter().map(|s| { let mut s = s.clone(); s.is_previous = s.username == self.score.username; s }).collect();
        //         self.score_loader = None;
        //     }
        // }
        #[cfg(feature="gameplay")]
        let scores_list = values.reflect_get::<Vec<IngameScore>>("score_list.scores").unwrap();
        let scores_loaded = *values.reflect_get::<bool>("score_list.loaded").unwrap();

        if !self.scores_loaded && self.gameplay_mode.should_load_scores() && scores_loaded {
            self.score_list = scores_list.clone();
            self.scores_loaded = true;

            for s in self.score_list.iter_mut() {
                s.is_previous = s.username == self.score.username;
            }
        }


        let tp_updates = self.timing_points.update(time);
        for tp_update in tp_updates {
            match tp_update {
                TimingPointUpdate::BeatHappened(pulse_length) => self.gamemode.beat_happened(pulse_length).await,
                TimingPointUpdate::KiaiChanged(kiai) => self.gamemode.kiai_changed(kiai).await,
            }
        }

        // update hit timings bar
        self.hitbar_timings.retain(|(hit_time, _)| {time - hit_time < HIT_TIMING_DURATION});

        // update judgement indicators
        self.judgement_indicators.retain(|a| a.should_keep(time));

        // update gamemode
        let mut state = create_update_state!(self, time, settings);


        self.gamemode.update(&mut state).await;
        for action in state.actions {
            self.handle_gamemode_action(action, settings).await;
        }
        //.into_iter().map(|f| ReplayFrame::new(time, f));



        // if self.lead_in_time == 0.0 && values.get_bool("song.stopped").unwrap_or_default() {
        //     debug!("Song over, saying map is complete");
        //     self.completed = true;
        // }

        // update score stuff now that gamemode has been updated
        
        self.score.accuracy = self.gamemode_info.calc_acc(&self.score);
        self.score.performance = self.gamemode_info.calc_perf( CalcPerfInfo {
            score: &self.score,
            map_difficulty: self.map_diff, 
            accuracy: self.score.accuracy
        });
        // self.score.take_snapshot(time, self.health.get_ratio());

        // do fail things
        // TODO: handle edge cases, like replays, spec, autoplay, etc
        #[cfg(feature="gameplay")]
        if self.failed && !self.gameplay_mode.is_multi() {
            let new_rate = f32::lerp(self.game_speed(), 0.0, (self.time() - self.failed_time) / 1000.0);

            if new_rate <= 0.05 {
                self.actions.push(SongAction::Pause);
                // self.song.pause();

                self.completed = true;
                // self.outgoing_spectator_frame_force((self.end_time + 10.0, SpectatorAction::Failed));
                trace!("show fail menu");
            } else {
                self.actions.push(SongAction::SetRate(new_rate));
                // self.song.set_rate(new_rate);
            }

            return self.actions.take();
        }

        // send map completed packets
        if self.completed {
            #[cfg(feature="gameplay")] {
                let mut score = self.score.score.clone();
                score.replay = None;
                self.outgoing_spectator_frame_force(SpectatorFrame::new(self.end_time + 10.0, SpectatorAction::ScoreSync { score }));
            }

            #[cfg(feature="gameplay")]
            self.outgoing_spectator_frame_force(SpectatorFrame::new(self.end_time + 10.0, SpectatorAction::Buffer));

            // check if we failed
            if self.health.is_dead(true) && !self.failed {
                self.fail();
            }
        }


        // update according to our gameplay mode
        match &mut *self.gameplay_mode {
            // read inputs from replay if replaying
            GameplayModeInner::Replaying {
                score,
                current_frame
            } /* if !self.current_mods.has_autoplay() */ => {
                let Some(replay) = &score.replay else { unreachable!() };

                // read any frames that need to be read
                loop {
                    if *current_frame >= replay.frames.len() { break }

                    let frame = replay.frames[*current_frame];
                    if frame.time > time { break }

                    self.pending_frames.push(frame);

                    *current_frame += 1;
                }
            }

            #[cfg(feature="gameplay")]
            GameplayModeInner::Spectator {
                state,
                frames,

                replay_frames,
                current_frame,

                host_id,
                // host_username,
                good_until,
                // spectators,
                buffered_score_frames ,

                ..
            } => {
                // buffer twice as long as we need
                let buffer_duration = (time + SPECTATOR_BUFFER_OK_DURATION * 2.0).clamp(0.0, self.end_time);

                // try to read new frames from the online manager
                frames.extend(self.frame_sender.get_pending_frames());
                // if let Some(mut online_manager) = OnlineManager::try_get_mut() {
                //     online_manager.get_pending_spec_frames(*host_id);
                // }

                // handle pending frames
                while let Some(SpectatorFrame { time:frame_time, action }) = frames.pop_front() {
                    *good_until = good_until.max(frame_time);

                    // debug!("Packet: {action:?}");
                    match action {
                        SpectatorAction::Pause => {
                            trace!("Spec pause");
                            *state = SpectatorState::Paused;
                            self.gameplay_actions.push(GameplayAction::Pause);
                        }
                        SpectatorAction::UnPause => {
                            trace!("Spec unpause");
                            *state = SpectatorState::Watching;
                            self.gameplay_actions.push(GameplayAction::Resume);
                        }
                        SpectatorAction::Buffer => { /*nothing to handle here*/ },
                        SpectatorAction::SpectatingOther { .. } => {
                            self.actions.push(GameAction::AddNotification(Notification::new_text("Host speccing someone", Color::BLUE, 2000.0)));
                        }
                        SpectatorAction::ReplayAction { action } => replay_frames.push(ReplayFrame::new(frame_time, action)),

                        SpectatorAction::ScoreSync { score } => {
                            // received score update
                            trace!("Got score update");
                            buffered_score_frames.push((frame_time, score));
                        }

                        SpectatorAction::ChangingMap => {
                            trace!("Host changing maps");
                            *state = SpectatorState::MapChanging;
                            // should return back to the spectator manager menu
                            // self.pause();
                        }

                        SpectatorAction::TimeJump { time } => self.gameplay_actions.push(GameplayAction::JumpToTime{time, skip_intro: true}), //self.jump_to_time(time, true),

                        other => warn!("ingame manager told to handle unexpected spec action: {other:?}"),
                    }
                }


                // handle current state
                match state {
                    SpectatorState::Buffering => {
                        if *good_until >= buffer_duration {
                            *state = SpectatorState::Watching;
                            trace!("No longer buffering");
                            self.gameplay_actions.push(GameplayAction::Resume);
                        }
                    }

                    // currently watching someone
                    SpectatorState::Watching => {
                        // check for buffered score frames
                        buffered_score_frames.retain(|(frame_time, score)| {
                            if time <= *frame_time {
                                let mut other_score = score.clone();
                                other_score.hit_timings = self.score.hit_timings.clone();
                                self.score.score = other_score;
                                false
                            } else {
                                true
                            }
                        });

                        if *good_until >= buffer_duration {
                            loop {
                                if *current_frame >= replay_frames.len() { break }

                                let frame = replay_frames[*current_frame];
                                if frame.time > time { break }

                                self.pending_frames.push(frame);

                                *current_frame += 1;
                            }

                        } else {
                            *state = SpectatorState::Buffering;
                            trace!("Starting buffer");
                            self.gameplay_actions.push(GameplayAction::Pause);
                        }

                    }

                    _ => {}
                }
            }

            #[cfg(feature="gameplay")]
            GameplayModeInner::Multiplayer {
                last_escape_press: _,
                score_send_timer,
            } => {
                if score_send_timer.as_millis() >= SCORE_SEND_TIME {
                    score_send_timer.elapsed_and_reset();
                    let score = self.score.score.clone();
                    self.frame_sender.update_lobby_score(score);
                }
            }

            _ => {}
        }

        // handle any pending gameplay actions
        for a in self.gameplay_actions.take() {
            self.handle_action(a, settings).await;
        }

        // TODO: rework this? 
        #[cfg(feature="gameplay")]
        // update our spectator list if we can
        if let Some(our_list) = self.frame_sender.our_spectator_list() {
            if our_list.updated || self.spectator_info.spectators.list.len() != our_list.list.len() {
                info!("updated ingame spectator list");
                self.spectator_info.spectators = our_list.clone();
                self.spectator_info.spectators.updated = true; // make sure this update gets propogated to the spectator element
                // our_list.updated = false
            }
        }

        // if its time to send another score sync packet
        #[cfg(feature="gameplay")]
        if self.spectator_info.last_score_sync + SPECTATOR_SCORE_SYNC_INTERVAL <= time {
            self.spectator_info.last_score_sync = time;

            // create and send the packet

            let mut score = self.score.score.clone();
            score.replay = None;
            self.outgoing_spectator_frame(SpectatorFrame::new(time, SpectatorAction::ScoreSync { score }))
        }

        // handle any frames
        for ReplayFrame { time, action } in self.pending_frames.take() {
            self.handle_frame(action, true, Some(time), true, settings).await;
        }


        // handle animation
        #[cfg(feature="graphics")] {
            let mut anim = std::mem::replace(&mut self.animation, Box::new(EmptyAnimation));
            anim.update(time).await;
            self.animation = anim;
        }

        // update value collection
        {
            // let score:TatakuValue = (&self.score).into();
            // let mut score_data = score.to_map();

            // score_data.set_value("health", TatakuVariable::new_game(self.health.get_ratio()));
            // TODO: placing
            values.reflect_insert("score", self.score.clone()).unwrap();
            // values.set("score", TatakuVariable::new_game(score_data));
        }


        // unpause the hitsound manager next frame if it was paused earlier this frame
        // hopefully this helps with the osu note spam sounds. i think the OsuHitObject::get_pending_combo is whats spamming audio
        if !self.hitsound_manager.enabled {
            // self.hitsound_manager.enabled = false;
            self.gameplay_actions.push(GameplayAction::SetHitsoundsEnabled(true));
        }

        self.actions.take()
    }

    #[cfg(feature="graphics")]
    pub async fn draw(&mut self, list: &mut RenderableCollection) {
        let time = self.time();

        // draw animation
        self.animation.draw(list).await;


        // draw gamemode
        if let Some(bounds) = self.fit_to_bounds { list.push_scissor([bounds.pos.x, bounds.pos.y, bounds.size.x, bounds.size.y]); }

        let state = GameplayStateForDraw {
            time,
            gameplay_mode: &self.gameplay_mode,
            current_timing_point: self.timing_points.timing_point(),
            mods: &self.current_mods,
            score: &self.score,
        };
        self.gamemode.draw(state, list).await;


        if self.fit_to_bounds.is_some() { list.pop_scissor(); }


        // dont draw score, combo, etc if this is a menu bg
        if self.gameplay_mode.is_preview() { return }


        // judgement indicators
        for indicator in self.judgement_indicators.iter_mut() {
            indicator.draw(time, list);
        }


        // // ui element editor
        // if let Some(ui_editor) = &mut self.ui_editor {
        //     ui_editor.draw(Vector2::ZERO, list).await;
        // }


        // ui elements
        for i in self.ui_elements.iter_mut() {
            i.draw(list)
        }

        // draw center text
        self.center_text_helper.draw(time, list);
    }

    pub async fn handle_action(
        &mut self, 
        action: GameplayAction,
        settings: &Settings,
    ) {
        match action {
            GameplayAction::Pause => self.pause(),
            GameplayAction::Resume => self.start().await,
            GameplayAction::JumpToTime { time, skip_intro } => self.jump_to_time(time, skip_intro),
            GameplayAction::ApplyMods(mods) => self.apply_mods(mods).await,
            GameplayAction::FitToArea(bounds) => self.fit_to_area(bounds).await,
            GameplayAction::SetMode(mode) => self.set_mode(mode),

            GameplayAction::AddReplayAction { action, should_save } => self.handle_frame(action, true, Some(self.time()), should_save, settings).await,
            GameplayAction::SetHitsoundsEnabled(enabled) => self.hitsound_manager.enabled = enabled,
        }
    }

    #[async_recursion::async_recursion]
    async fn handle_gamemode_action(
        &mut self, 
        action: GamemodeAction,
        settings: &Settings
    ) {
        match action {
            GamemodeAction::AddJudgment(judgment) => {

                // increment judgment, if applicable
                if let Some(count) = self.score.judgments.get_mut(judgment.id) {
                    *count += 1;
                }

                // do score
                let combo_mult = (self.score.combo as f32 * self.score_multiplier).floor() as u16;
                let score = judgment.base_score_value;

                let score = match judgment.combo_multiplier {
                    ComboMultiplier::None => score,
                    ComboMultiplier::Custom(mult) => (score as f32 * mult) as i32,
                    ComboMultiplier::Linear { combo, multiplier, combo_cap } => {
                        let combo_mult = combo_cap.map(|cap| combo_mult.min(cap)).unwrap_or(combo_mult);
                        let times = (combo_mult % combo).max(1) as f32;

                        (score as f32 * (multiplier * times)) as i32
                    }
                };

                match score {
                    score @ i32::MIN..=0 => self.score.score.score -= score.abs() as u64,
                    score @ 1.. => self.score.score.score += score as u64,
                }

                // do combo
                match judgment.affects_combo {
                    AffectsCombo::Increment => {
                        self.score.combo += 1;
                        self.score.max_combo = self.score.max_combo.max(self.score.combo);
                    }
                    AffectsCombo::Reset => self.combo_break(settings), // self.actions.push(GamemodeAction::ComboBreak),
                    AffectsCombo::Ignore => {},
                }

                // do health
                self.health.apply_hit(&judgment, &self.score);
                self.score.health = self.health.get_ratio();

                // check health
                if self.health.is_dead(false) {
                    // self.actions.push(GamemodeAction::FailGame);
                    self.fail();
                }

                // check sd/pf mods
                //TODO: if this happens, change the judgment to a miss
                if self.current_mods.has_sudden_death() && judgment.fails_sudden_death {
                    // self.actions.push(GamemodeAction::FailGame);
                    self.fail()
                }
                if self.current_mods.has_perfect() && judgment.fails_perfect {
                    // self.actions.push(GamemodeAction::FailGame);
                    self.fail()
                }
            }
            GamemodeAction::PlayHitsounds(sounds) => {
                // TODO: old note?
                // have a hitsound manager trait and hitsound_type trait, and have this pass the hitsound trait to a fn to get a sound, then play it
                // essentially the same thing as judgments


                // let timing_point = self.beatmap.control_point_at(note_time);

                // get volume
                let mut vol = settings.get_effect_vol();
                if self.gameplay_mode.is_preview() { vol *= settings.background_game_settings.hitsound_volume };

                self.hitsound_manager.play_sound(&sounds, vol);
            }


            GamemodeAction::AddTiming { hit_time, note_time } => {
                let diff = (hit_time - note_time) / HIT_DIFF_FACTOR;
                self.score.insert_stat(HitVarianceStat, diff);
                // self.add_stat(HitVarianceStat, diff);
                // $self.score.hit_timings.push(diff);
                self.hitbar_timings.push((hit_time, diff));
            }

            GamemodeAction::AddIndicator(mut indicator) => {
                indicator.set_start_time(self.time());
                indicator.set_draw_duration(self.common_game_settings.hit_indicator_draw_duration, settings);
                self.judgement_indicators.push(indicator)
            }

            GamemodeAction::AddStat { stat, value } => self.score.insert_stat(stat, value),
            GamemodeAction::RemoveLastJudgment => self.judgement_indicators.pop().nope(),
            GamemodeAction::ComboBreak => self.combo_break(settings),
            GamemodeAction::FailGame => self.fail(),
            GamemodeAction::ReplayAction(frame) => self.handle_frame(frame.action, true, Some(frame.time), true, settings).await,
            GamemodeAction::ResetHealth => self.health.reset(),
            GamemodeAction::ReplaceHealth(new_health) => self.health = new_health,
            GamemodeAction::MapComplete => self.completed = true,
        }
    }
}

// getters, setters, properties
impl GameplayManager {
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

    #[inline]
    pub fn time(&self) -> f32 {
        self.song_time - (self.lead_in_time + self.beatmap_preferences.audio_offset + self.global_offset)
    }

    pub fn should_save_score(&self) -> bool {
        let should = !(self.gameplay_mode.is_replay() || self.current_mods.has_autoplay() || self.ui_changed);
        should
    }

    // is this game pausable
    pub fn can_pause(&mut self) -> bool {
        // never allow pausing in multi
        #[cfg(feature="gameplay")]
        if self.gameplay_mode.is_multi() { return false; }
        self.should_pause || !(self.current_mods.has_autoplay() || self.gameplay_mode.is_replay() || self.failed)
    }

    #[inline]
    pub fn game_speed(&self) -> f32 {
        if self.gameplay_mode.is_preview() {
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


    pub fn should_hide_cursor(&self) -> bool {
        !(
            self.gamemode.show_cursor()
            || self.gameplay_mode.is_preview()
            || self.gameplay_mode.is_replay()
        )
    }
}

// Events and States
impl GameplayManager {
    // can be from either paused or new
    pub async fn start(
        &mut self,
    ) {
        // if !self.gameplay_mode.is_preview() {
        //     self.hitsound_manager.enabled = false;
        // }


        if self.should_hide_cursor() {
            self.actions.push(CursorAction::SetVisible(false));
        } else {
            self.actions.push(CursorAction::SetVisible(true));
        }

        // if !self.gamemode.show_cursor() {
        //     if !self.menu_background {
        //         CursorManager::set_visible(false)
        //     } else {
        //         CursorManager::set_visible(true);
        //     }
        // } else if self.gameplay_mode.is_replay() || self.current_mods.has_autoplay() {
        //     CursorManager::show_system_cursor(true)
        // } else {
        //     CursorManager::set_visible(true);
        //     CursorManager::show_system_cursor(false);
        // }

        self.pause_pending = false;
        self.should_pause = false;

        // offset our start time by the duration of the pause
        if let Some(pause_time) = self.pause_start.take() {
            self.start_time += chrono::Utc::now().timestamp() - pause_time
        }

        // re init ui because pointers may not be valid anymore
        self.ui_elements.clear();
        #[cfg(feature="graphics")]
        self.init_ui().await;

        if !self.started {
            self.reset().await;

            //TODO: probably want to skip other things as well
            if !self.gameplay_mode.is_replay() {
                #[cfg(feature="gameplay")]
                self.outgoing_spectator_frame(SpectatorFrame::new(0.0, SpectatorAction::Play {
                    beatmap_hash: self.beatmap.hash(),
                    mode: self.gamemode.playmode().clone().into_owned(),
                    mods: self.score.mods.clone(),
                    speed: self.current_mods.speed.as_u16(),
                    map_game: self.metadata.beatmap_type.into(),
                    map_link: None
                })
            );

                // self.outgoing_spectator_frame(SpectatorFrame::new(0.0, SpectatorAction::MapInfo {
                //     beatmap_hash: self.beatmap.hash(),
                //     game: format!("{:?}", self.metadata.beatmap_type).to_lowercase(),
                //     download_link: None
                // }));
            }

            if self.gameplay_mode.is_preview() {
                // dont reset the song, and dont do lead in
                self.lead_in_time = 0.0;
            } else {
                // self.actions.push(SongMenuAction::Restart);
                // self.actions.push(SongMenuAction::Pause);
                // self.actions.push(SongMenuAction::SetPosition(0.0));
                // self.actions.push(SongMenuAction::SetRate(self.game_speed()));

                // self.song.set_position(0.0);
                // if self.song.is_stopped() { self.song.play(true); }
                // self.song.pause();
                // self.song.set_rate(self.current_mods.get_speed());

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
            if self.gameplay_mode.is_preview() { return }

            let frame = SpectatorAction::UnPause;
            let time = self.time();
            #[cfg(feature="gameplay")]
            self.outgoing_spectator_frame(
                SpectatorFrame::new(time, frame),
            );
            self.actions.push(SongAction::Play);
            // self.song.play(false);

            self.gamemode.unpause();
        }
    }
    pub fn pause(&mut self) {
        // make sure the cursor is visible
        self.actions.push(CursorAction::SetVisible(true));
        // undo any cursor override
        self.actions.push(CursorAction::OverrideRippleRadius(None));

        // // make sure the cursor is visible
        // CursorManager::set_visible(true);
        // // undo any cursor override
        // CursorManager::set_ripple_override(None);

        // self.song.pause();
        self.actions.push(SongAction::Pause);
        self.pause_start = Some(chrono::Utc::now().timestamp());

        // is there anything else we need to do?

        // might mess with lead-in but meh

        let time = self.time();
        #[cfg(feature="gameplay")]
        self.outgoing_spectator_frame_force(
            SpectatorFrame::new(time, SpectatorAction::Pause),
        );

        self.gamemode.pause();
    }
    pub async fn reset(
        &mut self,
    ) {
        self.gamemode.reset(&self.beatmap).await;
        self.health.reset();
        self.key_counter.reset();
        self.hitbar_timings.clear();
        self.judgement_indicators.clear();
        self.restart_key_hold_start = None;

        if self.gameplay_mode.is_preview() {
            self.gamemode.apply_mods(self.current_mods.clone()).await;
        } else {
            // reset song
            self.actions.push(SongAction::Restart);
            self.actions.push(SongAction::Pause);
            self.actions.push(SongAction::SetPosition(0.0));
            self.actions.push(SongAction::SetRate(self.game_speed()));

            // self.song.set_rate(self.game_speed());
            // self.song.set_position(0.0);
            // if self.song.is_stopped() { self.song.play(true); }
            // self.song.pause();
        }

        self.completed = false;
        self.started = false;
        self.failed = false;
        self.lead_in_time = LEAD_IN_TIME / self.current_mods.get_speed();
        self.lead_in_timer = Instant::now();


        let playmode = self.gamemode.playmode().into_owned();
        self.map_diff = self.diff_provider.get_diff(
            &self.beatmap.get_beatmap_meta(), 
            &playmode, 
            &self.current_mods
        ).unwrap_or_default();

        let username = self.score.username.clone();
        self.score = IngameScore::new(Score::new(self.beatmap.hash(), username, playmode), true, false);
        self.score.speed = self.current_mods.speed;
        self.score_multiplier = 1.0;
        self.timing_points.reset();

        {
            // get all available mods for this playmode

            self.score.mods = self.current_mods.map_mods_to_thing(&self.gamemode_info);
            for m in &self.score.mods {
                self.score_multiplier *= m.score_multiplier;
            }

            // let ok_mods = ModManager::mods_for_playmode_as_hashmap(&playmode);

            // for i in self.current_mods.mods.iter() {
            //     let Some(m) = ok_mods.get(i) else { continue };
            //     self.score.mods.push((*m).into());
            //     self.score_multiplier *= m.score_multiplier;
            // }


            // self.score.mods = self.current_mods.mods.iter().map(ModDefinition::from).collect();
            // let playmode = self.gamemode.playmode();

            // // get all available mods for this playmode
            // let ok_mods = ModManager::mods_for_playmode_as_hashmap(&playmode);

            // // purge any non-gamemode mods, and get the score multiplier for mods that are enabled
            // self.score.mods.retain(|m| {
            //     if let Some(m) = ok_mods.get(m) {
            //         self.score_multiplier *= m.score_multiplier;
            //         true
            //     } else {
            //         false
            //     }
            // });
        }
        if self.score.replay.is_none() {
            self.score.replay = Some(Replay::new());
        }

        if !self.gameplay_mode.is_replay() {
            // only reset the replay if we arent replaying
            self.score.replay = Some(Replay::new());
            // self.replay = Replay::new();
            self.score.speed = self.current_mods.speed;
        } else {
            // if let Some(score) = &self.replay.score_data {
            //     self.score.username = score.username.clone();
            // }
        }

        // reset elements
        self.ui_elements.iter_mut().for_each(|e| e.reset_element());

        // re-add judgments to score
        for j in &self.judgments {
            self.score.judgments.insert(j.id.to_owned(), 0);
        }

        #[cfg(feature="gameplay")]
        if self.gameplay_mode.should_load_scores() {
            self.actions.push(GameAction::RefreshScores);
        }

    }
    pub fn fail(&mut self) {
        if self.failed || self.current_mods.has_nofail() || self.current_mods.has_autoplay() || self.gameplay_mode.is_preview() { return }
        self.failed = true;
        self.failed_time = self.time();
        debug!("failed");
    }

    pub fn combo_break(
        &mut self,
        settings: &Settings
    ) {
        // play hitsound
        if self.score.combo >= 20 && !self.gameplay_mode.is_preview() {
            let combobreak = Hitsound::new_simple("combobreak");
            // index of 1 because we want to try beatmap sounds
            self.hitsound_manager.play_sound_single(&combobreak, None, settings.get_effect_vol());
        }

        // reset combo to 0
        self.score.combo = 0;
    }

    /// the time set here will be properly applied next update call, as async is required
    pub fn jump_to_time(&mut self, time: f32, skip_intro: bool) {
        if skip_intro {
            self.lead_in_time = 0.0;
        }

        self.actions.push(SongAction::SetPosition(time));
        // self.song.set_position(time);

        self.pending_time_jump = Some(time);
    }

    pub fn on_complete(&mut self) {
        // make sure the cursor is visible
        // CursorManager::set_visible(true);
        self.actions.push(CursorAction::SetVisible(true));
        // undo any cursor override
        // CursorManager::set_ripple_override(None);
        self.actions.push(CursorAction::OverrideRippleRadius(None));

        #[cfg(feature="gameplay")]
        match &mut *self.gameplay_mode {
            GameplayModeInner::Spectator {
                buffered_score_frames,
                ..
            } => {
                // if we have a score frame we havent dealt with yet, its most likely the score frame sent once the map has ended
                if !buffered_score_frames.is_empty() {
                    self.score.score = buffered_score_frames.last().cloned().unwrap().1;
                }


                // let mut score_menu = ScoreMenu::new(&manager.score, manager.metadata.clone(), false);
                // score_menu.dont_close_on_back = true;
                // self.score_menu = Some(score_menu);
            }

            _ => {}
        }
    }


    /// using a getter for this since we dont want anything to directly change it
    pub fn get_mode(&self) -> &GameplayModeInner {
        &self.gameplay_mode
    }
    pub fn set_mode(&mut self, mode: impl Into<GameplayModeInner>) {
        let mode = mode.into();

        match &mode {
            GameplayModeInner::Normal => {
                // dont think there's anything to do for this one, since its the default
            }

            GameplayModeInner::Replaying { score, .. } => {
                // load speed from score
                let mods = ModManager {
                    mods: score.mods.iter().map(|m| m.name.clone()).collect(),
                    speed: score.speed,
                };
                self.current_mods = Arc::new(mods);

                self.score.mods = self.current_mods.map_mods_to_thing(&self.gamemode_info);
                self.score.username = score.username.clone();

                // if let Some(score) = &replay.score_data {

                //     self.current_mods = Arc::new(mods);
                //     *self.score.mods_mut() = self.current_mods.mods.clone();

                //     self.score.username = score.username.clone()
                // } else {
                //     self.score.username = "Unknown user".to_owned();
                // }
            }

            GameplayModeInner::Preview => {
                self.lead_in_time = 0.0;
                self.pending_time_jump = Some(self.time());

                let mut mods = self.current_mods.as_ref().clone();
                mods.add_mod(Autoplay);
                self.current_mods = Arc::new(mods);
            }

            // in a multi match
            #[cfg(feature="gameplay")]
            GameplayModeInner::Multiplayer { .. } => {
                // self.score_loader = None;
            }

            // handling spec
            #[cfg(feature="gameplay")]
            GameplayModeInner::Spectator { host_username, .. } => {
                self.score.username = host_username.clone();
                // self.replay.score_data.as_mut().unwrap().username = host_username.clone();
            }
        }

        self.gameplay_mode = Box::new(mode);
    }

    async fn fit_to_area(&mut self, bounds: Bounds) {
        // info!("fitting to area: {bounds:?}");
        self.fit_to_bounds = Some(bounds);
        self.gamemode.fit_to_area(bounds).await;
        #[cfg(feature="graphics")]
        self.animation.fit_to_area(bounds);
    }

    #[cfg(feature="graphics")]
    pub fn cleanup_textures(&mut self, skin_manager: &mut dyn SkinProvider) {
        // drop all texture references by dropping the gamemode
        // this should be fine since we shouldnt be re-using this gamemode at this time anyways
        self.gamemode = Box::new(NoMode);
        skin_manager.free_by_usage(SkinUsage::Beatmap);

        let path = self.beatmap.get_parent_dir().unwrap().to_string_lossy().to_string();
        skin_manager.free_by_source(TextureSource::Beatmap(path));
    }

    async fn handle_frame(
        &mut self,
        frame: ReplayAction,
        force: bool,
        force_time: Option<f32>,
        should_add: bool,
        settings: &Settings
    ) {
        // note to self: force is used when the frames are from the gamemode's update function
        if let ReplayAction::Press(KeyPress::SkipIntro) = frame {
            if let Some(mut time) = self.gamemode.skip_intro(self.time()) {

                // really not sure whats happening here lol
                if self.lead_in_time > 0.0 {
                    if time > self.lead_in_time {
                        time -= self.lead_in_time - 0.01;
                        self.lead_in_time = 0.01;
                    }
                }

                self.actions.push(SongAction::SetPosition(time));
            }

            // more to do?
            return;
        }

        let add_frames = !(self.current_mods.has_autoplay() || self.gameplay_mode.is_replay());

        if force || add_frames {
            match frame {
                ReplayAction::Press(k) => self.key_counter.key_down(k),
                ReplayAction::Release(k) => self.key_counter.key_up(k),
                _ => {}
            }

            let time = force_time.unwrap_or_else(|| self.time());
            let frame = ReplayFrame::new(time, frame);

            let mut state = create_update_state!(self, self.time(), settings);
            self.gamemode.handle_replay_frame(frame, &mut state).await;

            for action in state.actions.take() {
                self.handle_gamemode_action(action, settings).await;
            }

            if add_frames && should_add {
                self.score.replay.as_mut().map(|r| r.frames.push(frame));
                #[cfg(feature="gameplay")]
                self.outgoing_spectator_frame(
                    SpectatorFrame::new(time, SpectatorAction::ReplayAction { action: frame.action }),
                );
            }
        }
    }

}

// Input Handlers
#[cfg(feature="graphics")]
impl GameplayManager {

    pub async fn key_down(
        &mut self, 
        key_input: KeyInput, 
        mods: KeyModifiers,
        settings: &Settings,
    ) {
        let Some(key) = key_input.as_key() else { return };

        if (self.gameplay_mode.is_replay() || self.current_mods.has_autoplay()) && !self.gameplay_mode.is_preview() {
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
        if self.should_skip_input() { return }


        if key == Key::Escape {
            if self.can_pause() {
                self.should_pause = true;
            } else if let GameplayModeInner::Multiplayer { last_escape_press, .. } = &mut *self.gameplay_mode {
                if last_escape_press.elapsed_and_reset() < 1_000.0 {
                    self.actions.push(MultiplayerAction::ExitMultiplayer);
                    return;
                } else {
                    self.actions.push(Notification::new_text("Press escape again to quit the lobby", Color::BLUE, 3_000.0));
                }
            }
        }



        // #[cfg(feature="graphics")]
        // if let Some(ui_editor) = &mut self.ui_editor {
        //     ui_editor.on_key_press(key_input, &mods, &mut ()).await;
        //     if key == Key::F9 {
        //         ui_editor.should_close = true;

        //         // re-disable cursor
        //         if self.should_hide_cursor() {
        //             self.actions.push(CursorAction::SetVisible(false));
        //             // CursorManager::set_visible(false);
        //         }
        //     }
        // } else if key == Key::F9 {
        //     self.ui_editor = Some(GameUIEditorDialog::new(std::mem::take(&mut self.ui_elements)));
        //     self.ui_changed = true;

        //     // start autoplay
        //     if !self.current_mods.has_autoplay() {
        //         let mut new_mods = self.current_mods.as_ref().clone();
        //         new_mods.add_mod(Autoplay);
        //         self.current_mods = Arc::new(new_mods);
        //     }

        //     self.actions.push(CursorAction::SetVisible(true));
        //     // CursorManager::set_visible(true);
        // }


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
        let Some(frame) = (if key == Key::Space {
            Some(ReplayAction::Press(KeyPress::SkipIntro))
        } else {
            self.gamemode.key_down(key).await
        }) else { return };

        self.handle_frame(
            frame, 
            false, 
            None, 
            true,
            settings,
        ).await;
    }
    pub async fn key_up(
        &mut self, 
        key_input: KeyInput,
        settings: &Settings,
    ) {
        if self.should_skip_input() { return }
        let Some(key) = key_input.as_key() else { return };

        // check map restart key
        if key == self.common_game_settings.map_restart_key {
            self.restart_key_hold_start = None;
            return;
        }

        let Some(frame) = self.gamemode.key_up(key).await else { return };
        self.handle_frame(
            frame, 
            false, 
            None, 
            true,
            settings,
        ).await;
    }
    pub async fn on_text(
        &mut self, 
        text: &String, 
        mods: &KeyModifiers,
        settings: &Settings,
    ) {
        if self.should_skip_input() { return }
        let Some(frame) = self.gamemode.on_text(text, mods).await else { return };
        self.handle_frame(
            frame, 
            false, 
            None, 
            true,
            settings,
        ).await;
    }


    pub async fn mouse_move(
        &mut self, 
        pos: Vector2,
        settings: &Settings,
    ) {
        // #[cfg(feature="graphics")]
        // if let Some(ui_editor) = &mut self.ui_editor {
        //     ui_editor.on_mouse_move(pos, &mut ()).await;
        // }

        // !self.should_handle_input() { return }

        let Some(frame) = self.gamemode.mouse_move(pos).await else { return };
        self.handle_frame(
            frame, 
            false, 
            None, 
            true,
            settings,
        ).await;
    }
    pub async fn mouse_down(
        &mut self, 
        btn: MouseButton,
        settings: &Settings,
    ) {
        // #[cfg(feature="graphics")]
        // if let Some(ui_editor) = &mut self.ui_editor {
        //     ui_editor.on_mouse_down(Vector2::ZERO, btn, &KeyModifiers::default(), &mut ()).await;
        //     return
        // }

        if self.should_skip_input() { return }
        let Some(frame) = self.gamemode.mouse_down(btn).await else { return };
        self.handle_frame(
            frame, 
            false, 
            None, 
            true,
            settings,
        ).await;
    }
    pub async fn mouse_up(
        &mut self, 
        btn: MouseButton,
        settings: &Settings,
    ) {
        // #[cfg(feature="graphics")]
        // if let Some(ui_editor) = &mut self.ui_editor {
        //     ui_editor.on_mouse_up(Vector2::ZERO, btn, &KeyModifiers::default(), &mut ()).await;
        //     return;
        // }

        if self.should_skip_input() { return }
        let Some(frame) = self.gamemode.mouse_up(btn).await else { return };
        self.handle_frame(
            frame, 
            false, 
            None, 
            true,
            settings,
        ).await;
    }
    pub async fn mouse_scroll(
        &mut self, 
        delta: f32,
        settings: &Settings,
    ) {
        // #[cfg(feature="graphics")]
        // if let Some(ui_editor) = &mut self.ui_editor {
        //     ui_editor.on_mouse_scroll(delta, &mut ()).await;
        // }

        if self.should_skip_input() { return }
        let Some(frame) = self.gamemode.mouse_scroll(delta).await else { return };
        self.handle_frame(
            frame, 
            false, 
            None, 
            true,
            settings,
        ).await;
    }


    pub async fn controller_press(
        &mut self, 
        c: &GamepadInfo, 
        btn: ControllerButton,
        settings: &Settings,
    ) {
        if self.should_skip_input() { return }
        let Some(frame) = self.gamemode.controller_press(c, btn).await else { return };
        self.handle_frame(
            frame, 
            false, 
            None, 
            true,
            settings,
        ).await;
    }
    pub async fn controller_release(
        &mut self, 
        c: &GamepadInfo, 
        btn: ControllerButton,
        settings: &Settings,
    ) {
        if self.should_skip_input() { return }
        let Some(frame) = self.gamemode.controller_release(c, btn).await else { return };
        self.handle_frame(
            frame, 
            false, 
            None, 
            true,
            settings,
        ).await;
    }
    pub async fn controller_axis(
        &mut self, 
        c: &GamepadInfo, 
        axis_data: HashMap<Axis, (bool, f32)>,
        settings: &Settings,
    ) {
        if self.should_skip_input() { return }
        let Some(frame) = self.gamemode.controller_axis(c, axis_data).await else { return };
        self.handle_frame(
            frame, 
            false, 
            None, 
            true,
            settings,
        ).await;
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


    #[cfg(feature="graphics")]
    pub async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.window_size = window_size.clone();
        self.gamemode.window_size_changed(window_size).await;
        self.animation.window_size_changed(self.window_size.0);

        // // TODO: relocate ui elements properly
        // if let Some(mut editor) = std::mem::take(&mut self.ui_editor) {
        //     self.init_ui().await;
        //     editor.elements = std::mem::take(&mut self.ui_elements);
        //     self.ui_editor = Some(editor);
        // } else {
        //     self.ui_elements.clear();
        //     self.init_ui().await;
        // }
    }
}

// other misc stuff that isnt touched often and i just wanted it out of the way
impl GameplayManager {
    #[cfg(feature="gameplay")]
    fn should_skip_input(&self) -> bool {
        // never skip input for multi, because you can keep playing if you failed
        if self.gameplay_mode.is_multi() { return false }
        self.failed || self.gameplay_mode.skip_input()
    }

    pub async fn increment_offset(&mut self, delta:f32) {
        let time = self.time();
        self.beatmap_preferences.audio_offset += delta;
        self.center_text_helper.set_value(format!("Offset: {:.2}ms", self.beatmap_preferences.audio_offset), time);

        // update the beatmap offset
        let new_prefs = self.beatmap_preferences.clone();
        let hash = self.beatmap.hash();
        tokio::spawn(async move { Database::save_beatmap_prefs(hash, &new_prefs); });
    }

    pub async fn increment_global_offset(&mut self, delta:f32) {
        let time = self.time();
        // let mut settings = Settings::get_mut();
        // settings.global_offset += delta;
        self.global_offset += delta;
        self.center_text_helper.set_value(format!("Global Offset: {:.2}ms", self.global_offset), time);
    }

    pub async fn force_update_settings(&mut self, settings: &Settings) {
        // self.settings.update();
        self.gamemode.force_update_settings(settings).await;
        self.global_offset = settings.global_offset;
    }

    #[cfg(feature="graphics")]
    pub async fn reload_skin(
        &mut self, 
        skin_manager: &mut dyn SkinProvider,
        settings: &Settings,
    ) {
        let parent_folder = self.beatmap.get_parent_dir().unwrap().to_string_lossy().to_string();
        let source = self.gamemode.reload_skin(&parent_folder, skin_manager).await;
        self.hitsound_manager.reload_skin(settings, &mut self.actions).await;

        #[cfg(feature="storyboards")]
        if let Some(anim) = self.beatmap.get_animation(skin_manager).await {
            self.animation = anim;
        }
        // self.animation = beatmap.get_animation().await.unwrap_or_else(|| Box::new(EmptyAnimation));

        for i in self.ui_elements.iter_mut() {
            i.reload_skin(&source, skin_manager).await;
        }
    }

    fn in_break(&self) -> bool {
        let time = self.time();
        #[allow(irrefutable_let_patterns)]
        self.events.iter().find(|f| if let IngameEvent::Break { start, end } = f { time >= *start && time < *end } else { false }).is_some()
    }

}

// Spectator Stuff
#[cfg(feature="gameplay")]
impl GameplayManager {
    pub fn outgoing_spectator_frame(
        &mut self, 
        frame: SpectatorFrame,
    ) {
        if !self.gameplay_mode.should_send_spec_frames() { return }
        self.frame_sender.send_spec_frames(vec![frame], false);
    }
    pub fn outgoing_spectator_frame_force(
        &mut self, 
        frame: SpectatorFrame,
    ) {
        if !self.gameplay_mode.should_send_spec_frames() { return }
        self.frame_sender.send_spec_frames(vec![frame], true);
    }
}

pub trait GameplayManagerOnline: Send + Sync {
    fn send_spec_frames(&mut self, frames: Vec<SpectatorFrame>, force: bool);
    fn get_pending_frames(&mut self) -> Vec<SpectatorFrame>;
    fn update_lobby_score(&mut self, score: Score);
    fn our_spectator_list(&mut self) -> Option<SpectatorList>;
}
struct DummyOnlineThing;
impl GameplayManagerOnline for DummyOnlineThing {
    fn send_spec_frames(&mut self, _frames: Vec<SpectatorFrame>, _force: bool) {
        // unimplemented!("DummyOnlineThing")
    }

    fn get_pending_frames(&mut self) -> Vec<SpectatorFrame> {
        // unimplemented!("DummyOnlineThing")
        Vec::new()
    }

    fn update_lobby_score(&mut self, _score: Score) {
        // unimplemented!("DummyOnlineThing")
    }

    fn our_spectator_list(&mut self) -> Option<SpectatorList> {
        // unimplemented!("DummyOnlineThing")
        None
    }
}

pub trait DifficultyProvider: Send + Sync {
    fn get_diff(&mut self, map: &Arc<BeatmapMeta>, playmode: &String, mods: &ModManager) -> TatakuResult<f32>;
}
struct DummyDiffProvider;
impl DifficultyProvider for DummyDiffProvider {
    fn get_diff(&mut self, _: &Arc<BeatmapMeta>, _: &String, _: &ModManager) -> TatakuResult<f32> {
        Ok(-1.0)
    }
}


impl Drop for GameplayManager {
    fn drop(&mut self) {
        if self.gamemode.playmode() != "none" {
            error!("gameplay manager dropped without cleaning up textures !!!!!!!!!!!");
        }
    }
}



#[derive(Default)]
pub struct GameplaySpectatorInfo {
    /// when was the last time the score was synchronized?
    pub last_score_sync: f32,

    /// who is currently spectating us?
    pub spectators: SpectatorList
}


/// What gameplay method should we use for this gameplay manager?
#[derive(Clone, Debug, Default)]
pub enum GameplayModeInner {
    /// Just regular gameplay
    #[default]
    Normal,

    /// This manager is handling gameplay preview
    Preview,

    /// This manager is watching a replay
    Replaying {
        /// What score+replay are we watching?
        score: Score,

        /// What frame index are we at?
        current_frame: usize,
    },

    #[cfg(feature="gameplay")]
    /// We're handling spectating someone
    Spectator {
        /// What is the current spec state
        state: SpectatorState,
        /// List of buffered spectator frames
        frames: VecDeque<SpectatorFrame>,

        host_id: u32,
        host_username: String,

        /// list of buffered replay frames
        replay_frames: Vec<ReplayFrame>,
        /// what replay frame are we on
        current_frame: usize,

        /// Up to what time do we have data for?
        ///
        /// ie, up to what time we can show gameplay
        good_until: f32,

        /// List of (id,username) for other spectators
        spectators: HashMap<u32, String>,

        /// List of score frames to help sync the host score with our score
        ///
        /// TODO: ideally this wouldnt be necessary though
        buffered_score_frames: Vec<(f32, Score)>,
    },

    #[cfg(feature="gameplay")]
    /// The player is in a multiplayer match
    Multiplayer {
        /// when was escape pressed last
        last_escape_press: Instant,
        score_send_timer: Instant,
    },
}
impl GameplayModeInner {

    // convenience fns
    pub fn is_preview(&self) -> bool { if let &Self::Preview = self { true } else { false } }
    #[cfg(feature="gameplay")]
    pub fn is_multi(&self) -> bool { if let &Self::Multiplayer {..} = self { true } else { false } }
    pub fn is_replay(&self) -> bool { if let &Self::Replaying {..} = self { true } else { false } }

    #[cfg(feature="gameplay")]
    fn should_load_scores(&self) -> bool {
        match self {
            Self::Normal | Self::Spectator {..} | Self::Replaying {..} => true,
            Self::Multiplayer {..} | Self::Preview {..} => false,
        }
    }

    #[cfg(feature="gameplay")]
    fn should_send_spec_frames(&self) -> bool {
        match self {
            // send spec frames for normal gameplay and multi, not for anything else
            Self::Normal | Self::Multiplayer {..} => true,
            Self::Replaying {..} | Self::Spectator {..} | Self::Preview {..} => false,
        }
    }

    #[cfg(feature="gameplay")]
    fn skip_input(&self) -> bool {
        match self {
            Self::Replaying { .. } | Self::Preview { .. } | Self::Spectator { .. } => true,
            _ => false,
        }
    }
}

impl From<GameplayMode> for GameplayModeInner {
    fn from(value: GameplayMode) -> Self {
        match value {
            GameplayMode::Normal => Self::Normal,
            GameplayMode::Preview => Self::Preview,
            GameplayMode::Multiplayer => Self::Multiplayer { last_escape_press: Instant::now(), score_send_timer: Instant::now() },
            GameplayMode::Replay(score) => Self::Replaying { score, current_frame: 0 },
            GameplayMode::Spectator { host_id, host_username, pending_frames, spectators } => Self::Spectator {
                state: SpectatorState::None,
                frames: pending_frames,
                host_id,
                host_username,
                replay_frames: Vec::new(),
                current_frame: 0,
                good_until: 0.0,
                spectators,
                buffered_score_frames: Vec::new()
            }
        }
    }
}



// TODO: rename this
pub struct GameplayStateForDraw<'a> {
    pub time: f32,
    pub gameplay_mode: &'a Box<GameplayModeInner>,
    pub current_timing_point: &'a TimingPoint,
    pub mods: &'a ModManager,
    pub score: &'a IngameScore,
}

//TODO: rename this please god
pub struct GameplayStateForUpdate<'a> {
    /// current map time
    pub time: f32,

    /// current game speed
    pub game_speed: f32,

    /// private so we dont set this to true accidentally
    /// need to use an action instead
    completed: bool,

    /// current mods
    pub mods: &'a ModManager,

    /// the current timing point
    pub current_timing_point: &'a TimingPoint,

    /// the current gameplay mode
    pub gameplay_mode: &'a Box<GameplayModeInner>,

    /// our current score
    pub score: &'a IngameScore,

    // all timing points
    pub timing_points: &'a TimingPointHelper,

    /// list of actions to be performed
    actions: Vec<GamemodeAction>,

    pub settings: &'a Settings,
}
impl<'a> GameplayStateForUpdate<'a> {
    /// does the manager believe the map has been completed?
    pub fn complete(&self) -> bool { self.completed }

    pub fn add_action(&mut self, action: impl Into<GamemodeAction>) {
        self.actions.push(action.into());
    }
    pub fn add_replay_action(&mut self, action: ReplayAction) {
        self.actions.push(GamemodeAction::ReplayAction(ReplayFrame::new(self.time, action)));
    }


    /*
    // fn add_timing(&mut self, time: f32, note_time: f32) {
    //     let diff = (time - note_time) / HIT_DIFF_FACTOR;
    //     self.add_stat(HitVarianceStat, diff);
    //     // $self.score.hit_timings.push(diff);
    //     self.hitbar_timings.push((time, diff));
    // }

    // /// add judgment, affects health and score, but not hit timings
    // pub async fn add_judgment(&mut self, judgment: &HitJudgment) {
    //     // increment judgment, if applicable
    //     if let Some(count) = self.score.judgments.get_mut(judgment.id) {
    //         *count += 1;
    //     }

    //     // do score
    //     let combo_mult = (self.score.combo as f32 * self.score_multiplier).floor() as u16;
    //     let score = judgment.base_score_value;

    //     let score = match judgment.combo_multiplier {
    //         ComboMultiplier::None => score,
    //         ComboMultiplier::Custom(mult) => (score as f32 * mult) as i32,
    //         ComboMultiplier::Linear { combo, multiplier, combo_cap } => {
    //             let combo_mult = combo_cap.map(|cap| combo_mult.min(cap)).unwrap_or(combo_mult);
    //             let times = (combo_mult % combo).max(1) as f32;

    //             (score as f32 * (multiplier * times)) as i32
    //         }
    //     };

    //     match score {
    //         score @ i32::MIN..=0 => self.score.score.score -= score.abs() as u64,
    //         score @ 1.. => self.score.score.score += score as u64,
    //     }

    //     // do combo
    //     match judgment.affects_combo {
    //         AffectsCombo::Increment => {
    //             self.score.combo += 1;
    //             self.score.max_combo = self.score.max_combo.max(self.score.combo);
    //         },
    //         AffectsCombo::Reset => self.actions.push(GamemodeAction::ComboBreak), // self.combo_break().await,
    //         AffectsCombo::Ignore => {},
    //     }

    //     // do health
    //     (self.health.do_health.clone())(&mut self.health, judgment, &self.score);

    //     // check health
    //     if !self.health.check_fail_at_end && self.health.is_dead() {
    //         self.actions.push(GamemodeAction::FailGame);
    //         // self.fail()
    //     }

    //     // check sd/pf mods
    //     //TODO: if this happens, change the judgment to a miss
    //     if self.current_mods.has_sudden_death() && judgment.fails_sudden_death {
    //         self.actions.push(GamemodeAction::FailGame);
    //         // self.fail()
    //     }
    //     if self.current_mods.has_perfect() && judgment.fails_perfect {
    //         self.actions.push(GamemodeAction::FailGame);
    //         // self.fail()
    //     }

    // }

    // */

    /// check and add to hit timings if found
    pub async fn check_judgment<'j>(
        &mut self,
        windows: &'j Vec<(HitJudgment, Range<f32>)>,
        time: f32,
        note_time: f32
    ) -> Option<&'j HitJudgment> {
        if let Some(hj) = self.check_judgment_only(windows, time, note_time) {
            self.actions.push(GamemodeAction::AddJudgment(*hj));
            self.actions.push(GamemodeAction::AddTiming { hit_time: time, note_time });
            // self.add_judgment(hj).await;
            // self.add_timing(time, note_time);
            // add_timing!(self, time, note_time);

            // return the hit judgment we got
            Some(hj)
        } else {
            None
        }

        // let diff = (time - note_time).abs() / HIT_DIFF_FACTOR / self.game_speed();
        // for (hj, window) in windows.iter() {
        //     if window.contains(&diff) {
        //         self.add_judgment(hj).await;
        //         self.add_timing(time, note_time);
        //         // add_timing!(self, time, note_time);

        //         // return the hit judgment we got
        //         return Some(hj)
        //     }
        // }

        // None
    }

    pub async fn check_judgment_condition<'j>(
        &mut self,
        windows: &'j Vec<(HitJudgment, Range<f32>)>,
        time: f32,
        note_time: f32,
        cond: impl Fn() -> bool,
        if_bad: &'j HitJudgment
    ) -> Option<&'j HitJudgment> {
        if let Some(hj) = self.check_judgment_only(windows, time, note_time) {
            if cond() {
                self.actions.push(GamemodeAction::AddJudgment(*hj));
                self.actions.push(GamemodeAction::AddTiming { hit_time: time, note_time });
                // self.add_judgment(hj).await;
                // self.add_timing(time, note_time);
                // add_timing!(self, time, note_time);
                // return the hit judgment we got
                Some(hj)
            } else {
                self.actions.push(GamemodeAction::AddJudgment(*if_bad));
                // self.add_judgment(if_bad).await;
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
    pub fn check_judgment_only<'j>(
        &self,
        windows: &'j Vec<(HitJudgment, Range<f32>)>,
        time: f32,
        note_time: f32
    ) -> Option<&'j HitJudgment> {
        let diff = (time - note_time).abs() / HIT_DIFF_FACTOR / self.game_speed;
        for (hj, window) in windows.iter() {
            if window.contains(&diff) {
                // return the hit judgment we got
                return Some(hj)
            }
        }

        None
    }

    pub fn add_judgment(&mut self, judgment: HitJudgment) {
        self.actions.push(GamemodeAction::AddJudgment(judgment));
    }
    pub fn add_indicator(&mut self, indicator: impl JudgementIndicator + 'static) {
        self.actions.push(GamemodeAction::AddIndicator(Box::new(indicator)));
    }
    pub fn add_stat(&mut self, stat: GameModeStat, value: f32) {
        self.actions.push(GamemodeAction::AddStat { stat, value });
    }

    pub fn play_note_sound(&mut self, hitsounds: Vec<Hitsound>) {
        self.actions.push(GamemodeAction::PlayHitsounds(hitsounds));
    }
}


#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum SpectatorState {
    None, // Default
    Buffering, // waiting for data
    Watching, // host playing
    Paused, // host paused
    MapChanging, // host is changing map
}

#[derive(Debug)]
pub enum SpectatorManagerAction {
    QuitSpec,
}
