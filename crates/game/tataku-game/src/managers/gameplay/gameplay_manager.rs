use std::sync::mpsc::TryRecvError;

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
    pub gamemode_properties: GameModeProperties,

    pub current_mods: Arc<ModManager>,
    pub beatmap_preferences: BeatmapPreferences,

    pub gameplay_mode: Box<GameplayModeInner>,
    gameplay_actions: Vec<GameplayAction>,


    pub score: IngameScore,
    // pub score_multiplier: f32,

    pub health: Box<dyn HealthManager>,
    pub judgments: Vec<HitJudgment>,
    pub key_counter: KeyCounter,

    #[cfg(feature = "graphics")] ui_elements: Vec<GameplayWidgetContainer>,
    #[cfg(feature = "graphics")] editor: Option<EditorChannels>,

    #[cfg(feature="graphics")]
    animation: Box<dyn BeatmapAnimation>,

    pub score_list: Vec<IngameScore>,
    scores_loaded: bool,

    /// used for discord rich presence
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

    /// (map.time, note.time - hit.time)
    hitbar_timings: Vec<(f32, f32)>,

    /// list of judgement indicators to draw
    #[cfg(feature="graphics")] pub judgement_indicators: Vec<Box<dyn JudgementIndicator>>,

    pub common_game_settings: Arc<CommonGameplaySettings>,
    window_size: Vector2,
    fit_to_bounds: Option<Bounds>,

    // spectator info
    pub spectator_info: GameplaySpectatorInfo,

    /// what should the game do on start?
    /// mainly a helper for spectator
    pub on_start: Option<Box<dyn FnOnce(&mut Self) + Send + Sync>>,

    pub events: Vec<BeatmapEvent>,

    pending_time_jump: Option<f32>,
    pending_frames: Vec<ReplayFrame>,

    map_diff: f32,
    song_time: f32,
}

impl GameplayManager {
    pub fn new(
        beatmap: Beatmap,
        mut gamemode: Box<dyn GameMode>,
        mut mods: ModManager,
        settings: &Settings,
    ) -> Self {
        let timing_points = TimingPointHelper::new_from_beatmap(&beatmap);

        let properties = gamemode.properties(&timing_points);
        let playmode = properties.playmode();
        let metadata = beatmap.get_beatmap_meta();

        if mods.get_speed() == 0.0 { mods.set_speed(1.0); }
        let current_mods = Arc::new(mods);

        let time = chrono::Utc::now().timestamp();
        let mut score = Score::new(
            beatmap.hash(), 
            settings.username.clone(), 
            playmode.to_string()
        );

        score.speed = current_mods.speed;
        score.time = time as u64;

        let mut actions = ActionQueue::new();
    
        for (id, list) in properties.sound_list.clone() {
            actions.push(AudioAction::new(
                id, 
                AudioActionType::Load { list }
            ).into());
        }
        // combo break sound
        actions.push(AudioAction::new(
            "combobreak", 
            AudioActionType::Load { 
                list: AudioLoadData::new_multi_source(
                    "combobreak", 
                    None::<String>, 
                    &[
                        HitsoundSource::Beatmap, 
                        HitsoundSource::Skin, 
                        HitsoundSource::Default
                    ]
                )
            }
        ).into());

        // make sure the gamemode has the correct mods applied
        gamemode.handle_gameplay_event(GameplayEvent::ApplyMods(current_mods.clone()));

        Self {
            id: Arc::new(u32::MAX),
            actions,

            timing_points,
            current_mods,
            health: Box::new(DefaultHealthManager::default()),
            key_counter: KeyCounter::new(&properties.keys),

            judgments: properties.info.judgments.to_vec(),
            score: IngameScore::new(score, true, false),

            events: beatmap.get_events(),

            lead_in_time: LEAD_IN_TIME,
            lead_in_timer: TatakuInstant::now(),
            end_time: properties.end_time,
            global_offset: settings.global_offset,

            beatmap_preferences: Database::get_beatmap_prefs(metadata.beatmap_hash),

            common_game_settings: Arc::new(settings.common_game_settings.clone()),

            metadata,
            beatmap,
            gamemode,
            gamemode_properties: properties,

            scores_loaded: false,
            score_list: Vec::new(),
            // score_loader, values: &mut dyn Reflec
            window_size: Vector2::ZERO,
            start_time: time,
            
            #[cfg(feature="graphics")] editor: None,
            #[cfg(feature="graphics")] ui_elements: Vec::new(),
            #[cfg(feature="graphics")] judgement_indicators: Vec::new(),
            #[cfg(feature="graphics")] animation: Box::new(EmptyAnimation),
            gameplay_mode: Box::new(GameplayModeInner::Normal),
            gameplay_actions: Vec::new(),

            failed: false,
            failed_time: 0.0,
            // score_multiplier: 1.0,
            started: false,
            completed: false,

            hitbar_timings: Vec::new(),
            spectator_info: GameplaySpectatorInfo::default(),
            on_start: None,
            fit_to_bounds: None,
            should_pause: false,
            pause_pending: false,
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
    pub fn init_ui(&mut self, font_contexts: &mut TextLayoutContexts) {
        let layouts = 
            std::fs::read("ui_layouts.json").ok()
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

        for i in DEFAULT_GAMEPLAY_WIDGETS
            .iter()
            .map(|i| i.name) 
        {
            loader.load(i);
        }

        // Anything in the gamemode itself
        self.gamemode.build_widgets(&mut loader);

        // update every ui element so they're all initialized
        let mut shell = GameplayWidgetUpdateShell {
            manager: self,
            font_context: font_contexts,
            scale: Vector2::ONE,
        };

        for e in loader.elements.iter_mut() {
            e.update(&mut shell);
        }

        // update our list
        self.ui_elements = loader.elements;

        // layout will be performed on game start (and skin load)
    }

    #[cfg(feature = "graphics")]
    fn layout_ui(&mut self) {
        if let Err(e) = GameplayWidgetContainer::layout(
            &mut self.ui_elements,
            self.gamemode.get_playfield().bounds,
            self.window_size
        ) {
            error!("error laying out ui elements! {e:?}");
        }

        if let Some(channels) = &self.editor {
            for i in self.ui_elements.iter() {
                let r = channels
                    .event_sender
                    .send(GameplayWidgetEvent { 
                    target: Some(i.element_name.clone()), 
                    action: GameplayWidgetEventType::Update {
                        bounds: i.get_bounds(),
                    },
                });

                if r.is_err() {
                    self.editor = None;
                    break;
                }
            }
        }
    }

    #[cfg(feature="gameplay")] 
    pub fn skip_intro(&mut self) {
        let Some(mut time) = self.gamemode.skip_intro(self.time()) 
        else { return };

        // really not sure whats happening here lol
        if self.lead_in_time > 0.0 && time > self.lead_in_time {
            time -= self.lead_in_time - 0.01;
            self.lead_in_time = 0.01;
        }

        self.actions.push(SongAction::SetPosition(time).into());
    }
}

// getters, setters, properties
impl GameplayManager {

    /// is this game pausable
    pub fn can_pause(&mut self) -> bool {
        // never allow pausing in multi
        #[cfg(feature="gameplay")]
        if self.gameplay_mode.is_multi() { return false; }
        self.should_pause 
        || !(
            self.current_mods.has_autoplay() 
            || self.gameplay_mode.is_replay() 
            || self.failed
        )
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
        !(
            self.gameplay_mode.is_replay() 
            || self.current_mods.has_autoplay() 
            || self.ui_changed
        )
    }


    pub fn update_difficulty(
        &mut self, 
        provider: &mut dyn DifficultyProvider
    ) {
        self.map_diff = provider.get_diff(
            &self.beatmap.get_beatmap_meta(), 
            self.gamemode_properties.playmode(), 
            &self.current_mods
        ).unwrap_or_default();

        trace!("Updated diff: {}", self.map_diff);
    }
}

// Events and States
impl GameplayManager {
    fn handle_frame(
        &mut self,
        frame: ReplayAction,
        force: bool,
        force_time: Option<f32>,
        should_add: bool,
        settings: &Settings
    ) {
        // note to self: force is used when the frames are from the gamemode's update function
        #[cfg(feature="gameplay")]
        if let ReplayAction::Press(KeyPress::SkipIntro) = frame {
            if self.gameplay_mode.is_multi() {
                self.actions.push(LobbyAction::SendSkipRequest.into());
            } else {
                self.skip_intro();
            }
            
            // more to do?
            return;
        }

        let add_frames = !(
            self.current_mods.has_autoplay() || self.gameplay_mode.is_replay()
        );

        if force || add_frames {
            match frame {
                ReplayAction::Press(k) => self.key_counter.key_down(k),
                ReplayAction::Release(k) => self.key_counter.key_up(k),
                _ => {}
            }

            let time = force_time.unwrap_or_else(|| self.time());
            let frame = ReplayFrame::new(time, frame);

            let mut state = create_update_state!(
                self, 
                self.time(), 
                settings
            );
            self.gamemode.handle_replay_frame(frame, &mut state);

            for action in state.actions.take() {
                self.handle_gamemode_action(action, settings);
            }

            if add_frames && should_add {
                if let Some(r) = self.score.replay.as_mut() { 
                    r.frames.push(frame);
                }

                #[cfg(feature="gameplay")]
                self.outgoing_spectator_frame(
                    SpectatorFrame::new(time, SpectatorAction::ReplayAction { 
                        action: frame.action 
                    }),
                );
            }
        }
    }
}

// Input Handlers
#[cfg(feature="graphics")]
impl GameplayManager {

    pub fn key_down(
        &mut self, 
        key_input: &KeyInput, 
        mods: KeyModifiers,
        settings: &Settings,
    ) -> bool {
        if key_input.repeat { return false }
        let Some(key) = key_input.as_key() else { return false };

        if (self.gameplay_mode.is_replay() || self.current_mods.has_autoplay()) 
            && !self.gameplay_mode.is_preview() 
        {
            // check replay-only keys
            if key == Key::Escape {
                self.started = false;
                self.completed = true;
                return true;
            }
        }

        // check map restart key
        if key == self.common_game_settings.map_restart_key 
            && !self.gameplay_mode.is_multi() 
        {
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
                    self.actions.push(MultiplayerAction::ExitMultiplayer.into());
                } else {
                    self.actions.push(Notification::new_text(
                        "Press escape again to quit the lobby", 
                        Color::RED, 
                        3_000.0
                    ).into());
                }
                
                return true;
            }
        }

        // ui editor toggle
        #[cfg(feature = "ui")]
        if key == Key::F9 {
            if self.editor.is_some() {
                self.editor = None;
                if !self.gamemode_properties.show_cursor {
                    self.actions.push(CursorAction::SetVisible(false).into());
                }
                return true;
            }


            // ensure autoplay is enabled
            self.ui_changed = true;
            if !self.current_mods.has_autoplay() {
                let mut mods = (*self.current_mods).clone();
                mods.add_mod(Autoplay);
                self.apply_mods(mods);
            }

            let (
                event_sender, 
                event_receiver
            ) = channel();
            let (
                action_sender, 
                action_receiver
            ) = channel();

            let editor = GameplayWidgetEditor::new(
                &self.ui_elements,
                action_sender,
                event_receiver
            );

            self.actions.push(MenuAction::AddDialogRaw { 
                dialog: Box::new(editor), 
                options: Box::new(DialogCreateOptions {
                    background: false,
                    ..Default::default()
                })
            }.into());

            self.editor = Some(EditorChannels {
                event_sender,
                action_receiver: Arc::new(Mutex::new(action_receiver)),
            });

            self.actions.push(CursorAction::SetVisible(true).into());
            return true;
        }

        // check for offset changing keys
        if mods.shift {
            let mut t = 0.0;
            if key == self.common_game_settings.key_offset_up { t = 5.0 }
            if key == self.common_game_settings.key_offset_down { t = -5.0 }

            if t != 0.0 {
                self.increment_global_offset(t);
                return true;
            }
        } else {
            if key == self.common_game_settings.key_offset_up { 
                self.increment_offset(5.0);
                return true;
            }
            if key == self.common_game_settings.key_offset_down { 
                self.increment_offset(-5.0); 
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
            );
            
            return true;
        }

        false
    }


    #[cfg(feature="graphics")]
    pub fn window_size_changed(&mut self, window_size: Vector2) {
        self.window_size = window_size;
        if self.fit_to_bounds.is_none() {
            self.gamemode.handle_gameplay_event(GameplayEvent::SetBounds { 
                bounds: Bounds::new(Vector2::ZERO, window_size), 
                full_window: true
            });
        }

        if self.animation.use_gamemode_playfield(self.gamemode_properties.info) {
            self.animation.fit_to_area(self.gamemode.get_playfield());
        } else {
            self.animation.window_size_changed(window_size);
        }

        self.layout_ui();
    }


    pub fn handle_input(&mut self, input: InputEvent, settings: &Settings) {
        match &input.event {
            InputType::KeyPress(key_input) => {
                if self.key_down(key_input, input.key_mods, settings) {
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

        let Some(frame) = self.gamemode.handle_input(input) 
        else { return };

        self.handle_frame(
            frame, 
            false, 
            None, 
            true,
            settings,
        );

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

    pub fn increment_offset(&mut self, delta: f32) {
        self.beatmap_preferences.audio_offset += delta;
        // #[cfg(feature="graphics")]
        // self.center_text_helper.set_value(
        //     format!("Offset: {:.2}ms", self.beatmap_preferences.audio_offset),
        //     self.time()
        // );

        // update the beatmap offset
        let new_prefs = self.beatmap_preferences.clone();
        let hash = self.beatmap.hash();
        Database::save_beatmap_prefs(hash, &new_prefs);
    }

    pub fn increment_global_offset(&mut self, delta: f32) {
        self.global_offset += delta;
        // #[cfg(feature="graphics")]
        // self.center_text_helper.set_value(
        //     format!("Global Offset: {:.2}ms", self.global_offset),
        //     self.time()
        // );
    }

    pub fn force_update_settings(&mut self, settings: &Settings) {
        self.gamemode.force_update_settings(settings);
        self.global_offset = settings.global_offset;
    }

    fn in_break(&self) -> bool {
        let time = self.time();

        fn check(event: &BeatmapEvent, time: f32) -> bool {
            #[allow(irrefutable_let_patterns, reason = "more events will be added eventually")]
            let BeatmapEvent::Break { start, end } = event 
            else { return false };

            time >= *start && time < *end 
        }

        self.events
            .iter()
            .any(|e| check(e, time))
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
        self.actions.push(OnlineAction::SendSpectatorFrame {
            frame: Box::new(frame),
            force: false
        }.into());
    }

    pub fn outgoing_spectator_frame_force(
        &mut self, 
        frame: SpectatorFrame,
    ) {
        if !self.gameplay_mode.should_send_spec_frames() { return }
        self.actions.push(OnlineAction::SendSpectatorFrame {
            frame: Box::new(frame),
            force: true
        }.into());
    }

    pub fn add_spec_frame(&mut self, frame_host_id: u32, frame: SpectatorFrame) {
        let GameplayModeInner::Spectator { 
            frames, 
            host_id, 
            ..
        } = &mut *self.gameplay_mode else { return };

        if *host_id == frame_host_id {
            frames.push_back(frame);
        } 
    }
}

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


    fn apply_mods(&mut self, mut mods: ModManager) {
        if self.gameplay_mode.is_preview() {
            mods.add_mod(Autoplay);
        }

        self.current_mods = Arc::new(mods);
        self.gamemode.handle_gameplay_event(GameplayEvent::ApplyMods(
            self.current_mods.clone()
        ));
    }

    fn update(
        &mut self, 
        values: &mut dyn Reflect,
        font_contexts: &mut TextLayoutContexts,
        actions: &mut ActionQueue,
    ) {
        let new_time = *values.reflect_get::<f32>("song.position").unwrap();
        let settings = values
            .reflect_get::<Settings>("settings")
            .unwrap();

        self.song_time = new_time;

        // make sure we jump to the time we're supposed to be at
        if let Some(time) = self.pending_time_jump {
            self.pending_time_jump = None;

            let mut state = create_update_state!(self, time, &settings);
            self.gamemode.time_jump(time, &mut state);
        }

        // check map restart
        if let Some(press_time) = self.restart_key_hold_start
        && press_time.as_millis() >= self.common_game_settings.map_restart_delay {
            self.reset();
            actions.extend(self.actions.take());
            return
        }

        // check pause
        if self.pause_pending && !self.in_break() {
            info!("pausing");
            self.pause();
            self.pause_pending = false;
            self.should_pause = true;
        }


        // update ui editor
        #[cfg(feature = "graphics")]
        if let Some(channels) = &self.editor {
            let receiver = channels
                .action_receiver.clone();
            let receiver = receiver.lock();

            loop {
                match receiver.try_recv() {
                    Ok(action) => {
                        if let GameplayWidgetActionType::Done = &action.action {
                            // TODO: save widgets
                            continue
                        }

                        let Some(ele) = self
                            .ui_elements
                            .iter_mut()
                            .find(|i| 
                                i.element_name == action.target
                            )
                        else { continue };

                        match action.action {
                            GameplayWidgetActionType::Add(layout) => {
                                ele.layout = layout;
                                ele.layout.visible = true;
                                self.layout_ui();
                            }
                            GameplayWidgetActionType::Move(layout) => {
                                ele.layout = layout;
                                self.layout_ui();
                            }
                            GameplayWidgetActionType::Remove => {
                                ele.layout.visible = false;
                            }
                            GameplayWidgetActionType::Done => {}
                        }
                    }

                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        self.editor = None;
                        break;
                    },
                }
            }
        }

        // update ui elements
        #[cfg(feature = "graphics")]
        if !self.gameplay_mode.is_preview() {
            let mut ui_elements = self.ui_elements.take();
            let mut shell = GameplayWidgetUpdateShell {
                manager: self,
                font_context: font_contexts,
                scale: Vector2::ONE
            };

            for ui in ui_elements.iter_mut() {
                ui.update(&mut shell);
            }
            self.ui_elements = ui_elements;
        }

        // get the time with offsets
        let time = self.time();

        // check lead-in time
        if self.lead_in_time > 0.0 {
            let elapsed = self.lead_in_timer.elapsed().as_micros() as f32 / 1000.0;
            self.lead_in_timer = TatakuInstant::now();
            self.lead_in_time -= elapsed * self.game_speed();

            if self.lead_in_time <= 0.0 {
                self.actions.push(SongAction::SetRate(self.game_speed()).into());
                self.actions.push(SongAction::SetVolume(settings.get_music_vol()).into());
                self.actions.push(SongAction::SetPosition(-self.lead_in_time).into());
                self.actions.push(SongAction::Play.into());
                self.lead_in_time = 0.0;
            }
        }

        #[cfg(feature="gameplay")]
        let scores_list = values
            .reflect_get::<Vec<IngameScore>>("score_list.scores")
            .unwrap();
        #[cfg(feature="gameplay")]
        let scores_loaded = *values
            .reflect_get::<bool>("score_list.loaded")
            .unwrap();

        #[cfg(feature="gameplay")]
        if !self.scores_loaded 
            && self.gameplay_mode.should_load_scores() 
            && scores_loaded 
        {
            self.score_list = (*scores_list).clone();
            self.scores_loaded = true;

            for s in self.score_list.iter_mut() {
                s.is_previous = s.username == self.score.username;
            }
        }


        let tp_updates = self.timing_points.update(time);
        for tp_update in tp_updates {
            match tp_update {
                TimingPointUpdate::BeatHappened(pulse_length) 
                    => self.gamemode.handle_gameplay_event(GameplayEvent::BeatHappened { 
                        pulse_length
                    }),

                TimingPointUpdate::KiaiChanged(enabled) 
                    => self.gamemode.handle_gameplay_event(GameplayEvent::KiaiChanged { 
                        enabled 
                    }),
            }
        }

        // update hit timings bar
        #[cfg(feature="graphics")] 
        self.hitbar_timings
            .retain(|(hit_time, _)| time - hit_time < HIT_TIMING_DURATION );

        // update judgement indicators
        #[cfg(feature="graphics")] 
        self.judgement_indicators.retain(|a| a.should_keep(time));

        // update gamemode
        let mut state = create_update_state!(self, time, &settings);


        self.gamemode.update(&mut state);
        for action in state.actions {
            self.handle_gamemode_action(action, &settings);
        }

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
            let new_rate = f32::lerp(
                self.game_speed(), 
                0.0, 
                (self.time() - self.failed_time) / 1000.0
            );

            if new_rate <= 0.05 {
                self.actions.push(SongAction::Pause.into());
                // self.song.pause();

                self.completed = true;
                // self.outgoing_spectator_frame_force((self.end_time + 10.0, SpectatorAction::Failed));
                trace!("show fail menu");
            } else {
                self.actions.push(SongAction::SetRate(new_rate).into());
            }

            actions.extend(self.actions.take());
        }

        // send map completed packets
        #[allow(clippy::collapsible_if, reason = "features")]
        if self.completed {
            #[cfg(feature="gameplay")] {
                let mut score = self.score.score.clone();
                score.replay = None;
                self.outgoing_spectator_frame_force(SpectatorFrame::new(
                    self.end_time + 10.0, 
                    SpectatorAction::ScoreSync { score }
                ));
            }

            #[cfg(feature="gameplay")]
            self.outgoing_spectator_frame_force(SpectatorFrame::new(
                self.end_time + 10.0, 
                SpectatorAction::Buffer
            ));


            #[cfg(feature="gameplay")]
            if self.gameplay_mode.is_multi() {
                self.actions.push(LobbyAction::MapComplete(
                    Box::new(self.score.score.clone())
                ).into());
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
                let buffer_duration = (time + SPECTATOR_BUFFER_OK_DURATION * 2.0)
                    .clamp(0.0, self.end_time);

                // handle pending frames
                while let Some(SpectatorFrame { 
                    time: frame_time, 
                    action 
                }) = frames.pop_front() {
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
                            self.actions.push(Notification::new_text(
                                "Host speccing someone", 
                                Color::BLUE, 
                                2000.0
                            ).into());
                        }
                        SpectatorAction::ReplayAction { 
                            action 
                        } => replay_frames.push(ReplayFrame::new(frame_time, action)),

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

                        SpectatorAction::TimeJump { time } 
                            => self.gameplay_actions.push(GameplayAction::JumpToTime {
                                time, 
                                skip_intro: true
                            }),

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
                    self.actions.push(LobbyAction::ScoreUpdate(Box::new(score)).into());
                }
            }

            _ => {}
        }

        // handle any pending gameplay actions
        for a in self.gameplay_actions.take() {
            self.handle_action(a, &settings);
        }

        // if its time to send another score sync packet
        #[cfg(feature="gameplay")]
        if self.spectator_info.last_score_sync + SPECTATOR_SCORE_SYNC_INTERVAL <= time {
            self.spectator_info.last_score_sync = time;

            // create and send the packet

            let mut score = self.score.score.clone();
            score.replay = None;
            self.outgoing_spectator_frame(SpectatorFrame::new(
                time, 
                SpectatorAction::ScoreSync { score })
            );
        }

        // handle any frames
        for ReplayFrame { 
            time, 
            action 
        } in self.pending_frames.take() {
            self.handle_frame(
                action, 
                true, 
                Some(time), 
                true, 
                &settings
            );
        }


        // handle animation
        #[cfg(feature="graphics")] {
            self.animation.update(time);
        }

        // update value collection
        {
            // TODO: placing
            let score = values
                .reflect_get_mut::<ReflectScore>("score")
                .unwrap();

            if score.time != self.score.time {
                *score = ReflectScore::new(&self.score, self.gamemode_properties.info);
            } else {
                score.update(&self.score);
            }
        }

        actions.extend(self.actions.take());
    }

    #[cfg(feature="graphics")]
    fn draw(&mut self, list: &mut RenderableCollection) {
        let time = self.time();

        // draw animation
        self.animation.draw(list);

        // draw gamemode
        // if let Some(bounds) = self.fit_to_bounds {
        //     list.push_scissor(bounds.into_scissor());
        // todo: is this still necessary?
        // }

        let state = GameplayDrawShell {
            time,
            gameplay_mode: &self.gameplay_mode,
            current_timing_point: self.timing_points.timing_point(),
            mods: &self.current_mods,
            score: &self.score,
            window_size: self.window_size
        };
        self.gamemode.draw(state, list);


        // if self.fit_to_bounds.is_some() {
        //     list.pop_scissor();
        // }

        // dont draw score, combo, etc if this is a menu bg
        if self.gameplay_mode.is_preview() { return }


        // judgement indicators
        for indicator in self
            .judgement_indicators
            .iter() 
        {
            indicator.draw(time, list);
        }

        // ui elements
        for i in self.ui_elements.iter_mut() {
            i.draw(list);
        }

        // // draw playfield border (debug)
        // let b = self.gamemode.get_playfield();
        // list.push(Rectangle::new(
        //     b.pos,
        //     b.size,
        //     Color::TRANSPARENT_WHITE,
        //     Some(Border::new(Color::AQUA, 2.0))
        // ))
    }

    fn handle_action(
        &mut self, 
        action: GameplayAction,
        settings: &Settings,
    ) {
        match action {
            GameplayAction::Pause => self.pause(),
            GameplayAction::Resume => self.start(),
            GameplayAction::JumpToTime { 
                time, 
                skip_intro
            } => self.jump_to_time(time, skip_intro),

            GameplayAction::ApplyMods(mods) => self.apply_mods(mods),
            
            GameplayAction::FitToArea(bounds) => {
                #[cfg(feature="graphics")] 
                self.fit_to_area(bounds);
            },
            GameplayAction::SetMode(mode) => self.set_mode(mode.into()),

            GameplayAction::AddReplayAction { 
                action, 
                should_save 
            } => self.handle_frame(
                action, 
                true, 
                Some(self.time()), 
                should_save, 
                settings
            ),
        
            // not used here
            GameplayAction::RequestDifficulty => {}
        }
    }

    fn handle_gamemode_action(
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
                let combo_mult = (
                    self.score.combo as f32 
                    * self.current_mods.score_multiplier
                ).floor() as u16;

                let score = judgment.base_score_value;

                let score = match judgment.combo_multiplier {
                    ComboMultiplier::None => score,
                    ComboMultiplier::Custom(mult) => (score as f32 * mult) as i32,
                    ComboMultiplier::Linear { 
                        combo, 
                        multiplier, 
                        combo_cap 
                    } => {
                        let combo_mult = combo_cap.map_or(
                            combo_mult, 
                            |cap| combo_mult.min(cap)
                        );
                        
                        let times = (combo_mult % combo).max(1) as f32;

                        (score as f32 * (multiplier * times)) as i32
                    }
                };

                match score {
                    score @ i32::MIN..=0 
                        => self.score.score.score -= score.unsigned_abs() as u64,

                    score @ 1.. 
                        => self.score.score.score += score as u64,
                }

                // do combo
                match judgment.affects_combo {
                    AffectsCombo::Increment => {
                        self.score.combo += 1;
                        self.score.max_combo = self.score.max_combo.max(self.score.combo);
                    }
                    AffectsCombo::Reset => self.combo_break(),
                    AffectsCombo::Ignore => {},
                }

                // do health
                self.health.apply_hit(&judgment, &self.score);
                self.score.health = self.health.get_ratio();

                // check health
                if self.health.is_dead(false) {
                    self.fail();
                }

                // check sd/pf mods
                if self.current_mods.has_sudden_death() && judgment.fails_sudden_death {
                    // TODO: change the judgment to a miss
                    self.fail();
                }
                if self.current_mods.has_perfect() && judgment.fails_perfect {
                    self.fail();
                }
            }
            GamemodeAction::PlayHitsound { 
                id, 
                volume, 
                repeat 
            } => {
                // TODO: timing point volume?
                // let timing_point = self.beatmap.control_point_at(note_time);
                // if self.gameplay_mode.is_preview() { vol *= settings.background_game_settings.hitsound_volume };
                self.actions.push(AudioAction::new(
                    id, 
                    AudioActionType::Play { 
                        volume, 
                        repeat, 
                        restart: true 
                    }).into()
                );
            }


            GamemodeAction::AddTiming { 
                hit_time, 
                note_time
            } => {
                let diff = hit_time - note_time;
                self.score.insert_stat(HitVarianceStat, diff);
                self.hitbar_timings.push((hit_time, diff));
            }

            #[cfg(feature="graphics")] 
            GamemodeAction::AddIndicator(
                mut indicator
            ) => {
                indicator.set_start_time(self.time());
                indicator.set_draw_duration(
                    self.common_game_settings.hit_indicator_draw_duration, 
                    settings
                );
                self.judgement_indicators.push(indicator);
            }

            GamemodeAction::AddStat { 
                stat, 
                value 
            } => self.score.insert_stat(stat, value),

            #[cfg(feature="graphics")] 
            GamemodeAction::RemoveLastJudgment => self.judgement_indicators.pop().nope(),
            GamemodeAction::ComboBreak => self.combo_break(),
            GamemodeAction::FailGame => self.fail(),
            GamemodeAction::ReplayAction(frame) => self.handle_frame(
                frame.action, 
                true, 
                Some(frame.time), 
                true, 
                settings
            ),
            
            
            GamemodeAction::ResetHealth => self.health.reset(),
            GamemodeAction::ReplaceHealth(new_health) 
                => self.health = new_health,
            GamemodeAction::MapComplete => self.completed = true,


            GamemodeAction::PlayfieldChanged => {
                #[cfg(feature="graphics")] 
                if self.animation.use_gamemode_playfield(
                    self.gamemode_properties.info
                ) {
                    self.animation.fit_to_area(self.gamemode.get_playfield());
                }

                #[cfg(feature = "graphics")]
                self.layout_ui();
            }

            #[cfg(not(feature="graphics"))] 
            _ => {}
        }
    }


    fn all_scores(&self) -> Vec<&IngameScore> {
        let mut list = self.score_list
            .iter()
            .chain([&self.score])
            .collect::<Vec<_>>();

        // sort by points
        list.sort_by(|a, b| 
            b.score.score.cmp(&a.score.score)
        );

        list
    }

    fn all_non_user_scores(&self) -> &[IngameScore] {
        &self.score_list
    }

    #[inline]
    fn time(&self) -> f32 {
        self.song_time - (
            self.lead_in_time 
            + self.beatmap_preferences.audio_offset 
            + self.global_offset
        )
    }

    #[cfg(feature="graphics")]
    fn reload_skin(
        &mut self, 
        skin_manager: &mut dyn SkinProvider,
        _settings: &Settings,
    ) {
        let parent_folder = self
            .beatmap
            .get_parent_dir()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let source = self.gamemode.reload_skin(
            &parent_folder, 
            skin_manager
        );

        for (id, list) in self
            .properties()
            .sound_list
            .clone() 
        {
            self.actions.push(AudioAction::new(
                id, 
                AudioActionType::Load { list }
            ).into());
        }

        #[cfg(feature="storyboards")]
        if let Some(anim) = self
            .beatmap
            .get_animation(skin_manager) 
        {
            self.animation = anim;

            if self.animation.use_gamemode_playfield(self.gamemode_properties.info) {
                self.animation.fit_to_area(self.gamemode.get_playfield());
            } else {
                self.animation.window_size_changed(self.window_size);
            }
        }

        let mut shell = GameplayWidgetReloadSkinShell {
            source: &source,
            skin_manager,
        };

        for i in self.ui_elements.iter_mut() {
            i.reload_skin(&mut shell);
        }

        self.layout_ui();
    }


    #[cfg(feature="graphics")]
    fn window_focus_changed(&mut self, got_focus: bool) {
        // info!("window focus changed");
        if got_focus {
            self.pause_pending = false;
        } else if self.can_pause() {
            if self.in_break() { 
                self.pause_pending = true;
            } else { 
                self.should_pause = true;
            }
        }
    }

    #[cfg(feature="graphics")]
    fn cleanup_textures(&mut self, skin_manager: &mut dyn SkinProvider) {
        // drop all texture references by dropping the gamemode
        // this should be fine since we shouldnt be re-using this gamemode at this time anyways
        self.gamemode = Box::new(NoMode);
        self.gamemode_properties = self.gamemode.properties(&self.timing_points);
        skin_manager.free_by_usage(SkinUsage::Beatmap);

        let path = self.beatmap
            .get_parent_dir()
            .unwrap()
            .to_string_lossy()
            .to_string();

        skin_manager.free_by_source(TextureSource::Beatmap(path));
    }


    #[cfg(feature="graphics")]
    fn fit_to_area(&mut self, bounds: Bounds) {
        // info!("fitting to area: {bounds:?}");
        self.fit_to_bounds = Some(bounds);
        self.gamemode.handle_gameplay_event(GameplayEvent::SetBounds { 
            bounds, 
            full_window: false 
        });

        // if the anim uses the gamemode playfield, it will get updated once the gamemode's playfield is updated
        #[cfg(feature="graphics")]
        if self.animation.use_gamemode_playfield(self.gamemode_properties.info) {
            self.animation.fit_to_area(self.gamemode.get_playfield());
        } else {
            // .is_fullscreen(true) = hack-ish
            self.animation.fit_to_area(
                PlayfieldNonsense::new_simple(bounds).is_fullscreen(true)
            );
        }

        self.layout_ui();
    }



    // can be from either paused or new
    fn start(&mut self) {
        #[cfg(feature="graphics")] {
            let event = if let Some(bounds) = self.fit_to_bounds {
                GameplayEvent::SetBounds { 
                    bounds, 
                    full_window: false 
                }
            } else {
                GameplayEvent::SetBounds { 
                    bounds: Bounds::new(Vector2::ZERO, self.window_size), 
                    full_window: true 
                }
            };

            self.gamemode.handle_gameplay_event(event);
        }
        

        #[cfg(feature="graphics")] 
        self.actions.push(CursorAction::SetVisible(
            !self.should_hide_cursor()
        ).into());

        self.pause_pending = false;
        self.should_pause = false;

        // offset our start time by the duration of the pause
        if let Some(pause_time) = self.pause_start.take() {
            self.start_time += chrono::Utc::now().timestamp() - pause_time;
        }

        // re init ui
        #[cfg(feature = "graphics")]
        self.layout_ui();

        if !self.started {
            self.reset();

            //TODO: probably want to skip other things as well
            if !self.gameplay_mode.is_replay() {
                #[cfg(feature="gameplay")]
                self.outgoing_spectator_frame(SpectatorFrame::new(
                    0.0, 
                    SpectatorAction::Play {
                        beatmap_hash: self.beatmap.hash(),
                        mode: self.gamemode_properties.playmode().to_string(),
                        mods: self.score.mods.clone(),
                        speed: self.current_mods.speed.as_u16(),
                        map_game: self.metadata.beatmap_type.into(),
                        map_link: None
                    })
                );
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
            if let Some(on_start) 
                = self.on_start.take() 
            {
                on_start(self);
            }

        } else if self.lead_in_time <= 0.0 {
            // if this is a preview, dont do anything
            if self.gameplay_mode.is_preview() { return }

            #[cfg(feature="gameplay")]
            self.outgoing_spectator_frame(SpectatorFrame::new(
                self.time(), 
                SpectatorAction::UnPause
            ));
            self.actions.push(SongAction::Play.into());
            self.gamemode.handle_gameplay_event(GameplayEvent::UnPaused);
        }
        

        #[cfg(feature = "graphics")]
        self.layout_ui();
    }

    fn pause(&mut self) {
        // make sure the cursor is visible
        #[cfg(feature="graphics")] 
        self.actions.push(CursorAction::SetVisible(true).into());
        // undo any cursor override
        #[cfg(feature="graphics")] 
        self.actions.push(CursorAction::OverrideRippleRadius(None).into());

        // self.song.pause();
        self.actions.push(SongAction::Pause.into());
        self.pause_start = Some(chrono::Utc::now().timestamp());

        // is there anything else we need to do?

        // might mess with lead-in but meh
        let time = self.time();
        #[cfg(feature="gameplay")]
        self.outgoing_spectator_frame_force(
            SpectatorFrame::new(time, SpectatorAction::Pause),
        );

        self.gamemode.handle_gameplay_event(GameplayEvent::Paused);
    }
    fn reset(&mut self) {
        self.gamemode.reset(&self.beatmap);
        self.health.reset();
        self.key_counter.reset();
        self.hitbar_timings.clear();
        #[cfg(feature="graphics")] 
        self.judgement_indicators.clear();
        self.restart_key_hold_start = None;

        if self.gameplay_mode.is_preview() {
        self.gamemode.handle_gameplay_event(GameplayEvent::ApplyMods(
            self.current_mods.clone()
        ));
        } else {
            // reset song
            self.actions.push(SongAction::Restart.into());
            self.actions.push(SongAction::Pause.into());
            self.actions.push(SongAction::SetPosition(0.0).into());
            self.actions.push(SongAction::SetRate(self.game_speed()).into());
        }

        self.completed = false;
        self.started = false;
        self.failed = false;
        self.lead_in_time = LEAD_IN_TIME / self.current_mods.get_speed();
        self.lead_in_timer = TatakuInstant::now();


        let playmode = self.gamemode_properties.playmode().to_string();
        self.actions.push(GameAction::from((
            self.id.clone(), 
            GameplayAction::RequestDifficulty
        )).into());

        let username = self.score.username.clone();
        self.score = IngameScore::new(
            Score::new(
                self.beatmap.hash(), 
                username, 
                playmode
            ), 
            true, 
            false
        );

        self.score.speed = self.current_mods.speed;
        self.timing_points.reset();

        // get all available mods for this playmode
        {
            // self.score_multiplier = self
            //     .current_mods
            //     .calculate_score_multiplier(self.gamemode_properties.info);

            self.score.mods = self.current_mods.map_mods_to_thing(
                self.gamemode_properties.info
            );

            // for m in self.score.mods.iter() {
            //     self.score_multiplier *= m.score_multiplier;
            // }
        }
        if self.score.replay.is_none() {
            self.score.replay = Some(Replay::new());
        }

        if !self.gameplay_mode.is_replay() {
            // only reset the replay if we arent replaying
            self.score.replay = Some(Replay::new());
            self.score.speed = self.current_mods.speed;
        } else {
            // if let Some(score) = &self.replay.score_data {
            //     self.score.username = score.username.clone();
            // }
        }

        // reset elements
        #[cfg(feature = "graphics")]
        for e in self.ui_elements.iter_mut() {
            e.reset_element();
        }

        // re-add judgments to score
        for j in &self.judgments {
            self.score.judgments.insert(j.id.to_owned(), 0);
        }

        #[cfg(feature="gameplay")]
        if self.gameplay_mode.should_load_scores() {
            self.actions.push(GameAction::RefreshScores.into());
        }

    }
    fn fail(&mut self) {
        #[cfg(feature="gameplay")] 
        let a = self.gameplay_mode.is_multi();
        #[cfg(not(feature="gameplay"))] 
        let a = false;

        if self.failed 
            || self.current_mods.has_nofail() 
            || self.current_mods.has_autoplay() 
            || self.gameplay_mode.is_preview() 
            || a
        { 
            return
        }
        
        self.failed = true;
        self.failed_time = self.time();
        debug!("failed");
    }

    fn combo_break(&mut self) {
        // play hitsound
        if self.score.combo >= 20 && !self.gameplay_mode.is_preview() {
            self.actions.push(AudioAction::new(
                "combobreak", 
                AudioActionType::Play { 
                    volume: 1.0, 
                    repeat: false, 
                    restart: true 
                }
            ).into());
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

        self.actions.push(SongAction::SetPosition(time).into());
        self.pending_time_jump = Some(time);
    }

    fn on_complete(&mut self) {
        // make sure the cursor is visible
        #[cfg(feature="graphics")] 
        self.actions.push(CursorAction::SetVisible(true).into());
        // undo any cursor override
        #[cfg(feature="graphics")] 
        self.actions.push(CursorAction::OverrideRippleRadius(None).into());

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
        // println!("setting gameplay mode to {mode:?}");

        match &mode {
            GameplayModeInner::Normal => {
                // dont think there's anything to do for this one, since its the default
            }

            GameplayModeInner::Replaying { score, .. } => {
                // load speed from score
                self.current_mods = Arc::new(ModManager::new(
                    score.mods.iter(),
                    score.speed,
                    self.gamemode_properties.info
                ));

                self.score.username = score.username.clone();
                self.score.mods = self
                    .current_mods
                    .map_mods_to_thing(self.gamemode_properties.info);
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
                self.score.username = host_username.to_string();
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

#[derive(Default)]
pub struct GameplaySpectatorInfo {
    /// when was the last time the score was synchronized?
    pub last_score_sync: f32,

    /// who is currently spectating us?
    pub spectators: SpectatorList
}


    #[cfg(feature = "graphics")]
struct EditorChannels {
    event_sender: Sender<GameplayWidgetEvent>,
    action_receiver: Arc<Mutex<Receiver<GameplayWidgetAction>>>,
}


pub fn manager_from_playmode_path_hash(
    infos: &GamemodeInfos,
    incoming_mode: &str,
    map_path: &str,
    map_hash: Md5Hash,
    mods: ModManager,
    settings: &Settings,
) -> TatakuResult<GameplayManager> {
    let beatmap = Beatmap::from_path_and_hash(map_path, map_hash)?;
    let playmode = beatmap.playmode(incoming_mode.to_owned());

    let info = infos.get_info(&playmode)?;

    let gamemode = info.create_game(
        &beatmap, 
        settings
    )?;
    Ok(GameplayManager::new(beatmap, gamemode, mods, settings))
}

pub fn manager_from_playmode(
    infos: &GamemodeInfos,
    incoming_mode: &str,
    beatmap: &BeatmapMeta,
    mods: ModManager,
    settings: &Settings,
) -> TatakuResult<GameplayManager> {
    let beatmap = Beatmap::from_metadata(beatmap)?;
    let playmode = beatmap.playmode(incoming_mode.to_owned());

    let info = infos.get_info(&playmode)?;

    let gamemode = info.create_game(
        &beatmap, 
        settings
    )?;

    Ok(GameplayManager::new(beatmap, gamemode, mods, settings))
}
