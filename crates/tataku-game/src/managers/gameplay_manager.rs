use crate::prelude::*;

/// how long should center text be drawn for?
const CENTER_TEXT_DRAW_TIME:f32 = 2_000.0;

/// ms between spectator score sync packets
const SPECTATOR_SCORE_SYNC_INTERVAL:f32 = 1000.0;


pub const SCORE_SEND_TIME:f32 = 1_000.0;

/// how long of a buffer should we have? (ms)
pub const SPECTATOR_BUFFER_OK_DURATION:f32 = 500.0;

macro_rules! create_update_state {
    ($self: ident, $time: expr, $settings: expr) => {
        GameplayUpdateShell {
            time: $time,
            game_speed: $self.game_speed(),
            completed: $self.completed,

            mods: &$self.current_mods,
            current_timing_point: $self.timing_points.timing_point(),
            timing_points: &$self.timing_points,
            gameplay_mode: &$self.gameplay_mode,
            score: &$self.score,
            actions: Vec::new(),
            settings: $settings,
            action_queue: &mut $self.actions,
            window_size: $self.window_size,
        }
    }
}

pub struct GameplayManager {
    pub id: GameplayId,
    pub actions: ActionQueue,

    pub beatmap: Beatmap,
    pub metadata: Arc<BeatmapMeta>,
    pub gamemode: Box<dyn GameMode>,
    // pub gamemode_info: GameModeInfo,
    pub gamemode_properties: GameModeProperties,

    pub current_mods: Arc<ModManager>,
    pub beatmap_preferences: BeatmapPreferences,

    pub gameplay_mode: Box<GameplayModeInner>,
    gameplay_actions: Vec<GameplayAction>,


    pub score: IngameScore,
    pub score_multiplier: f32,

    pub health: Box<dyn HealthManager>,
    pub judgments: Vec<HitJudgment>,
    pub key_counter: KeyCounter,
    ui_elements: Vec<GameplayWidgetContainer>,

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
    pub lead_in_timer: TatakuInstant,
    
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
    restart_key_hold_start: Option<TatakuInstant>,

    pub timing_points: TimingPointHelper,
    pub hitsound_manager: HitsoundManager,

    /// center text helper (ie, for offset and global offset)
    pub center_text_helper: CenteredTextHelper,

    /// (map.time, note.time - hit.time)
    pub hitbar_timings: Vec<(f32, f32)>,

    /// list of judgement indicators to draw
    pub judgement_indicators: Vec<Box<dyn JudgementIndicator>>,

    pub common_game_settings: Arc<CommonGameplaySettings>,
    window_size: Vector2,
    fit_to_bounds: Option<Bounds>,

    // spectator info
    pub spectator_info: GameplaySpectatorInfo,

    frame_sender: Box<dyn GameplayManagerOnline>,

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
        let properties = gamemode.properties();
        let playmode = properties.playmode();
        let metadata = beatmap.get_beatmap_meta();

        if current_mods.get_speed() == 0.0 { current_mods.set_speed(1.0); }
        let current_mods = Arc::new(current_mods);

        let mut score = Score::new(beatmap.hash(), settings.username.clone(), playmode.to_string());
        score.speed = current_mods.speed;

        
        let mut actions = ActionQueue::new();
        let mut hitsound_manager = HitsoundManager::new(properties.audio_prefix.clone());
        hitsound_manager.init(&metadata, &mut actions, settings).await;

        // make sure the gamemode has the correct mods applied
        gamemode.apply_mods(current_mods.clone()).await;

        let mut gm = Self {
            id: Arc::new(u32::MAX),
            actions,
            frame_sender: Box::new(DummyOnlineThing),
            
            timing_points: TimingPointHelper::new_from_beatmap(&beatmap),
            // hitsound_cache,
            current_mods,
            health: Box::new(DefaultHealthManager::new()),
            key_counter: KeyCounter::new(&properties.keys),

            judgments: properties.info.judgments.to_vec(),
            score: IngameScore::new(score, true, false),

            #[cfg(feature="graphics")]
            animation: Box::new(EmptyAnimation),

            hitsound_manager,
            events: beatmap.get_events(),
            // song,

            lead_in_time: LEAD_IN_TIME,
            lead_in_timer: TatakuInstant::now(),
            end_time: properties.end_time,
            global_offset: settings.global_offset,

            center_text_helper: CenteredTextHelper::new(CENTER_TEXT_DRAW_TIME).await,
            beatmap_preferences: Database::get_beatmap_prefs(metadata.beatmap_hash).await,

            common_game_settings: Arc::new(settings.common_game_settings.clone()),

            metadata,
            beatmap,
            gamemode,
            gamemode_properties: properties,

            scores_loaded: false,
            score_list: Vec::new(),
            // score_loader, values: &mut dyn Reflec
            window_size: Vector2::ZERO,
            start_time: chrono::Utc::now().timestamp(),

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
        };

        gm.init_ui().await;

        gm
    }

    #[cfg(feature="graphics")]
    async fn init_ui(
        &mut self
    ) {
        let layouts = std::fs::read("ui_layouts.json").ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();


        let info = self.gamemode_properties.info;

        // TODO: would be nice to make widgets from all gamemodes available (don-chan in osu!?)
        let widgets = DEFAULT_GAMEPLAY_WIDGETS
            .iter()
            .chain(info.available_widgets)
            .cloned()
            .collect::<Vec<_>>();

        let mut loader = DefaultUiElementLoader::new(
            self.gamemode_properties.playmode(),
            layouts,
            widgets,

            *info,
            self.common_game_settings.clone()
        );
        // if self.ui_editor.is_some() { return }

        for i in DEFAULT_GAMEPLAY_WIDGETS.iter().map(|i| i.name) {
            loader.load(i);
        }

        // Anything in the gamemode itself
        self.gamemode.build_widgets(&mut loader).await;

        // update every ui element so they're all initialized
        loader.elements
            .iter_mut()
            .for_each(|e| e.update(self));

        // update our list
        self.ui_elements = loader.elements;

        // layout will be performed on game start (and skin load)
    }

    fn layout_ui(&mut self) {
        if let Err(e) = GameplayWidgetContainer::layout(
            &mut self.ui_elements,
            self.gamemode.get_playfield().bounds,
            self.window_size
        ) {
            error!("error laying out ui elements! {e:?}")
        }
    }

    pub fn skip_intro(&mut self) {
        let Some(mut time) = self.gamemode.skip_intro(self.time()) else { return };

        // really not sure whats happening here lol
        if self.lead_in_time > 0.0 && time > self.lead_in_time {
            time -= self.lead_in_time - 0.01;
            self.lead_in_time = 0.01;
        }

        self.actions.push(SongAction::SetPosition(time));
    }
}

// getters, setters, properties
impl GameplayManager {

    /// is this game pausable
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

    pub fn should_hide_cursor(&self) -> bool {
        if self.gameplay_mode.is_preview()
        || self.gameplay_mode.is_replay() 
        || self.current_mods.has_autoplay() {
            false
        } else {
            !self.gamemode_properties.show_cursor
        }
    }

    pub fn should_save_score(&self) -> bool {
        !(self.gameplay_mode.is_replay() || self.current_mods.has_autoplay() || self.ui_changed)
    }


    pub fn update_difficulty(&mut self, provider: &mut dyn DifficultyProvider) {
        self.map_diff = provider.get_diff(
            &self.beatmap.get_beatmap_meta(), 
            self.gamemode_properties.playmode(), 
            &self.current_mods
        ).unwrap_or_default();

        debug!("Updated diff: {}", self.map_diff);
    }

    fn set_online(&mut self, sender: Box<dyn GameplayManagerOnline>) {
        self.frame_sender = sender;
    }
}

// Events and States
impl GameplayManager {
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
            if self.gameplay_mode.is_multi() {
                self.actions.push(LobbyAction::SendSkipRequest);
            } else {
                self.skip_intro();
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
                if let Some(r) = self.score.replay.as_mut() { r.frames.push(frame) }
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
    ) -> bool {
        if key_input.repeat { return false }
        let Some(key) = key_input.as_key() else { return false };

        if (self.gameplay_mode.is_replay() || self.current_mods.has_autoplay()) && !self.gameplay_mode.is_preview() {
            // check replay-only keys
            if key == Key::Escape {
                self.started = false;
                self.completed = true;
                return true;
            }
        }

        // check map restart key
        if key == self.common_game_settings.map_restart_key && !self.gameplay_mode.is_multi() {
            self.restart_key_hold_start = Some(TatakuInstant::now());
            return true;
        }

        if self.failed && key == Key::Escape && !self.gameplay_mode.is_multi() {
            // set the failed time to negative, so it triggers the end
            self.failed_time = -1000.0;
        }

        if self.should_skip_input() { return false }


        if key == Key::Escape {
            if self.can_pause() {
                self.should_pause = true;
            } else if let GameplayModeInner::Multiplayer { last_escape_press, .. } = &mut *self.gameplay_mode {
                if last_escape_press.elapsed_and_reset() < 3_000.0 {
                    self.actions.push(MultiplayerAction::ExitMultiplayer);
                } else {
                    self.actions.push(Notification::new_text("Press escape again to quit the lobby", Color::RED, 3_000.0));
                }
                
                return true;
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
                return true;
            }
        } else {
            if key == self.common_game_settings.key_offset_up { 
                self.increment_offset(5.0).await;
                return true;
            }
            if key == self.common_game_settings.key_offset_down { 
                self.increment_offset(-5.0).await; 
                return true;
            }
        }


        // skip intro
        if key == Key::Space {
            self.handle_frame(
                ReplayAction::Press(KeyPress::SkipIntro), 
                false, 
                None, 
                true,
                settings,
            ).await;
            
            return true;
        }

        false
    }


    #[cfg(feature="graphics")]
    pub async fn window_size_changed(&mut self, window_size: Vector2) {
        self.window_size = window_size;
        if self.fit_to_bounds.is_none() {
            self.gamemode.set_bounds(Bounds::new(Vector2::ZERO, window_size), true);
        }

        if self.animation.use_gamemode_playfield(self.gamemode_properties.info) {
            self.animation.fit_to_area(self.gamemode.get_playfield());
        } else {
            self.animation.window_size_changed(window_size);
        }

        self.layout_ui();
    }


    pub async fn handle_input(&mut self, input: InputEvent, settings: &Settings) {
        // #[cfg(feature="graphics")]
        // if let Some(ui_editor) = &mut self.ui_editor {
        //     if ui_editor.handle_input(&input).await { return}
        // }

        match &input.event {
            InputType::KeyPress(key_input) => {
                if self.key_down(key_input.clone(), input.key_mods, settings).await {
                    return 
                }
            }
            InputType::KeyRelease(key_input) => {
                let Some(key) = key_input.as_key() else { return };

                // check map restart key
                if key == self.common_game_settings.map_restart_key {
                    self.restart_key_hold_start = None;
                    return;
                }
            }
            
            _ => {}
        }

        if self.should_skip_input() { return }

        let Some(frame) = self.gamemode.handle_input(input).await else { return };
        self.handle_frame(
            frame, 
            false, 
            None, 
            true,
            settings,
        ).await;

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

    pub async fn increment_offset(&mut self, delta: f32) {
        let time = self.time();
        self.beatmap_preferences.audio_offset += delta;
        self.center_text_helper.set_value(format!("Offset: {:.2}ms", self.beatmap_preferences.audio_offset), time);

        // update the beatmap offset
        let new_prefs = self.beatmap_preferences.clone();
        let hash = self.beatmap.hash();
        tokio::spawn(async move { Database::save_beatmap_prefs(hash, &new_prefs); });
    }

    pub async fn increment_global_offset(&mut self, delta: f32) {
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

    fn in_break(&self) -> bool {
        let time = self.time();
        #[allow(irrefutable_let_patterns)]
        self.events.iter().any(|f| if let IngameEvent::Break { start, end } = f { time >= *start && time < *end } else { false })
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

#[async_trait]
impl GameplayManagerTrait for GameplayManager {
    fn end_time(&self) -> f32 { self.end_time }

    fn score(&self) -> &IngameScore { &self.score }
    fn score_mut(&mut self) -> &mut IngameScore { &mut self.score }
    fn mods(&self) -> &ModManager { &self.current_mods }
    fn metadata(&self) -> &BeatmapMeta { &self.metadata }
    fn key_counter(&self) -> &KeyCounter { &self.key_counter }
    fn spectators(&mut self) -> &mut SpectatorList { &mut self.spectator_info.spectators }
    fn judgments(&self) -> &Vec<HitJudgment> { &self.judgments }
    fn health(&self) -> &dyn HealthManager { &*self.health }
    fn hitbar_timings(&self) -> Vec<(f32, f32)> { self.hitbar_timings.clone() }
    fn timing_points(&self) -> &TimingPointHelper { &self.timing_points }

    fn properties(&self) -> &GameModeProperties { &self.gamemode_properties }

    fn bounds(&self) -> Bounds {
        self.fit_to_bounds.unwrap_or(Bounds::new(Vector2::ZERO, self.window_size))
    }


    async fn apply_mods(&mut self, mut mods: ModManager) {
        if self.gameplay_mode.is_preview() {
            mods.add_mod(Autoplay);
        }

        self.current_mods = Arc::new(mods);
        self.gamemode.apply_mods(self.current_mods.clone()).await;
    }

    async fn update(&mut self, values: &mut dyn Reflect) -> Vec<TatakuAction> {
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

            let mut state = create_update_state!(self, time, &settings);
            self.gamemode.time_jump(time, &mut state).await;
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
            self.lead_in_timer = TatakuInstant::now();
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
            self.score_list = (*scores_list).clone();
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
        let mut state = create_update_state!(self, time, &settings);


        self.gamemode.update(&mut state).await;
        for action in state.actions {
            self.handle_gamemode_action(action, &settings).await;
        }
        //.into_iter().map(|f| ReplayFrame::new(time, f));



        // if self.lead_in_time == 0.0 && values.get_bool("song.stopped").unwrap_or_default() {
        //     debug!("Song over, saying map is complete");
        //     self.completed = true;
        // }

        // update score stuff now that gamemode has been updated
        
        let info = self.gamemode_properties.info;
        self.score.accuracy = info.calc_acc(&self.score);
        self.score.performance = info.calc_perf(CalcPerfInfo {
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


            if self.gameplay_mode.is_multi() {
                self.actions.push(LobbyAction::MapComplete(Box::new(self.score.score.clone())));
            } 

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

                // host_id,
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
                while let Some(SpectatorFrame { time: frame_time, action }) = frames.pop_front() {
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
            self.handle_action(a, &settings).await;
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
            self.handle_frame(action, true, Some(time), true, &settings).await;
        }


        // handle animation
        #[cfg(feature="graphics")] {
            // let mut anim = std::mem::replace(&mut self.animation, Box::new(EmptyAnimation));
            // anim.update(time).await;
            // self.animation = anim;
            self.animation.update(time).await;
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
    async fn draw(&mut self, list: &mut RenderableCollection) {
        let time = self.time();

        // draw animation
        self.animation.draw(list).await;

        // draw gamemode
        if let Some(bounds) = self.fit_to_bounds { 
            list.push_scissor(bounds.into_scissor()); 
        }

        let state = GameplayDrawShell {
            time,
            gameplay_mode: &self.gameplay_mode,
            current_timing_point: self.timing_points.timing_point(),
            mods: &self.current_mods,
            score: &self.score,
            window_size: self.window_size
        };
        self.gamemode.draw(state, list).await;


        if self.fit_to_bounds.is_some() { 
            list.pop_scissor(); 
        }

        // dont draw score, combo, etc if this is a menu bg
        if self.gameplay_mode.is_preview() { return }


        // judgement indicators
        for indicator in self.judgement_indicators.iter() {
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
        self.center_text_helper.draw(time, self.window_size, list);


        // // draw playfield border (debug)
        // let b = self.gamemode.get_playfield();
        // list.push(Rectangle::new(
        //     b.pos,
        //     b.size,
        //     Color::TRANSPARENT_WHITE,
        //     Some(Border::new(Color::AQUA, 2.0))
        // ))

    }

    async fn handle_action(
        &mut self, 
        action: GameplayAction,
        settings: &Settings,
    ) {
        match action {
            GameplayAction::Pause => self.pause(),
            GameplayAction::Resume => self.start().await,
            GameplayAction::JumpToTime { time, skip_intro } => self.jump_to_time(time, skip_intro),
            GameplayAction::ApplyMods(mods) => self.apply_mods(mods).await,
            GameplayAction::FitToArea(bounds) => self.fit_to_area(bounds),
            GameplayAction::SetMode(mode) => self.set_mode(mode.into()),

            GameplayAction::AddReplayAction { action, should_save } => self.handle_frame(action, true, Some(self.time()), should_save, settings).await,
            GameplayAction::SetHitsoundsEnabled(enabled) => self.hitsound_manager.enabled = enabled,
        
            // not used here
            GameplayAction::RequestDifficulty => {}
        }
    }

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
                    score @ i32::MIN..=0 => self.score.score.score -= score.unsigned_abs() as u64,
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
                let diff = hit_time - note_time;
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


            GamemodeAction::PlayfieldChanged => {
                if self.animation.use_gamemode_playfield(self.gamemode_properties.info) {
                    self.animation.fit_to_area(self.gamemode.get_playfield());
                }

                self.layout_ui();
            }
        }
    }


    fn all_scores(&self) -> Vec<&IngameScore> {
        let mut list = self.score_list
            .iter()
            .chain([&self.score])
            .collect::<Vec<_>>();

        // sort by points
        list.sort_by(|a, b| b.score.score.cmp(&a.score.score));

        list
    }

    #[inline]
    fn time(&self) -> f32 {
        self.song_time - (self.lead_in_time + self.beatmap_preferences.audio_offset + self.global_offset)
    }

    #[cfg(feature="graphics")]
    async fn reload_skin(
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

            if self.animation.use_gamemode_playfield(self.gamemode_properties.info) {
                self.animation.fit_to_area(self.gamemode.get_playfield());
            } else {
                self.animation.window_size_changed(self.window_size);
            }
        }

        for i in self.ui_elements.iter_mut() {
            i.reload_skin(&source, skin_manager).await;
        }

        self.layout_ui();
    }


    fn window_focus_changed(&mut self, got_focus: bool) {
        // info!("window focus changed");
        if got_focus {
            self.pause_pending = false
        } else if self.can_pause() {
            if self.in_break() { self.pause_pending = true } else { self.should_pause = true }
        }
    }

    #[cfg(feature="graphics")]
    fn cleanup_textures(&mut self, skin_manager: &mut dyn SkinProvider) {
        // drop all texture references by dropping the gamemode
        // this should be fine since we shouldnt be re-using this gamemode at this time anyways
        self.gamemode = Box::new(NoMode);
        self.gamemode_properties = self.gamemode.properties();
        skin_manager.free_by_usage(SkinUsage::Beatmap);

        let path = self.beatmap.get_parent_dir().unwrap().to_string_lossy().to_string();
        skin_manager.free_by_source(TextureSource::Beatmap(path));
    }


    // make not async?
    fn fit_to_area(&mut self, bounds: Bounds) {
        // info!("fitting to area: {bounds:?}");
        self.fit_to_bounds = Some(bounds);
        self.gamemode.set_bounds(bounds, false);

        // if the anim uses the gamemode playfield, it will get updated once the gamemode's playfield is updated
        #[cfg(feature="graphics")]
        if self.animation.use_gamemode_playfield(self.gamemode_properties.info) {
            self.animation.fit_to_area(self.gamemode.get_playfield());
        } else {
            // .is_fullscreen(true) = hack-ish
            self.animation.fit_to_area(PlayfieldNonsense::new_simple(bounds).is_fullscreen(true));
        }

        self.layout_ui();
    }



    // can be from either paused or new
    async fn start(&mut self) {
        // if !self.gameplay_mode.is_preview() {
        //     self.hitsound_manager.enabled = false;
        // }
        if let Some(bounds) = self.fit_to_bounds {
            self.gamemode.set_bounds(bounds, false);
        } else {
            self.gamemode.set_bounds(Bounds::new(Vector2::ZERO, self.window_size), true);
        }

        if self.should_hide_cursor() {
            self.actions.push(CursorAction::SetVisible(false));
        } else {
            self.actions.push(CursorAction::SetVisible(true));
        }

        self.pause_pending = false;
        self.should_pause = false;

        // offset our start time by the duration of the pause
        if let Some(pause_time) = self.pause_start.take() {
            self.start_time += chrono::Utc::now().timestamp() - pause_time
        }

        // re init ui
        self.layout_ui();

        if !self.started {
            self.reset().await;

            //TODO: probably want to skip other things as well
            if !self.gameplay_mode.is_replay() {
                #[cfg(feature="gameplay")]
                self.outgoing_spectator_frame(SpectatorFrame::new(0.0, SpectatorAction::Play {
                    beatmap_hash: self.beatmap.hash(),
                    mode: self.gamemode_properties.playmode().to_string(),
                    mods: self.score.mods.clone(),
                    speed: self.current_mods.speed.as_u16(),
                    map_game: self.metadata.beatmap_type.into(),
                    map_link: None
                }));

                // self.outgoing_spectator_frame(SpectatorFrame::new(0.0, SpectatorAction::MapInfo {
                //     beatmap_hash: self.beatmap.hash(),
                //     game: format!("{:?}", self.metadata.beatmap_type).to_lowercase(),
                //     download_link: None
                // }));
            }

            if self.gameplay_mode.is_preview() {
                // dont do lead in
                self.lead_in_time = 0.0;
            } else {
                self.lead_in_timer = TatakuInstant::now();
                self.lead_in_time = LEAD_IN_TIME;
            }

            // volume is set when the song is actually started (when lead_in_time is <= 0)
            self.started = true;

            // run the startup code
            let mut on_start:Box<dyn FnOnce(&mut Self) + Send + Sync> = Box::new(|_|{});
            std::mem::swap(&mut self.on_start, &mut on_start);
            on_start(self);

        } else if self.lead_in_time <= 0.0 {
            // if this is a preview, dont do anything
            if self.gameplay_mode.is_preview() { return }

            #[cfg(feature="gameplay")]
            self.outgoing_spectator_frame(SpectatorFrame::new(self.time(), SpectatorAction::UnPause));
            self.actions.push(SongAction::Play);
            self.gamemode.unpause();
        }
    
        self.layout_ui();
    }
    fn pause(&mut self) {
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
    async fn reset(&mut self) {
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
        self.lead_in_timer = TatakuInstant::now();


        let playmode = self.gamemode_properties.playmode().to_string();

        self.actions.push(GameAction::from((self.id.clone(), GameplayAction::RequestDifficulty)));

        let username = self.score.username.clone();
        self.score = IngameScore::new(Score::new(self.beatmap.hash(), username, playmode), true, false);
        self.score.speed = self.current_mods.speed;
        self.timing_points.reset();

        // get all available mods for this playmode
        {
            self.score_multiplier = 1.0;

            self.score.mods = self.current_mods.map_mods_to_thing(self.gamemode_properties.info);
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
    fn fail(&mut self) {
        if self.failed 
            || self.current_mods.has_nofail() 
            || self.current_mods.has_autoplay() 
            || self.gameplay_mode.is_preview() 
            || self.gameplay_mode.is_multi() { 
            return
        }
        
        self.failed = true;
        self.failed_time = self.time();
        debug!("failed");
    }

    fn combo_break(
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
    fn jump_to_time(
        &mut self, 
        time: f32, 
        skip_intro: bool
    ) {
        if skip_intro {
            self.lead_in_time = 0.0;
        }

        self.actions.push(SongAction::SetPosition(time));
        // self.song.set_position(time);

        self.pending_time_jump = Some(time);
    }

    fn on_complete(&mut self) {
        // make sure the cursor is visible
        // CursorManager::set_visible(true);
        self.actions.push(CursorAction::SetVisible(true));
        // undo any cursor override
        // CursorManager::set_ripple_override(None);
        self.actions.push(CursorAction::OverrideRippleRadius(None));

        #[cfg(feature="gameplay")]
        if let GameplayModeInner::Spectator {
            buffered_score_frames, ..
        } = &mut *self.gameplay_mode {
            // if we have a score frame we havent dealt with yet, its most likely the score frame sent once the map has ended
            if !buffered_score_frames.is_empty() {
                self.score.score = buffered_score_frames.last().cloned().unwrap().1;
            }


            // let mut score_menu = ScoreMenu::new(&manager.score, manager.metadata.clone(), false);
            // score_menu.dont_close_on_back = true;
            // self.score_menu = Some(score_menu);
        }
    }

    /// using a getter for this since we dont want anything to directly change it
    fn get_mode(&self) -> &GameplayModeInner {
        &self.gameplay_mode
    }
    fn set_mode(&mut self, mode: GameplayModeInner) {
        println!("setting gameplay mode to {mode:?}");

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

                self.score.mods = self.current_mods.map_mods_to_thing(self.gamemode_properties.info);
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

    fn set_id(&mut self, id: GameplayId) {
        // make sure we dont add a reference count to our copy of the id
        // this makes sure things are cleaned up properly when the manager is dropped
        self.id = Arc::new(*id);
    }

}



impl Drop for GameplayManager {
    fn drop(&mut self) {
        if self.gamemode_properties.playmode() != "none" {
            error!("gameplay manager dropped without cleaning up textures !!!!!!!!!!!");
        }
    }
}

struct DummyOnlineThing;
impl GameplayManagerOnline for DummyOnlineThing {
    fn send_spec_frames(&mut self, _frames: Vec<SpectatorFrame>, _force: bool) {}
    fn get_pending_frames(&mut self) -> Vec<SpectatorFrame> { Vec::new() }
    fn update_lobby_score(&mut self, _score: Score) { }
    fn our_spectator_list(&mut self) -> Option<SpectatorList> { None }
}

#[derive(Default)]
pub struct GameplaySpectatorInfo {
    /// when was the last time the score was synchronized?
    pub last_score_sync: f32,

    /// who is currently spectating us?
    pub spectators: SpectatorList
}



pub async fn manager_from_playmode_path_hash(
    infos: &GamemodeInfos,
    incoming_mode: &str,
    map_path: String,
    map_hash: Md5Hash,
    mods: ModManager,
    settings: &Settings,
) -> TatakuResult<GameplayManager> {
    let beatmap = Beatmap::from_path_and_hash(map_path, map_hash)?;
    let playmode = beatmap.playmode(incoming_mode.to_owned());

    let info = infos.get_info(&playmode)?;

    let gamemode = info.create_game(&beatmap, settings).await?;
    Ok(GameplayManager::new(beatmap, gamemode, mods, settings).await)
}

pub async fn manager_from_playmode(
    infos: &GamemodeInfos,
    incoming_mode: &str,
    beatmap: &BeatmapMeta,
    mods: ModManager,
    settings: &Settings,
) -> TatakuResult<GameplayManager> {
    let beatmap = Beatmap::from_metadata(beatmap)?;
    let playmode = beatmap.playmode(incoming_mode.to_owned());

    let info = infos.get_info(&playmode)?;

    let gamemode = info.create_game(&beatmap, settings).await?;

    Ok(GameplayManager::new(beatmap, gamemode, mods, settings).await)
}
