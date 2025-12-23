// massive ass file
use crate::prelude::*;
use super::helpers::*;

#[cfg(feature="ui")] use ui::widget::TextLayoutContexts;

use std::sync::mpsc::{
    Sender,
    Receiver,
    TryRecvError,
};

use common::{
    Score,
    replays::*,
    network::spectator::*,
};

use tataku::{
    Color,
    Bounds,
    Vector2,
};

use engine::{
    actions,
    gameplay,
    Settings,
    BeatmapMeta,
    Notification,
    beatmaps::Beatmap,
    settings::common_gameplay::CommonGameplaySettings as GameplaySettings,

    actions::{
        song::SongAction as SongAction,
        gameplay::GameplayAction as GameplayAction,
        audio::{
            AudioLoadData,
            HitsoundSource,
            AudioAction as AudioAction,
            AudioActionType as AudioActionType,
        },
        game::{
            GameplayId,
            GameAction as GameAction,
        },
        multiplayer::{
            LobbyAction as LobbyAction,
            MultiplayerAction as MultiplayerAction,
        }
    },
    gameplay::{
        *,
        mods::*,
        judgments::*,
        gameplay_manager::*,
        health_manager::{
            HealthManager,
            DefaultHealthManager,
        }
    }
};

#[cfg(feature="graphics")]
use engine::{
    BeatmapAnimation,
    gameplay::widgets::*,
    actions::cursor::CursorAction as CursorAction,
};

use input::{
    Key,
    KeyInput,
    InputType,
    InputEvent,
    KeyModifiers,
};



/// how long should center text be drawn for?
const CENTER_TEXT_DRAW_TIME:f32 = 2_000.0;

/// ms between spectator score sync packets
const SPECTATOR_SCORE_SYNC_INTERVAL:f32 = 1000.0;

const SCORE_SEND_TIME:f32 = 1_000.0;

/// how long of a buffer should we have? (ms)
const SPECTATOR_BUFFER_OK_DURATION:f32 = 500.0;

macro_rules! create_update_state {
    ($self: ident, $time: expr, $settings: expr) => {
        GameplayUpdateShell {
            time: $time,
            game_speed: $self.game_speed(),
            completed: $self.state.completed,

            mods: &$self.mods,
            current_timing_point: $self.timing_points.timing_point(),
            timing_points: &$self.timing_points,
            gameplay_type: &$self.gameplay_type_small,
            score: &$self.score,
            actions: Vec::new(),
            settings: $settings,
            action_queue: &mut $self.actions,
            window_size: $self.state.window_size,
        }
    }
}

pub struct GameplayManager {
    pub actions: actions::ActionQueue,
    pending_frames: Vec<ReplayFrame>,

    mods: Arc<ModManager>,
    
    metadata: Arc<BeatmapMeta>,
    timing_points: TimingPointHelper,
    beatmap_events: Vec<BeatmapEvent>,
    beatmap: engine::beatmaps::Beatmap,
    beatmap_preferences: engine::data::BeatmapPreferences,

    gamemode: Box<dyn Gamemode>,
    gamemode_properties: GamemodeProperties,

    gameplay_type: Box<GameplayType>,
    gameplay_actions: Vec<GameplayAction>,
    gameplay_type_small: GameplayTypeSmall,
    gameplay_settings: Arc<GameplaySettings>,


    score: IngameScore,
    pub score_list: ScoreList,

    state: GameplayState,
    key_counter: KeyCounter,
    judgments: Vec<HitJudgment>,
    health: Box<dyn health_manager::HealthManager>,

    #[cfg(feature="graphics")] editor: Option<EditorChannels>,
    #[cfg(feature="graphics")] animation: Box<dyn BeatmapAnimation>,
    #[cfg(feature="graphics")] ui_elements: WidgetTree,
    #[cfg(feature="graphics")] judgement_indicators: Vec<Box<dyn JudgementIndicator>>,

    // spectator info
    pub spectator_info: GameplaySpectatorInfo,

    /// what should the game do on start?
    /// mainly a helper for spectator
    pub on_start: Option<Box<dyn FnOnce(&mut Self) + Send + Sync>>,

    map_diff: f32,
}
impl GameplayManager {
    fn new(
        beatmap: engine::beatmaps::Beatmap,
        mut gamemode: Box<dyn Gamemode>,
        mut mods: ModManager,
        settings: &Settings,
        database: &dyn engine::database::DatabaseProvider,
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
            settings.connection().tataku_username.clone(),
            playmode.to_string()
        );

        score.speed = current_mods.speed;
        score.time = time as u64;

        let mut actions = actions::ActionQueue::new();

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
            actions,

            timing_points,
            mods: current_mods,
            health: Box::new(DefaultHealthManager::default()),
            key_counter: KeyCounter::new(&properties.keys),

            judgments: properties.info.judgments.to_vec(),
            score: IngameScore::new(score, true, false),

            beatmap_events: beatmap.get_events(),

            state: GameplayState {
                start_time: time,
                lead_in_time: LEAD_IN_TIME,
                end_time: properties.end_time,
                global_offset: settings.global_offset,
                ..GameplayState::default()
            },

            beatmap_preferences: database
                .get_beatmap_preferences(metadata.beatmap_hash)
                .unwrap_or_default(),

            gameplay_settings: Arc::new(settings.common_game_settings.clone()),

            metadata,
            beatmap,
            gamemode,
            gamemode_properties: properties,

            score_list: ScoreList::default(),

            #[cfg(feature="graphics")] editor: None,
            #[cfg(feature="graphics")] ui_elements: WidgetTree::new(),
            #[cfg(feature="graphics")] judgement_indicators: Vec::new(),
            #[cfg(feature="graphics")] animation: Box::new(engine::game::beatmap_animation::EmptyAnimation),
            gameplay_type: Box::new(GameplayType::Normal),
            gameplay_type_small: GameplayTypeSmall::Normal,
            gameplay_actions: Vec::new(),
            pending_frames: Vec::new(),
            spectator_info: GameplaySpectatorInfo::default(),
            on_start: None,

            map_diff: 0.0,
        }
    }


    fn create_inner(
        infos: &GamemodeInfos,
        incoming_mode: &str,
        beatmap: Beatmap,
        mods: ModManager,
        settings: &Settings,
        database: &dyn engine::database::DatabaseProvider,
    ) -> tataku::Result<GameplayManager> {
        let playmode = beatmap.playmode(incoming_mode.to_owned());
        let info = infos.get_info(&playmode)?;
        let gamemode = info.create_game(
            &beatmap,
            settings
        )?;

        Ok(GameplayManager::new(
            beatmap,
            gamemode,
            mods,
            settings,
            database,
        ))
    }
    
    pub fn create_from_path_hash(
        infos: &GamemodeInfos,
        incoming_mode: &str,
        map_path: &str,
        map_hash: common::Md5Hash,
        mods: ModManager,
        settings: &Settings,
        database: &dyn engine::database::DatabaseProvider,
    ) -> tataku::Result<GameplayManager> {
        let beatmap = Beatmap::from_path_and_hash(map_path, map_hash)?;

        Self::create_inner(
            infos, 
            incoming_mode, 
            beatmap, 
            mods, 
            settings, 
            database
        )
    }


    pub fn create(
        infos: &GamemodeInfos,
        incoming_mode: &str,
        beatmap: &BeatmapMeta,
        mods: ModManager,
        settings: &Settings,
        database: &dyn engine::database::DatabaseProvider,
    ) -> tataku::Result<GameplayManager> {
        let beatmap = Beatmap::from_metadata(beatmap)?;
        Self::create_inner(
            infos, 
            incoming_mode, 
            beatmap, 
            mods, 
            settings, 
            database
        )
    }


    pub fn update(
        &mut self,
        values: &mut ValueCollection,
        #[cfg(feature="ui")] font_contexts: &mut TextLayoutContexts,
        actions: &mut actions::ActionQueue,
    ) {
        match self.gameplay_type_small {
            GameplayTypeSmall::Simulating => {
                let time = self.time();
                // update gamemode
                self.update_gamemode(time, &values.settings);

                // update score stuff now that gamemode has been updated
                self.update_score_values();

                // then a few other things
                self.check_complete();
                self.update_gamemode_type(time);
                self.handle_pending(&values.settings);
            },

            _ => {
                self.state.song_time = values.values.song.position;

                self.check_time_jump(&values.settings);
                self.check_pause_and_restart(actions);

                // get the time with offsets
                let time = self.time();

                self.state.update_hit_timings(time);
                #[cfg(feature="ui")] self.update_ui(time, font_contexts);
                #[cfg(feature="gameplay")] self.check_leadin(&values.settings);
                #[cfg(feature="gameplay")] self.update_scores(values);
                #[cfg(feature="gameplay")] self.update_timing_points(time);
                #[cfg(feature="gameplay")] self.update_spectators(time);

                #[cfg(feature="graphics")] self.animation.update(time);

                // update gamemode
                self.update_gamemode(time, &values.settings);

                // update score stuff now that gamemode has been updated
                self.update_score_values();

                self.check_complete();
                self.check_failed(time);
                self.update_gamemode_type(time);
                self.handle_pending(&values.settings);
                self.update_values(values);
            }
        }

        actions.extend(self.actions.take());
    }

    #[cfg(feature="graphics")]
    pub fn draw(&mut self, list: &mut graphics::RenderableCollection) {
        let time = self.time();

        // draw animation below gamemode
        self.animation.draw(list);

        // draw gamemode
        self.draw_gamemode(time, list);

        // dont draw score, combo, etc if this is a menu bg
        match self.gameplay_type_small {
            GameplayTypeSmall::Preview | GameplayTypeSmall::Simulating => {}
            _ => self.draw_ui(time, list),
        }
    }


    #[cfg(feature="gameplay")]
    pub fn skip_intro(&mut self) {
        let Some(mut time) = self.gamemode.skip_intro(self.time())
        else { return };

        // really not sure whats happening here lol
        if self.state.lead_in_time > 0.0 && time > self.state.lead_in_time {
            time -= self.state.lead_in_time - 0.01;
            self.state.lead_in_time = 0.01;
        }

        self.actions.push(SongAction::SetPosition(time).into());
    }

    pub fn on_complete(&mut self) {
        #[cfg(feature="graphics")] {
            // make sure the cursor is visible
            self.actions.push(CursorAction::SetVisible(true).into());
            // undo any cursor override
            self.actions.push(CursorAction::OverrideRippleRadius(None).into());
        }

        #[cfg(feature="gameplay")]
        if let GameplayType::Spectator {
            buffered_score_frames,
            ..
        } = &mut *self.gameplay_type {
            // if we have a score frame we havent dealt with yet, its most likely the score frame sent once the map has ended
            if !buffered_score_frames.is_empty() {
                self.score.score = buffered_score_frames.last().cloned().unwrap().1;
            }

            // let mut score_menu = ScoreMenu::new(&manager.score, manager.metadata.clone(), false);
            // score_menu.dont_close_on_back = true;
            // self.score_menu = Some(score_menu);
        }
    }

    // can be from either paused or new
    pub fn start(&mut self) {
        #[cfg(feature="graphics")] {
            // set bounds
            let event = if let Some(bounds) = self.state.fit_to_bounds {
                GameplayEvent::SetBounds {
                    bounds,
                    full_window: false
                }
            } else {
                GameplayEvent::SetBounds {
                    bounds: Bounds::new(Vector2::ZERO, self.state.window_size),
                    full_window: true
                }
            };
            self.gamemode.handle_gameplay_event(event);

            // set cursor visibility
            self.actions.push(CursorAction::SetVisible(
                !self.should_hide_cursor()
            ).into());
        }

        self.state.pause_pending = false;
        self.state.should_pause = false;

        // offset our start time by the duration of the pause
        if let Some(pause_time) = self.state.pause_start.take() {
            self.state.start_time += chrono::Utc::now().timestamp() - pause_time;
        }

        if !self.state.started {
            self.reset();

            //TODO: probably want to skip other things as well
            #[cfg(feature="gameplay")]
            if !self.gameplay_type_small.is_replay() {
                self.outgoing_spectator_frame(|this| SpectatorFrame::new(
                    0.0,
                    SpectatorAction::Play {
                        beatmap_hash: this.beatmap.hash(),
                        mode: this.gamemode_properties.playmode().to_string(),
                        mods: this.score.mods.clone(),
                        speed: this.mods.speed.as_u8(),
                        map_game: this.metadata.beatmap_type.into(),
                        map_link: None
                    })
                );
            }

            // reset lead-in
            self.state.reset_lead_in(self.gameplay_type_small.is_preview());

            // volume is set when the song is actually started (when lead_in_time is <= 0)
            self.state.started = true;

            // run the startup function
            if let Some(on_start)
            = self.on_start.take() {
                on_start(self);
            }
        } else if self.state.lead_in_time <= 0.0 {
            // if this is a preview, dont do anything
            if self.gameplay_type_small.is_preview() { return }

            #[cfg(feature="gameplay")]
            self.outgoing_spectator_frame(|s| SpectatorFrame::new(
                s.time(),
                SpectatorAction::UnPause
            ));
            self.actions.push(SongAction::Play.into());
            self.gamemode.handle_gameplay_event(GameplayEvent::UnPaused);
        }


        // re init ui
        #[cfg(feature="graphics")] self.layout_ui();
    }

    pub fn pause(&mut self) {
        #[cfg(feature="graphics")] {
            // make sure the cursor is visible
            self.actions.push(CursorAction::SetVisible(true).into());
            // undo any cursor override
            self.actions.push(CursorAction::OverrideRippleRadius(None).into());
        }

        self.actions.push(SongAction::Pause.into());
        self.state.pause_start = Some(chrono::Utc::now().timestamp());

        // is there anything else we need to do?

        // might mess with lead-in but meh
        #[cfg(feature="gameplay")]
        self.outgoing_spectator_frame_force(
            |this| SpectatorFrame::new(this.time(), SpectatorAction::Pause),
        );

        self.gamemode.handle_gameplay_event(GameplayEvent::Paused);
    }
    pub fn reset(&mut self) {
        self.health.reset();
        self.key_counter.reset();
        self.timing_points.reset();
        self.gamemode.reset(&self.beatmap);
        self.state.reset(self.mods.get_speed());

        if self.gameplay_type_small.is_preview() {
            self.gamemode.handle_gameplay_event(GameplayEvent::ApplyMods(
                self.mods.clone()
            ));
        } else {
            // reset song
            self.actions.push(SongAction::Restart.into());
            self.actions.push(SongAction::Pause.into());
            self.actions.push(SongAction::SetPosition(0.0).into());
            self.actions.push(SongAction::SetRate(self.game_speed()).into());
        }

        if self.map_diff == 0.0 {
            self.actions.push(GameAction::from((
                self.state.id.clone(),
                GameplayAction::RequestDifficulty
            )).into());
        }

        self.score = IngameScore::new(
            Score {
                speed: self.mods.speed,
                mods: self.gamemode_properties.filter_mods(&self.mods),
                replay: Some(Replay::new()),
                judgments: self.judgments
                    .iter()
                    .map(|j| (j.id.to_owned(), 0))
                    .collect(),

                ..Score::new(
                    self.beatmap.hash(),
                    self.score.username.clone(),
                    self.gamemode_properties.playmode().to_string()
                )
            },
            true,
            false
        );

        // reset elements
        #[cfg(feature="graphics")] {
            self.judgement_indicators.clear();
            for e in self.ui_elements.elements_mut() {
                e.reset_element();
            }
        }

        #[cfg(feature="gameplay")]
        if self.gameplay_type.should_load_scores() {
            self.actions.push(GameAction::RefreshScores.into());
        }

    }
    fn fail(&mut self) {
        if self.failed()
        || self.mods.has_nofail()
        || self.mods.has_autoplay()
        || self.gameplay_type_small.is_preview()
        || self.gameplay_type_small.is_multi() {
            return
        }

        debug!("failed");
        self.state.failed = Some(self.time());
    }

    pub fn force_update_settings(&mut self, settings: &Settings) {
        self.gamemode.force_update_settings(settings);
        if self.state.global_offset != settings.global_offset {
            self.state.global_offset = settings.global_offset;
            self.increment_global_offset(0.0);
        }
    }

    fn increment_offset(&mut self, delta: f32) {
        self.beatmap_preferences.audio_offset += delta;
        // #[cfg(feature="graphics")]
        // self.center_text_helper.set_value(
        //     format!("Offset: {:.2}ms", self.beatmap_preferences.audio_offset),
        //     self.time()
        // );

        // update the beatmap offset
        let prefs = self.beatmap_preferences.clone();
        let hash = self.beatmap.hash();
        self.actions.push(actions::database::Action::SaveBeatmapPreferences {
            hash,
            prefs
        }.into());
    }
    fn increment_global_offset(&mut self, delta: f32) {
        self.state.global_offset += delta;
        // #[cfg(feature="graphics")]
        // self.center_text_helper.set_value(
        //     format!("Global Offset: {:.2}ms", self.global_offset),
        //     self.time()
        // );
    }
}

// update fns
impl GameplayManager {

    #[cfg(feature="gameplay")]
    fn update_scores(&mut self, values: &mut ValueCollection) {
        if !self.gameplay_type.should_load_scores() { return }

        let list = &values.score_list;

        if !self.score_list.loaded
            && self.gameplay_type.should_load_scores()
            && list.loaded
        {
            self.score_list = list.clone();

            for s in self.score_list.scores.iter_mut() {
                if s.username == self.score.username {
                    s.score_type = ScoreType::Previous;
                }
            }
        }
    }

    #[cfg(feature="ui")]
    fn update_ui(
        &mut self,
        time: f32,
        font_contexts: &mut TextLayoutContexts,
    ) {
        // update ui editor
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
                            .elements_mut()
                            .find(|i|
                                i.name == action.target
                            )
                        else { continue };

                        match action.action {
                            GameplayWidgetActionType::Add(layout) => {
                                ele.layout = Some(layout);
                                ele.visible = true;
                                self.layout_ui();
                            }
                            GameplayWidgetActionType::Move(layout) => {
                                ele.layout = Some(layout);
                                self.layout_ui();
                            }
                            GameplayWidgetActionType::Remove => {
                                ele.visible = false;
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
        if !self.gameplay_type_small.is_preview() {
            let mut ui_elements = self.ui_elements.take();
            let mut shell = GameplayWidgetUpdateShell {
                manager: self,
                font_context: font_contexts,
                scale: Vector2::ONE
            };

            for ui in ui_elements.elements_mut() {
                ui.update(&mut shell);
            }
            self.ui_elements = ui_elements;
        }

        // update hit timings bar

        // update judgement indicators
        self.judgement_indicators.retain(|a| a.should_keep(time));
    }

    #[cfg(feature="gameplay")]
    fn update_spectators(&mut self, time: f32) {
        if time >= self.spectator_info.last_score_sync + SPECTATOR_SCORE_SYNC_INTERVAL {
            self.spectator_info.last_score_sync = time;

            let our_score = &mut self.score.score;

            // create and send the packet
            let replay = our_score.replay.take();
            let score = our_score.clone();
            our_score.replay = replay;

            self.outgoing_spectator_frame(|_| SpectatorFrame::new(
                time,
                SpectatorAction::ScoreSync { score })
            );
        }
    }

    // TODO: this is kinda shit
    fn update_values(&mut self, values: &mut ValueCollection) {
        if *self.state.id != u32::MAX { return }

        // TODO: placing
        let score = &mut values.score;

        if score.time != self.score.time {
            *score = ReflectScore::new(&self.score, self.gamemode_properties.info);
        } else {
            score.update(&self.score);
        }
    }

    fn check_pause_and_restart(&mut self, actions: &mut actions::ActionQueue) {
        // check map restart
        if let Some(press_time) = self.state.restart_hold_start
        && press_time.as_millis() >= self.gameplay_settings.map_restart_delay {
            self.reset();
            actions.extend(self.actions.take());
            return;
        }

        // check pause
        if self.state.pause_pending && !self.in_break() {
            self.state.pause_pending = false;
            self.state.should_pause = true;
            self.pause();
        }
    }

    fn check_time_jump(&mut self, settings: &Settings) {
        // make sure we jump to the time we're supposed to be at
        let Some(time) = self.state.pending_time_jump.take()
        else { return };

        let mut state = create_update_state!(self, time, settings);
        self.gamemode.time_jump(time, &mut state);
    }

    fn check_leadin(&mut self, settings: &Settings) {
        if self.state.lead_in_time <= 0.0 { return }

        let elapsed = self.state.lead_in_timer.elapsed_and_reset();
        self.state.lead_in_time -= elapsed * self.game_speed();

        use actions::song::SongAction as SongAction;
        if self.state.lead_in_time <= 0.0 {
            self.actions.push(SongAction::SetRate(self.game_speed()).into());
            self.actions.push(SongAction::SetVolume(settings.get_music_vol()).into());
            self.actions.push(SongAction::SetPosition(-self.state.lead_in_time).into());
            self.actions.push(SongAction::Play.into());
            self.state.lead_in_time = 0.0;
        }
    }

    fn update_timing_points(&mut self, time: f32) {
        for tp_update in self.timing_points.update(time) {
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
    }

    fn update_gamemode(
        &mut self,
        time: f32,
        settings: &Settings,
    ) {
        let mut state = create_update_state!(self, time, settings);

        self.gamemode.update(&mut state);
        for action in state.actions {
            self.handle_gamemode_action(action, settings);
        }
    }

    fn update_score_values(&mut self) {
        let info = self.gamemode_properties.info;
        self.score.accuracy = info.calc_acc(&self.score);
        self.score.performance = info.calc_perf(gameplay::info::CalcPerfInfo {
            score: &self.score,
            map_difficulty: self.map_diff,
            accuracy: self.score.accuracy
        });
        // self.score.take_snapshot(time, self.health.get_ratio());
    }

    // TODO: handle edge cases, like replays, spec, autoplay, etc
    fn check_failed(&mut self, time: f32) {
        #[cfg(feature="gameplay")]
        if let Some(failed_time) = self.state.failed && !self.gameplay_type_small.is_multi() {
            let new_rate = f32::lerp(
                self.game_speed(),
                0.0,
                (time - failed_time) / 1000.0
            );

            if new_rate <= 0.05 {
                self.actions.push(SongAction::Pause.into());
                // self.song.pause();

                self.state.completed = true;
                // self.outgoing_spectator_frame_force((self.end_time + 10.0, SpectatorAction::Failed));
                trace!("show fail menu");
            } else {
                self.actions.push(SongAction::SetRate(new_rate).into());
            }
        }
    }

    fn check_complete(&mut self) {
        #[allow(clippy::collapsible_if, reason="features")]
        if !self.state.completed { return }

        #[cfg(feature="gameplay")] {
            let mut score = self.score.score.clone();
            score.replay = None;

            let time = self.state.end_time + 10.0;
            self.outgoing_spectator_frame_force(|_| SpectatorFrame::new(
                time,
                SpectatorAction::ScoreSync { score }
            ));

            self.outgoing_spectator_frame_force(|_| SpectatorFrame::new(
                time,
                SpectatorAction::Buffer
            ));

            if self.gameplay_type_small.is_multi() {
                self.actions.push(actions::multiplayer::LobbyAction::MapComplete(
                    Box::new(self.score.score.clone())
                ).into());
            }
        }

        // check if we failed
        if self.health.is_dead(true) && !self.failed() {
            self.fail();
        }
    }

    fn update_gamemode_type(&mut self, time: f32) {
        match &mut *self.gameplay_type {
            // read inputs from replay if replaying
            GameplayType::Replaying {
                score,
                current_frame
            }
            | GameplayType::Simulating {
                score,
                current_frame
            } => {
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
            GameplayType::Spectator {
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
                    .clamp(0.0, self.state.end_time);

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
            GameplayType::Multiplayer {
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
    }

    fn handle_pending(&mut self, settings: &Settings) {
        // handle any pending gameplay actions
        for a in self.gameplay_actions.take() {
            self.handle_action(a, settings);
        }

        // handle any pending frames
        if !self.pending_frames.is_empty() {
            for ReplayFrame {
                time,
                action
            } in self.pending_frames.take() {
                self.handle_frame(
                    action,
                    true,
                    Some(time),
                    true,
                    settings
                );
            }
        }
    }
}

// graphics
#[cfg(feature="graphics")]
impl GameplayManager {
    pub fn init_ui(&mut self, font_contexts: &mut TextLayoutContexts) {
        let layouts =
            std::fs::read("ui_layouts.json").ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();

        let info = self.gamemode_properties.info;

        // TODO: would be nice to make widgets from all gamemodes available (don-chan in osu!?)
        let widgets = interface::DEFAULT_GAMEPLAY_WIDGETS
            .iter()
            .chain(info.available_widgets)
            .cloned()
            .collect::<Vec<_>>();

        let mut loader = UiElementLoader::new(
            self.gamemode_properties.playmode(),
            layouts,
            widgets,

            *info,
            self.gameplay_settings.clone()
        );

        for i in interface::DEFAULT_GAMEPLAY_WIDGETS
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
        self.ui_elements.add_elements(loader.elements);

        // layout will be performed on game start (and skin load)
    }

    fn layout_ui(&mut self) {
        if let Err(e) = self.ui_elements.layout(
            self.gamemode.get_playfield().bounds,
            self.state.window_size
        ) {
            error!("error laying out ui elements! {e:?}");
        }

        if let Some(channels) = &self.editor {
            for i in self.ui_elements.elements() {
                let r = channels
                    .event_sender
                    .send(GameplayWidgetEvent {
                    target: Some(i.name.to_string()),
                    action: GameplayWidgetEventType::Update {
                        bounds: i.resolved_bounds(),
                    },
                });

                if r.is_err() {
                    self.editor = None;
                    break;
                }
            }
        }
    }

    fn draw_ui(
        &mut self,
        time: f32,
        list: &mut graphics::RenderableCollection,
    ) {
        // judgement indicators
        for indicator in self
            .judgement_indicators
            .iter()
        {
            indicator.draw(time, list);
        }

        let mut shell = GameplayWidgetDrawShell {
            transform: tataku::Matrix::identity(),
            list,
        };

        // ui elements
        for i in self.ui_elements.elements() {
            i.draw(&mut shell);
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

    fn draw_gamemode(
        &mut self,
        time: f32,
        list: &mut graphics::RenderableCollection,
    ) {
        // if let Some(bounds) = self.fit_to_bounds {
        //     list.push_scissor(bounds.into_scissor());
        // todo: is this still necessary?
        // }

        let shell = GameplayDrawShell {
            time,
            gameplay_type: &self.gameplay_type_small,
            current_timing_point: self.timing_points.timing_point(),
            mods: &self.mods,
            score: &self.score,
            window_size: self.state.window_size
        };
        self.gamemode.draw(shell, list);

        // if self.fit_to_bounds.is_some() {
        //     list.pop_scissor();
        // }
    }


    pub fn reload_skin(&mut self, skin_manager: &mut SkinManager) {
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
                self.animation.window_size_changed(self.state.window_size);
            }
        }

        let mut shell = GameplayWidgetReloadSkinShell {
            source: &source,
            skin_manager,
        };

        for i in self.ui_elements.elements_mut() {
            i.reload_skin(&mut shell);
        }

        self.layout_ui();
    }

    pub fn cleanup_textures(&mut self, skin_manager: &mut dyn graphics::SkinProvider) {
        // drop all texture references by dropping the gamemode
        // this should be fine since we shouldnt be re-using this gamemode at this time anyways
        self.gamemode = Box::new(engine::gameplay::default::NoMode);
        self.gamemode_properties = self.gamemode.properties(&self.timing_points);
        skin_manager.free_by_usage(graphics::SkinUsage::Beatmap);

        let path = self.beatmap
            .get_parent_dir()
            .unwrap()
            .to_string_lossy()
            .to_string();

        skin_manager.free_by_source(graphics::TextureSource::Beatmap(path));
    }

}

// Getters, Setters, Properties
impl GameplayManager {
    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn set_id(&mut self, id: GameplayId) {
        // make sure we dont add a reference count to our copy of the id
        // this makes sure things are cleaned up properly when the manager is dropped
        self.state.id = Arc::new(*id);
    }

    pub(crate) fn started(&self) -> bool {
        self.state.started
    }
    pub(crate) fn completed(&self) -> bool {
        self.state.completed
    }
    pub(crate) fn failed(&self) -> bool {
        self.state.failed.is_some()
    }
    pub(crate) fn start_time(&self) -> i64 {
        self.state.start_time
    }

    /// is this game pausable
    pub fn can_pause(&mut self) -> bool {
        // never allow pausing in multi
        #[cfg(feature="gameplay")]
        if self.gameplay_type_small.is_multi() { return false; }
        self.state.should_pause
        || !(
            self.mods.has_autoplay()
            || self.gameplay_type_small.is_replay()
            || self.failed()
        )
    }

    pub fn should_pause(&self) -> bool {
        self.state.should_pause
    }

    fn in_break(&self) -> bool {
        let time = self.time();

        fn check(event: &BeatmapEvent, time: f32) -> bool {
            #[allow(irrefutable_let_patterns, reason = "more events will be added eventually")]
            let BeatmapEvent::Break { start, end } = event
            else { return false };

            time >= *start && time < *end
        }

        self.beatmap_events
            .iter()
            .any(|e| check(e, time))
    }

    #[inline]
    fn game_speed(&self) -> f32 {
        if self.gameplay_type_small.is_preview() {
            1.0 // TODO:
        } else {
            self.mods.get_speed()
        }
    }

    fn should_hide_cursor(&self) -> bool {
        if self.gameplay_type_small.is_preview()
        || self.gameplay_type_small.is_replay()
        || self.mods.has_autoplay() {
            false
        } else {
            !self.gamemode_properties.show_cursor
        }
    }

    pub fn should_save_score(&self) -> bool {
        !(
            self.gameplay_type_small.is_replay()
            || self.mods.has_autoplay()
            || self.state.unrankable
        )
    }

    pub fn update_difficulty(
        &mut self,
        provider: &mut dyn engine::database::DifficultyProvider
    ) {
        self.map_diff = provider.get_diff(
            &self.beatmap.get_beatmap_meta(),
            self.gamemode_properties.playmode(),
            &self.mods
        ).unwrap_or_default();

        trace!("Updated diff: {}", self.map_diff);
    }
}

// Input/Event Handlers
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
            if self.gameplay_type_small.is_multi() {
                self.actions.push(LobbyAction::SendSkipRequest.into());
            } else {
                self.skip_intro();
            }

            // more to do?
            return;
        }

        let add_frames = !(
            self.mods.has_autoplay() || self.gameplay_type_small.is_replay()
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
                    |_| SpectatorFrame::new(time, SpectatorAction::ReplayAction {
                        action: frame.action
                    }),
                );
            }
        }
    }

    pub fn add_action(&mut self, action: GameplayAction) {
        self.gameplay_actions.push(action);
    }
    fn handle_action(
        &mut self,
        action: GameplayAction,
        settings: &engine::Settings,
    ) {
        match action {
            GameplayAction::Pause => self.pause(),
            GameplayAction::Resume => self.start(),
            GameplayAction::JumpToTime {
                time,
                skip_intro
            } => {
                if skip_intro {
                    self.state.lead_in_time = 0.0;
                }

                self.state.pending_time_jump = Some(time);
                self.actions.push(SongAction::SetPosition(time).into());
            },

            GameplayAction::ApplyMods(mut mods) => {
                if self.gameplay_type_small.is_preview() {
                    mods.add_mod(Autoplay);
                }

                self.mods = Arc::new(mods);
                self.gamemode.handle_gameplay_event(GameplayEvent::ApplyMods(
                    self.mods.clone()
                ));
            },

            GameplayAction::FitToArea(bounds) => {
                // info!("fitting to area: {bounds:?}");
                self.state.fit_to_bounds = Some(bounds);
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
            },

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
        action: gameplay::Action,
        settings: &engine::Settings
    ) {
        match action {
            gameplay::Action::AddJudgment(judgment) => {

                // increment judgment, if applicable
                if let Some(count) = self.score.judgments.get_mut(judgment.id) {
                    *count += 1;
                }

                // do score
                let combo_mult = (
                    self.score.combo as f32
                    * self.mods.score_multiplier
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
                    AffectsCombo::Reset => {
                        self.handle_gamemode_action(
                            gameplay::action::GamemodeAction::ComboBreak,
                            settings
                        );
                    },
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
                if self.mods.has_sudden_death() && judgment.fails_sudden_death {
                    // TODO: change the judgment to a miss
                    self.fail();
                }
                if self.mods.has_perfect() && judgment.fails_perfect {
                    self.fail();
                }
            }

            #[cfg(feature="gameplay")]
            gameplay::Action::PlayHitsound {
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


            gameplay::Action::AddTiming {
                hit_time,
                note_time
            } => {
                let diff = hit_time - note_time;
                self.score.insert_stat(gameplay::stats::HitVarianceStat, diff);

                self.state.hit_timings.push(HitTiming::new(hit_time, diff));
            }

            gameplay::Action::AddStat {
                stat,
                value
            } => self.score.insert_stat(stat, value),

            gameplay::Action::ComboBreak => {
                // play hitsound
                #[cfg(feature="gameplay")]
                if self.score.combo >= 20 && !self.gameplay_type_small.is_preview() {
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
            gameplay::Action::FailGame => self.fail(),
            gameplay::Action::ReplayAction(frame) => self.handle_frame(
                frame.action,
                true,
                Some(frame.time),
                true,
                settings
            ),


            gameplay::Action::ResetHealth => self.health.reset(),
            gameplay::Action::ReplaceHealth(new_health)
                => self.health = new_health,
            gameplay::Action::MapComplete => self.state.completed = true,


            #[cfg(feature="graphics")]
            gameplay::Action::AddIndicator(
                mut indicator
            ) => {
                indicator.set_start_time(self.time());
                indicator.set_draw_duration(
                    self.gameplay_settings.hit_indicator_draw_duration,
                    settings
                );
                self.judgement_indicators.push(indicator);
            }

            #[cfg(feature="graphics")]
            gameplay::Action::RemoveLastJudgment => {
                self.judgement_indicators.pop();
            }

            #[cfg(feature="graphics")]
            gameplay::Action::PlayfieldChanged => {
                if self.animation.use_gamemode_playfield(
                    self.gamemode_properties.info
                ) {
                    self.animation.fit_to_area(self.gamemode.get_playfield());
                }

                self.layout_ui();
            }

            #[cfg(not(all(feature="graphics", feature="gameplay")))]
            _ => {}
        }
    }

    #[cfg(feature="gameplay")]
    fn should_skip_input(&self) -> bool {
        // never skip input for multi, because you can keep playing if you failed
        if self.gameplay_type_small.is_multi() { return false }
        self.failed() || self.gameplay_type.skip_input()
    }

    #[cfg(feature="gameplay")]
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
                if key == self.gameplay_settings.map_restart_key {
                    self.state.restart_hold_start = None;
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

    #[cfg(feature="gameplay")]
    fn key_down(
        &mut self,
        key_input: &KeyInput,
        mods: KeyModifiers,
        settings: &Settings,
    ) -> bool {
        if key_input.repeat { return false }
        let Some(key) = key_input.as_key() else { return false };

        if (self.gameplay_type_small.is_replay() || self.mods.has_autoplay())
            && !self.gameplay_type_small.is_preview()
        {
            // check replay-only keys
            if key == Key::Escape {
                self.state.started = false;
                self.state.completed = true;
                return true;
            }
        }

        // check map restart key
        if key == self.gameplay_settings.map_restart_key
            && !self.gameplay_type_small.is_multi()
        {
            self.state.restart_hold_start = Some(tataku::Instant::now());
            return true;
        }

        if self.failed() && key == Key::Escape && !self.gameplay_type_small.is_multi() {
            // set the failed time to negative, so it triggers the end
            self.state.failed = Some(-1000.0);
        }

        if self.should_skip_input() { return false }


        if key == Key::Escape {
            if self.can_pause() {
                self.state.should_pause = true;
            } else if let GameplayType::Multiplayer { last_escape_press, .. } = &mut *self.gameplay_type {
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
        if key == input::Key::F9 {
            if self.editor.is_some() {
                self.editor = None;
                if !self.gamemode_properties.show_cursor {
                    self.actions.push(CursorAction::SetVisible(false).into());
                }
                return true;
            }


            // ensure autoplay is enabled
            self.state.unrankable = true;
            if !self.mods.has_autoplay() {
                let mut mods = (*self.mods).clone();
                mods.add_mod(mods::Autoplay);
                self.gameplay_actions.push(GameplayAction::ApplyMods(mods));
            }

            let (
                event_sender,
                event_receiver
            ) = std::sync::mpsc::channel();
            let (
                action_sender,
                action_receiver
            ) = std::sync::mpsc::channel();

            let editor = GameplayWidgetEditor::new(
                self.ui_elements.elements(),
                action_sender,
                event_receiver
            );

            self.actions.push(actions::menu::MenuAction::AddDialogRaw {
                dialog: Box::new(editor),
                options: Box::new(actions::menu::DialogCreateOptions {
                    background: false,
                    ..Default::default()
                })
            }.into());

            self.editor = Some(EditorChannels {
                event_sender,
                action_receiver: Arc::new(Mutex::new(action_receiver)),
            });

            self.actions.push(actions::cursor::CursorAction::SetVisible(true).into());
            return true;
        }

        // check for offset changing keys
        if mods.shift {
            let mut t = 0.0;
            if key == self.gameplay_settings.key_offset_up { t = 5.0 }
            if key == self.gameplay_settings.key_offset_down { t = -5.0 }

            if t != 0.0 {
                self.increment_global_offset(t);
                return true;
            }
        } else {
            if key == self.gameplay_settings.key_offset_up {
                self.increment_offset(5.0);
                return true;
            }
            if key == self.gameplay_settings.key_offset_down {
                self.increment_offset(-5.0);
                return true;
            }
        }


        // skip intro
        if key == input::Key::Space {
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
        self.state.window_size = window_size;
        if self.state.fit_to_bounds.is_none() {
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

    #[cfg(feature="graphics")]
    pub fn window_focus_changed(&mut self, got_focus: bool) {
        // info!("window focus changed");
        if got_focus {
            self.state.pause_pending = false;
        } else if self.can_pause() {
            if self.in_break() {
                self.state.pause_pending = true;
            } else {
                self.state.should_pause = true;
            }
        }
    }
}

// Spectator Stuff
#[cfg(feature="gameplay")]
impl GameplayManager {
    fn outgoing_spectator_frame(
        &mut self,
        frame: impl FnOnce(&Self) -> SpectatorFrame,
    ) {
        if !self.gameplay_type.should_send_spec_frames() { return }
        self.actions.push(actions::online::OnlineAction::SendSpectatorFrame {
            frame: Box::new(frame(self)),
            force: false
        }.into());
    }

    fn outgoing_spectator_frame_force(
        &mut self,
        frame: impl FnOnce(&Self) -> SpectatorFrame,
    ) {
        if !self.gameplay_type.should_send_spec_frames() { return }
        self.actions.push(actions::online::OnlineAction::SendSpectatorFrame {
            frame: Box::new(frame(self)),
            force: true
        }.into());
    }

    pub fn add_spec_frame(&mut self, frame_host_id: u32, frame: SpectatorFrame) {
        let GameplayType::Spectator {
            frames,
            host_id,
            ..
        } = &mut *self.gameplay_type else { return };

        if *host_id == frame_host_id {
            frames.push_back(frame);
        }
    }
}

impl GameplayManagerTrait for GameplayManager {
    fn end_time(&self) -> f32 { self.state.end_time }

    fn score(&self) -> &IngameScore { &self.score }
    fn score_mut(&mut self) -> &mut IngameScore { &mut self.score }
    fn mods(&self) -> &ModManager { &self.mods }
    fn metadata(&self) -> &BeatmapMeta { &self.metadata }
    fn key_counter(&self) -> &KeyCounter { &self.key_counter }
    fn spectators(&mut self) -> &mut engine::online::SpectatorList { &mut self.spectator_info.spectators }
    fn judgments(&self) -> &Vec<HitJudgment> { &self.judgments }
    fn health(&self) -> &dyn HealthManager { &*self.health }
    fn hit_timings(&self) -> &Vec<HitTiming> { &self.state.hit_timings }
    fn timing_points(&self) -> &TimingPointHelper { &self.timing_points }

    fn properties(&self) -> &GamemodeProperties { &self.gamemode_properties }

    fn bounds(&self) -> Bounds {
        self.state.fit_to_bounds.unwrap_or(Bounds::new(Vector2::ZERO, self.state.window_size))
    }

    fn all_scores(&self) -> Vec<&IngameScore> {
        let mut list = self.score_list
            .scores
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
        &self.score_list.scores
    }

    #[inline]
    fn time(&self) -> f32 {
        self.state.time(&self.beatmap_preferences)
    }

    /// using a getter for this since we dont want anything to directly change it
    fn get_mode(&self) -> GameplayTypeSmall { self.gameplay_type_small }
    fn set_mode(&mut self, mode: GameplayType) {
        // println!("setting gameplay mode to {mode:?}");


        match &mode {
            GameplayType::Normal => {
                // dont think there's anything to do for this one, since its the default
            }

            GameplayType::Replaying { score, .. }
            | GameplayType::Simulating { score, .. } => {
                // load speed from score
                self.mods = Arc::new(ModManager::new(
                    score.mods.iter(),
                    score.speed,
                    self.gamemode_properties.info
                ));

                self.score.username = score.username.clone();
                self.score.mods = self
                    .mods
                    .filter_mods_for_mode(self.gamemode_properties.info);
            }

            GameplayType::Preview => {
                self.state.lead_in_time = 0.0;
                self.state.pending_time_jump = Some(self.time());

                let mut mods = self.mods.as_ref().clone();
                mods.add_mod(Autoplay);
                self.mods = Arc::new(mods);
            }

            // in a multi match
            #[cfg(feature="gameplay")]
            GameplayType::Multiplayer { .. } => {
                // self.score_loader = None;
            }

            // handling spec
            #[cfg(feature="gameplay")]
            GameplayType::Spectator { host_username, .. } => {
                self.score.username = host_username.to_string();
            }
        }

        *self.gameplay_type = mode;
        self.gameplay_type_small = GameplayTypeSmall::from(&*self.gameplay_type);
    }

}
impl Drop for GameplayManager {
    fn drop(&mut self) {
        if self.gamemode_properties.playmode() != "none" {
            error!("gameplay manager dropped without cleaning up textures !!!!!!!!!!!");
        }
    }
}


#[cfg(feature="graphics")]
struct EditorChannels {
    event_sender: Sender<GameplayWidgetEvent>,
    action_receiver: Arc<Mutex<Receiver<GameplayWidgetAction>>>,
}
