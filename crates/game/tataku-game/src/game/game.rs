use crate::prelude::*;
use tataku_audio::prelude::*;
use chrono::{ Datelike, Timelike };

/// how long transitions between states should last
const TRANSITION_TIME:f32 = 500.0;

#[cfg(feature="dynamic_gamemodes")] pub type IncomingGamemode = GamemodeLibrary;
#[cfg(not(feature="dynamic_gamemodes"))] pub type IncomingGamemode = GamemodeInfo;

pub struct BuiltinMenus {
    pub menus: &'static [(&'static str, &'static [u8])],
    pub dialogs: &'static [(&'static str, &'static [u8])],
}

pub struct Game {
    // engine things
    pub actions: ActionQueue,
    runtime: Rc<tokio::runtime::Runtime>,

    pub(super) current_state: GameState,
    pub(super) queued_state: GameState,
    #[cfg(feature="graphics")] window_event_receiver: AsyncReceiver<WindowEvent>,
    #[cfg(feature="graphics")] mouse_position_receiver: TripleBufferReceiver<Vector2>,
    #[cfg(feature="graphics")] pub(super) window_proxy: winit::event_loop::EventLoopProxy<WindowAction>,

    // managers
    pub song_manager: SongManager,
    pub audio_manager: AudioManager,
    pub(crate) task_manager: TaskManager,
    pub(super) score_manager: ScoreManager,
    pub(super) difficulty_manager: DifficultyManager,
    pub(super) online_content_manager: OnlineContentManager,

    #[cfg(feature="graphics")] pub(super) ui_manager: UiManager,
    #[cfg(feature="graphics")] pub(super) skin_manager: SkinManager,
    #[cfg(feature="graphics")] pub(super) cursor_manager: CursorManager,
    #[cfg(feature="graphics")] pub(super) volume_controller: VolumeControl,
    #[cfg(feature="graphics")] pub(super) custom_menu_manager: CustomMenuManager,
    #[cfg(feature="graphics")] pub(super) text_layout_contexts: TextLayoutContexts,
    #[cfg(feature="graphics")] pub(super) xml_test_manager: Option<XmlTestManager>,
    #[cfg(feature="graphics")] pub(super) notification_manager: NotificationManager,
    #[cfg(feature="graphics")] pub(super) gameplay_managers: HashMap<GameplayId, (GameplayManager, NewManager)>,
    
    #[cfg(feature="gameplay")] pub(super) input_manager: InputManager,
    #[cfg(feature="gameplay")] pub(super) spectator_manager: Option<Box<SpectatorManager>>,
    #[cfg(feature="gameplay")] pub(super) multiplayer_manager: Option<Box<MultiplayerManager>>,
    #[cfg(feature="gameplay")] pub(super) pending_gameplay_manager: Option<Box<GameplayManager>>,


    integrations: Vec<Box<dyn TatakuIntegration>>,

    // fps
    #[cfg(feature="graphics")] fps_display: FpsDisplay,
    #[cfg(feature="graphics")] update_display: FpsDisplay,
    #[cfg(feature="graphics")] render_display: AsyncFpsDisplay,
    #[cfg(feature="graphics")] input_display: AsyncFpsDisplay,

    // misc
    game_start: TatakuInstant,

    #[cfg(feature="graphics")] builtin_menus: BuiltinMenus,
    #[cfg(feature="graphics")] pub(super) background_image: Option<Image>,
    #[cfg(feature="graphics")] wallpapers: Vec<Image>,
    #[cfg(feature="graphics")] background_loader: Option<AsyncLoader<Option<Image>>>,
    // spec_watch_action: SpectatorWatchAction,

    #[cfg(feature="graphics")]
    pub queued_events: Vec<(TatakuEvent, Option<TatakuValue>)>,

    pub values: ValueCollection,
}
impl Game {
    pub fn new(
        #[cfg(feature="graphics")]
        window_event_receiver: tokio::sync::mpsc::Receiver<WindowEvent>,
        #[cfg(feature="graphics")]
        mouse_position_receiver: TripleBufferReceiver<Vector2>,

        #[cfg(feature="graphics")]
        window_proxy: winit::event_loop::EventLoopProxy<WindowAction>,

        audio_engines: Vec<AudioApiInit>,
        gamemodes: Vec<IncomingGamemode>,

        #[cfg(feature="graphics")]
        builtin_menus: BuiltinMenus,
    ) -> Self {
        let settings = Settings::load();
        let infos = GamemodeInfos::new(gamemodes);
        #[cfg(feature = "graphics")]
        let skin_manager = SkinManager::new(&settings);

        let online_content_manager = OnlineContentManager::new(&settings);
        let online_content_engines = online_content_manager.get_capabilities();

        Self {
            actions: ActionQueue::new(),
            runtime: Rc::new(tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
            ),

            // engine
            #[cfg(feature="graphics")] window_proxy,
            #[cfg(feature="graphics")] window_event_receiver,
            #[cfg(feature="graphics")] mouse_position_receiver,
            #[cfg(feature="gameplay")] input_manager: InputManager::default(),
            #[cfg(feature="graphics")] volume_controller: VolumeControl::default(),
            #[cfg(feature="graphics")] background_image: None,
            #[cfg(feature="graphics")] wallpapers: Vec::new(),
            #[cfg(feature="graphics")] builtin_menus,
            #[cfg(feature="gameplay")] spectator_manager: None,
            #[cfg(feature="gameplay")] multiplayer_manager: None,

            difficulty_manager: DifficultyManager,
            online_content_manager,

            song_manager: SongManager::default(),
            audio_manager: AudioManager::init_audio(audio_engines)
                .expect("failed to initialize audio engine!"),
            score_manager: ScoreManager::new(infos.clone()),
            task_manager: TaskManager::default(),


            #[cfg(feature="graphics")] cursor_manager: CursorManager::new(
                skin_manager.skin().clone(),
                settings.cursor_settings.clone(),
            ),
            #[cfg(feature="graphics")] skin_manager,
            #[cfg(feature="graphics")] xml_test_manager: None,
            #[cfg(feature="gameplay")] pending_gameplay_manager: None,
            #[cfg(feature="graphics")] ui_manager: UiManager::default(),
            #[cfg(feature="graphics")] gameplay_managers: HashMap::new(),
            #[cfg(feature="graphics")] custom_menu_manager: CustomMenuManager::default(),
            #[cfg(feature="graphics")] notification_manager: NotificationManager::default(),

            #[cfg(feature="graphics")] text_layout_contexts: TextLayoutContexts::new(),

            integrations: Vec::new(),

            current_state: GameState::None,
            queued_state: GameState::None,
            // spec_watch_action: SpectatorWatchAction::FullMenu,

            // fps
            #[cfg(feature="graphics")] render_display: AsyncFpsDisplay::new(
                "fps",
                3,
                RENDER_COUNT.clone(),
                RENDER_FRAMETIME.clone()
            ),
            #[cfg(feature="graphics")] fps_display: FpsDisplay::new("prepares/s", 2),
            #[cfg(feature="graphics")] update_display: FpsDisplay::new("updates/s", 1),
            #[cfg(feature="graphics")] input_display: AsyncFpsDisplay::new(
                "inputs/s",
                0,
                INPUT_COUNT.clone(),
                INPUT_FRAMETIME.clone()
            ),

            // misc
            game_start: TatakuInstant::now(),
            #[cfg(feature="graphics")] background_loader: None,
            #[cfg(feature="graphics")] queued_events: Vec::new(),

            values: ValueCollection {
                values: TatakuValues::new(&infos, online_content_engines, settings),
                custom: DynMap::default()
            },
        }
    }

    #[cfg(feature="graphics")]
    pub fn make_xml_helper(&mut self, path: String) {
        let mut tester = XmlTestManager::default();
        if tester.load_file(
            path,
            &mut self.ui_manager,
            &mut self.values,
            &mut self.actions,
            &mut self.text_layout_contexts,
        ).is_err() {
            self.ui_manager.set_root(
                EmptyWidget::new_boxed(),
                &mut self.values,
                &mut self.actions,
                &mut self.text_layout_contexts,
            );
        }

        self.xml_test_manager = Some(tester);
    }

    #[cfg(feature="graphics")]
    fn load_custom_menus(&mut self) {
        if self.custom_menu_manager.reload_entries(CustomMenuSource::Any) {
            debug!("Reloading custom menus");
            self.custom_menu_manager.update_values(&mut self.values);

            debug!("Done reloading custom menus");
            return;
        }

        for (entries, entry_type) in [
            (self.builtin_menus.menus, CustomEntryType::Menu),
            (self.builtin_menus.dialogs, CustomEntryType::Dialog),
        ] {
            for (name, data) in entries {
                let _ = self.custom_menu_manager.load_entry_bytes(
                    data,
                    None,
                    CustomMenuSource::Game,
                    entry_type
                ).inspect_err(|e| {
                    error!("Error loading {entry_type:?} {name}: {e}");
                });
            }
        }

        self.custom_menu_manager.update_values(&mut self.values);
        debug!("Done loading custom menus");
    }

    #[cfg(feature="gameplay")]
    pub(super) fn init_online(&mut self) {
        self.values.values.online_manager.start(
            &self.values.values.settings,
            &self.runtime
        );
    }

    #[cfg(feature="graphics")]
    fn init_fonts(&mut self) {
        // init FontAwesome
        {
            let data = std::fs::read(
                "resources/fonts/font_awesome_6_regular.otf"
            ).unwrap();

            let font_context = &mut self
                .text_layout_contexts
                .font;

            let ids = font_context
                .collection
                .register_fonts(data.into(), None);

            font_context.collection.append_generic_families(
                parley::GenericFamily::Emoji, 
                ids.into_iter().map(|(i, _)| i)
            );
        }
    }

    fn init(&mut self) {
        let now = std::time::Instant::now();

        #[cfg(feature="graphics")] {
            self.init_fonts();
            
            // init the default cursor
            self.cursor_manager.handle_cursor_action(
                CursorAction::SetCursorMode(CursorMode::Normal),
                &mut self.text_layout_contexts,
            );

            self.load_custom_menus();
            self.load_theme();
        }

        #[cfg(feature="gameplay")] {
            self.init_online();

            // setup double tap protection
            self.input_manager.set_double_tap_protection(
                self.settings.enable_double_tap_protection
                    .then_some(self.settings.double_tap_protection_duration)
            );
        }

        // new beatmap check task
        self.actions.push(TaskAction::AddTask(Box::new(
            BeatmapDownloadsCheckTask::default()
        )).into());

        let mut settings = self.settings.clone();
        settings.gamemode_settings.build(self.values.global.gamemode_infos.clone());
        settings.init(&mut self.values, "settings".to_string());
        self.settings = settings;

        debug!("game init took {:.2}ms", now.elapsed().as_secs_f32() * 1000.0);




        // download first maps
        {
            const FIRST_MAPS: &[u32] = &[
                75, // disco prince (osu)
                905576, // triumph and regret (mania)
                1605148, // mayday (osu)
                727903, // galaxy collapse (taiko)
            ];

            // check if songs folder is empty
            if std::fs::read_dir(SONGS_DIR).unwrap().count() == 0 {
                // no songs, download some
                for id in FIRST_MAPS {
                    self.download_manager.add_download(Downloadable {
                        filename: format!("{DOWNLOADS_DIR}/{id}.osz"),
                        download_progress: None,
                        download: Arc::new(move ||
                            Downloader::download(DownloadOptions::new(
                                format!("https://cdn.ayyeve.dev/tataku/maps/{id}.osz"),
                                0
                            ))
                        ),
                        on_complete: None,
                    });
                }
            }


            // // TODO: remove when done testing downloads stuff
            // self.download_manager.add_download(Downloadable {
            //     filename: "/tmp/75.osz".to_string(),
            //     download_progress: None,
            //     download: Arc::new(move ||
            //         Downloader::download(DownloadOptions::new(
            //             "https://cdn.ayyeve.dev/tataku/maps/75.osz".to_string(),
            //             0
            //         ))
            //     ),
            //     on_complete: None,
            // });
        }

        self.actions.push(InitGameTask::default().into());
        #[cfg(feature="graphics")]
        self.handle_custom_menu("loading_menu");
    }

    #[cfg(feature="gameplay")]
    pub fn game_loop(mut self) {
        let _guard = self.runtime.enter();
        self.init();

        let mut update_timer = TatakuInstant::now();
        #[cfg(feature="graphics")] let mut draw_timer = TatakuInstant::now();
        let mut last_draw_offset = 0.0;

        let game_start = std::time::Instant::now();
        let mut last_setting_update = None;

        #[cfg(feature="graphics")]
        let mut render_rate   = 1.0 / self.settings.display_settings.fps_target as f32;
        let mut update_target = 1.0 / self.settings.display_settings.update_target as f32;

        let mut settings = self.settings.clone();

        loop {
            // update our settings
            if self.settings != settings {
                #[cfg(feature="graphics")]
                if self.settings.display_settings != settings.display_settings {
                    render_rate = 1.0 / self.settings.display_settings.fps_target as f32;
                    update_target = 1.0 / self.settings.display_settings.update_target as f32;
                    self.window_proxy.send_event(
                        WindowAction::SettingsUpdated(self.settings.display_settings.clone())
                    ).unwrap();
                }

                // update our timer
                last_setting_update = Some(TatakuInstant::now());

                #[cfg(feature="graphics")]
                let skin_changed = self.settings.current_skin != settings.current_skin;

                #[cfg(feature="graphics")]
                if skin_changed {
                    self.skin_manager.change_skin(
                        &self.values.settings.current_skin
                    );

                    for (i, _) in self
                        .gameplay_managers
                        .values_mut()
                    {
                        i.reload_skin(&mut self.skin_manager, &self.values.settings);
                    }
                }

                #[cfg(feature="graphics")]
                if self.settings.theme != settings.theme {
                    self.load_theme();
                }

                if self.settings.server_url != settings.server_url {
                    self.online_manager.reset();
                }

                if self.settings.integrations != settings.integrations {
                    for i in self
                        .integrations
                        .iter_mut()
                    {
                        if let Err(e) = i
                            .check_enabled(&self.values.settings)
                        {
                            warn!("Integration error ({}): {e:?}", i.name());
                        }
                    }
                }

                // update doubletap protection
                self.input_manager.set_double_tap_protection(
                    self.settings.enable_double_tap_protection
                    .then_some(self.settings.double_tap_protection_duration)
                );

                // update game mode with new information
                if let GameState::Ingame(igm) = &mut self.current_state {
                    #[cfg(feature="graphics")]
                    if skin_changed {
                        igm.reload_skin(
                            &mut self.skin_manager,
                            &self.values.settings
                        );
                    }
                    igm.force_update_settings(&self.values.settings);
                }

                #[cfg(feature="graphics")]
                for (i, _) in self.gameplay_managers.values_mut() {
                    i.force_update_settings(&self.values.settings);
                }

                settings = self.settings.clone();
            }

            // wait 100ms before writing settings changes
            if let Some(last_update) = last_setting_update
            && last_update.as_millis() > 500.0 {
                self.settings.clone().save();
                last_setting_update = None;
            }

            // update our instant's time
            set_time(game_start.elapsed());
            let mut now = TatakuInstant::now();

            let update_elapsed = now.duration_since(update_timer).as_secs_f32();
            if update_elapsed >= update_target {
                update_timer = now;
                if self.update() {
                    return
                };

                // re-update the time
                set_time(game_start.elapsed());
                now = TatakuInstant::now();
            }

            if let GameState::Closing = &self.current_state {
                self.close_game();
                return;
            }


            #[cfg(feature="graphics")] {
                const DRAW_DAMPENING_FACTOR:f32 = 0.9;
                let elapsed = now.duration_since(draw_timer).as_secs_f32();
                if elapsed + last_draw_offset >= render_rate {
                    draw_timer = now;
                    last_draw_offset = (elapsed - render_rate)
                        .clamp(-5.0, 5.0)
                        * DRAW_DAMPENING_FACTOR;
                    self.draw();
                }
            }
        }

    }

    /// use this for cleanup, not to tell the game to close
    /// to tell the game to close, set the state to GameState::Closing
    fn close_game(&mut self) {
        warn!("stopping game");

        // cleanup gameplay manager textures
        #[cfg(feature="graphics")] {
            for (i, _) in self.gameplay_managers.values_mut() {
                i.cleanup_textures(&mut self.skin_manager);
            }

            if let Some(i) = self.current_state.get_ingame() {
                i.cleanup_textures(&mut self.skin_manager);
            }
        }
    }

    #[cfg(feature="gameplay")]
    fn update(&mut self) -> bool {
        let elapsed = self.game_start.as_millis();
        self.values.game.time = elapsed;

        #[cfg(feature="graphics")]
        let mut window_size = self.values.game.window_size;
        #[cfg(feature="graphics")]
        while let Ok(e) = self.window_event_receiver.try_recv() {
            match e {
                #[cfg(feature="graphics")]
                WindowEvent::FileDrop(path) => self.handle_file_drop(path),
                WindowEvent::Closed => { self.close_game(); return true }
                WindowEvent::ScreenshotComplete(
                    bytes,
                    size,
                    info
                ) => if let Err(e) = self.finish_screenshot(bytes, size, info) {
                    self.actions.push(Notification::new_error(
                        "Screenshot Error",
                        e
                    ).into());
                }

                WindowEvent::GotFocus => self.input_manager.set_window_focus(true),
                WindowEvent::LostFocus => self.input_manager.set_window_focus(false),
                WindowEvent::Input(i) => self.input_manager.handle_input(i),

                WindowEvent::SizeChanged(new_size) => window_size = new_size,

                WindowEvent::IntegrationsLoaded(mut integrations) => {
                    for i in &mut integrations {
                        if let Err(e) = i.check_enabled(&self.settings) {
                            error!("Error checking integration '{}': {e}", i.name());
                        }
                    }

                    self.integrations = integrations;
                }

                WindowEvent::VsyncModes(modes) => {
                    let current = self.settings.display_settings.vsync;
                    if !modes.contains(&current) {
                        self.actions.push(Notification::new_text(
                            "Unsupported Vsync mode, changing to fallback!",
                            Color::YELLOW,
                            10_000.0
                        ).into());
                        self.settings.display_settings.vsync = current.get_fallback();
                        let _ = self.window_proxy.send_event(WindowAction::SettingsUpdated(
                            self.settings.display_settings.clone()
                        ));
                    }

                    self.values.enums.vsync = modes;
                }

                _ => {}
            }
        }
        // since window resizes can spam and laying out the ui can be slow,
        // sometimes they happen too fast for us to keep up with.
        // so this should make sure things arent delayed because of the spam
        #[cfg(feature="graphics")]
        if self.values.game.window_size != window_size {
            self.values.game.window_size = window_size;
            self.resize_bg();
            self.ui_manager.window_size_changed(window_size, &self.values);

            self.volume_controller.window_size_changed(window_size);
            self.update_display.window_size_changed(window_size);
            self.fps_display.window_size_changed(window_size);
            self.input_display.window_size_changed(window_size);
            self.render_display.window_size_changed(window_size);

            if let Some(manager) = self.current_state.get_ingame() {
                manager.window_size_changed(window_size);
            }
        }

        // check bg loaded
        #[cfg(feature="graphics")]
        if let Some(loader) = self.background_loader.clone()
        && let Some(image) = loader.check() {
            self.background_loader = None;

            // unload the old image so the atlas can reuse the space
            if let Some(old_img) = self.background_image.take() {
                self.actions.push(LoadImage::FreeTexture {
                    tex: *old_img.tex,
                    deferred: false
                }.into());
            }

            self.background_image = image;

            if self.background_image.is_none() && !self.wallpapers.is_empty() {
                self.background_image = Some(self.wallpapers[0].clone());
            }

            self.resize_bg();
        }

        #[cfg(feature="graphics")] self.update_display.increment();

        // update counters
        #[cfg(feature="graphics")] self.fps_display.update();
        #[cfg(feature="graphics")] self.update_display.update();
        #[cfg(feature="graphics")] self.render_display.update();
        #[cfg(feature="graphics")] self.input_display.update();

        // read input events
        let mut input_state = self.handle_inputs();

        // update the cursor
        #[cfg(feature="graphics")]
        self.cursor_manager.update(
            elapsed,
            input_state.mouse_pos
        );



        // update our global values
        {
            let values = &mut self.values;
            values.song.position = self.song_manager.position();

            #[cfg(feature = "ui")]
            if let Some(audio) = self.song_manager.instance() {
                if self.values.song.set_state(audio.get_state()) {
                    let action = match self.values.song.state {
                        AudioState::Stopped
                        | AudioState::Unknown => TatakuEvent::SongEnd,
                        AudioState::Playing => TatakuEvent::SongStart,
                        AudioState::Paused => TatakuEvent::SongPause,
                    };
                    self.actions.push(action.into());
                }
            } else {
                self.values.song.set_state(AudioState::Unknown);
            }
        }

        // update any ingame managers
        #[cfg(feature="graphics")]
        for (a, (manager, _)) in self
            .gameplay_managers
            .iter_mut()
        {
            if Arc::strong_count(a) == 1 {
                manager.cleanup_textures(&mut self.skin_manager);
                continue;
            }

            manager.update(&mut self.values, &mut self.actions);

            if manager.completed {
                manager.on_complete();
            }
        }
        #[cfg(feature="graphics")]
        self.gameplay_managers.retain(|a, _| Arc::strong_count(a) > 1);

        // #[cfg(feature="graphics")]
        // let mut input_state = CurrentInputState {
        //     mouse_pos,
        //     mouse_moved,
        //     scroll_delta,
        //     mouse_down,
        //     mouse_up,
        //     keys_down,
        //     keys_up,
        //     mods,

        //     controller_axes: controller_axis
        //         .into_iter()
        //         .flat_map(|(info, axes)|
        //             axes
        //             .clone()
        //             .into_iter()
        //             .filter_map(move |(axis, state)|
        //                 state.changed.then_some((
        //                     axis,
        //                     state.value,
        //                     info.id,
        //                     info.name.clone()
        //                 ))
        //             )
        //         )
        //         .collect(),

        //     controller_down: controller_down
        //         .into_iter()
        //         .flat_map(|(info, buttons)|
        //             buttons
        //             .into_iter()
        //             .map(move |b| (b, info.id, info.name.clone()))
        //         )
        //         .collect(),

        //     controller_up: controller_up
        //         .into_iter()
        //         .flat_map(|(info, buttons)|
        //             buttons
        //             .into_iter()
        //             .map(move |b| (b, info.id, info.name.clone()))
        //         )
        //         .collect(),
        // };

        #[cfg(feature="graphics")]
        self.ui_manager.update(
            &mut input_state,
            self.queued_events.take(),
            &mut self.values,
            &mut self.actions,
            &mut self.skin_manager,
            &mut self.text_layout_contexts,
        );

        #[cfg(feature="graphics")]
        if let Some(tester) = &mut self.xml_test_manager {
            tester.update(
                &mut self.ui_manager,
                &mut self.values,
                &mut self.actions,
                &mut self.text_layout_contexts,
            );
        }

        // update spec and multi managers
        if let Some(spec) = &mut self.spectator_manager {
            let manager = self.current_state.get_ingame();
            if let Some(manager) = spec.update(
                manager,
                &mut self.values,
                &mut self.actions
            ) {
                self.queue_state_change(GameState::Ingame(manager));
            }
        }
        if let Some(multi) = &mut self.multiplayer_manager {
            let manager = self
                .current_state
                .get_ingame();

            multi.update(
                manager,
                &mut self.values,
                &mut self.actions
            );
        }


        // update score manager
        self.score_manager.update(&mut self.values);

        // update song manager
        self.song_manager.update(&mut self.audio_manager);

        // update download manager
        self.values.download_manager.update(&mut self.actions);

        // update tasks
        let game_state = TaskGameState {
            ingame: self.current_state.is_ingame(),
            game_time: self.game_start.as_millis() as u64,
        };
        self.task_manager.update(
            &mut self.values,
            game_state,
            &mut self.actions
        );

        // run actions
        self.handle_actions(None);

        // run update on current state
        match self.current_state.take() {
            GameState::Ingame(mut manager) => {
                // pause button, or focus lost, only if not replaying
                #[cfg(feature="graphics")]
                if let Some(got_focus) = input_state.window_focus_changed
                && self.settings.display_settings.pause_on_focus_lost {
                    manager.window_focus_changed(got_focus);
                }

                if !manager.failed && manager.can_pause()
                    && (manager.should_pause || input_state.controller_pause)
                {
                    manager.pause();
                    let actions = manager.actions.take();
                    self.handle_actions(Some(actions));

                    self.pending_gameplay_manager = Some(manager);
                    #[cfg(feature="graphics")]
                    self.actions.push(MenuAction::SetMenu {
                        id: "pause_menu".into(),
                    }.into());
                } else {
                    // inputs
                    #[cfg(feature="graphics")]
                    for input in input_state.into_events() {
                        manager.handle_input(input, &self.settings);
                    }

                    // update, then check if complete
                    manager.update(&mut self.values, &mut self.actions);
                    if manager.completed {
                        #[cfg(feature="graphics")]
                        self.ingame_complete(manager);
                        // a menu is queued up, we dont need to reapply current_state
                    } else {
                        self.current_state = GameState::Ingame(manager);
                    }
                }
            }

            GameState::TransitionStarting {
                into,
                from,
                timer
            } => {
                if elapsed - timer > TRANSITION_TIME / 2.0 {
                    match *into {
                        GameState::Ingame(mut g) => {
                            #[cfg(feature="graphics")]
                            g.reload_skin(
                                &mut self.skin_manager,
                                &self.values.settings
                            );

                            // let trans = self.transition.take();
                            let elapsed = self.game_start.as_millis();
                            self.queue_state_change(GameState::TransitionEnding {
                                state: Box::new(GameState::Ingame(g)),
                                timer: elapsed,
                            });
                        }
                        #[cfg(feature="graphics")]
                        GameState::SetMenu(menu) => {
                            self.ui_manager.set_root(
                                menu,
                                &mut self.values,
                                &mut self.actions,
                                &mut self.text_layout_contexts,
                            );
                            self.ui_manager.reload_skin(
                                &mut self.values,
                                &mut self.actions,
                                &mut self.skin_manager,
                                &mut self.text_layout_contexts,
                            );

                            let elapsed = self.game_start.as_millis();
                            self.current_state = GameState::TransitionEnding {
                                state: Box::new(GameState::InMenu),
                                timer: elapsed
                            };
                        }

                        other => self.current_state = GameState::TransitionEnding {
                            state: Box::new(other),
                            timer: elapsed
                        },
                    }
                    // let trans = self.transition.take();
                    // self.transition_timer = elapsed;
                } else {
                    self.current_state = GameState::TransitionStarting {
                        from,
                        into,
                        timer,
                    };
                }
            }

            GameState::TransitionEnding {
                state,
                timer
            } => {
                if elapsed - timer > TRANSITION_TIME / 2.0 {
                    self.current_state = *state;
                } else {
                    self.current_state = GameState::TransitionEnding {
                        state,
                        timer
                    };
                }
            }

            other => self.current_state = other
        }

        // update game mode
        match &self.queued_state {
            // queued mode didnt change, set the unlocked's mode to the updated mode
            GameState::None => {} //self.current_state = current_state,
            GameState::Closing => {
                self.settings.clone().save();
                self.current_state = GameState::Closing;
                #[cfg(feature="graphics")]
                let _ = self.window_proxy.send_event(WindowAction::CloseGame);

                // send logoff
                self.online_manager.set_action(
                    SetAction::Closing,
                    None
                );
            }


            _ => {
                // force close all dialogs
                #[cfg(feature="graphics")]
                self.ui_manager.force_close_all(
                    &mut self.values,
                    &mut self.actions
                );

                match &mut self.queued_state {
                    GameState::Ingame(manager) => {
                        // reset the song position
                        if let Some(song) = self
                            .song_manager
                            .instance()
                        {
                            song.pause();
                            if !manager.started {
                                song.set_position(0.0);
                            }
                        }

                        // make sure it has the latest window size and skin
                        #[cfg(feature="graphics")] {
                            manager.window_size_changed(self.values.game.window_size);
                            manager.reload_skin(
                                &mut self.skin_manager,
                                &self.values.settings
                            );
                        }
                        manager.start();

                        let m = manager.metadata.clone();
                        let start_time = manager.start_time;

                        let action;
                        if let Some(manager) = &self
                            .spectator_manager
                        {
                            action = SetAction::Spectating {
                                artist: m.artist.clone(),
                                title: m.title.clone(),
                                version: m.version.clone(),
                                creator: m.creator.clone(),
                                player: manager.host_username.clone(),
                            }
                        } else {
                            action = SetAction::Playing {
                                artist: m.artist.clone(),
                                title: m.title.clone(),
                                version: m.version.clone(),
                                creator: m.creator.clone(),
                                multiplayer_lobby_name: None,
                                start_time
                            };
                        }

                        self.online_manager.set_action(
                            action,
                            Some(m.mode.to_string())
                        );
                        self.actions.push(GameAction::UpdateBackground.into());
                    }
                    #[cfg(feature="graphics")]
                    GameState::SetMenu(_menu) => {
                        self.online_manager.set_action(
                            SetAction::Idle,
                            None
                        );
                    }

                    _ => {}
                }

                let mut do_transition = true;
                #[cfg(feature="graphics")]
                match &self.current_state {
                    GameState::None => do_transition = false,
                    GameState::SetMenu(menu)
                        if menu.name() == "pause" => do_transition = false,
                    _ => {}
                }

                if do_transition {
                    // do a transition
                    let from = Box::new(self.current_state.take());
                    let into = Box::new(self.queued_state.take());

                    self.current_state = GameState::TransitionStarting {
                        into,
                        from,
                        timer: elapsed
                    };
                } else {
                    // old mode was none, or was pause menu, transition to new mode
                    std::mem::swap(
                        &mut self.queued_state,
                        &mut self.current_state
                    );
                }
            }
        }

        // update the notification manager
        #[cfg(feature="graphics")]
        self.notification_manager.update();

        let online_events = self.values
            .values
            .online_manager
            .update(&self.values.values.settings, &mut self.actions);
        for event in online_events {
            match event {
                OnlineEvent::Connected => {
                    self.online_manager.connected = true;
                }

                OnlineEvent::LoggedIn { user_id, username } => {
                    self.values.online_manager.logged_in = true;
                    self.values.online_manager.user_id = user_id;
                    self.values.global.username = username;
                }

                OnlineEvent::Disconnected => {
                    let online = &mut self.values.online_manager;
                    if online.user_id > 0 {
                        online.logged_in = false;
                        online.user_id = 0;
                        // dont nuke username because we used to be logged in
                    }

                    self.task_manager.add_task(Box::new(DelayTask::new(
                        ActionTask::new(ActionTaskAction::Action(
                            GameAction::RestartOnline.into()
                        )),
                        10_000
                    )));
                }
                OnlineEvent::SpectatorEvent(spectator_event) => {

                    match spectator_event {
                        SpectatorEvent::SpectatingHost {
                            host_id,
                            host_username
                        } => {
                            // if let SpectatorWatchAction::FullMenu = self.spec_watch_action {
                            //     // stop spectating everyone else
                            //     for other_host_id in manager.spectator_info.currently_spectating() {
                            //         if other_host_id == host_id { continue }
                            //         OnlineManager::stop_spectating(other_host_id);
                            //     }
                            //     let username = if let Some(u) = manager.users.get(&host_id) {
                            //         u.lock().await.username.clone()
                            //     } else {
                            //         "Host".to_owned()
                            //     };
                            //     self.spectator_manager = Some(Box::new(SpectatorManager::new(host_id, username, self.global.gamemode_infos.clone())));

                            //     // self.queue_state_change(GameState::Spectating(Box::new()));
                            // };

                            self.spectator_manager = Some(Box::new(SpectatorManager::new(
                                host_id,
                                host_username,
                                self.values.global.gamemode_infos.clone()
                            )));
                        }
                        SpectatorEvent::SpectatorJoined {
                            user_id,
                            username
                        } => {
                            if let Some(specman) = self
                                .spectator_manager.as_mut()
                            {
                                specman.spectator_cache.insert(user_id, username.clone().into());
                            }

                            if let Some(manager) = self
                                .current_state.get_ingame()
                            {
                                manager.spectator_info.spectators.add(
                                    SpectatingUser::new(user_id, username)
                                );
                            }
                        }
                        SpectatorEvent::SpectatorLeft { user_id } => {
                            if let Some(specman) = self
                                .spectator_manager.as_mut()
                            {
                                specman.spectator_cache.remove(&user_id);
                            }

                            if let Some(manager) = self
                                .current_state.get_ingame()
                            {
                                manager.spectator_info.spectators.remove(user_id);
                            }
                        }
                        SpectatorEvent::SpectatorFrame {
                            host,
                            frame
                        } => {
                            let frame = *frame;
                            if let Some(specman) = self
                                .spectator_manager.as_mut()
                            {
                                specman.add_frame(frame.clone());
                            }

                            if let Some(manager) = self
                                .current_state.get_ingame()
                                {
                                manager.add_spec_frame(host, frame);
                            }
                        },
                    }
                }
                OnlineEvent::MultiplayerPacket(packet) => {
                    if let Err(e) = self.handle_multiplayer_packet(*packet) {
                        error!("Error handling multiplayer packet: {e:?}");
                    }
                }
            }
        }


        for integration in self.integrations.iter_mut() {
            integration.update(&mut self.values, &mut self.actions);
        }

        // let elapsed = timer.elapsed().as_secs_f32() * 1000.0;
        // if elapsed > 1.0 {warn!("update took a while: {elapsed}");}

        false
    }


    #[cfg(feature="graphics")]
    fn draw(&mut self) {
        // let timer = Instant::now();
        let elapsed = self.game_start.as_millis();

        let mut render_queue = RenderableCollection::default();

        // draw background image
        if let Some(img) = &self.background_image {
            render_queue.push(img.clone());
        }

        // draw dim
        render_queue.push(Rectangle::new(
            Vector2::ZERO,
            self.values.game.window_size,
            Color::BLACK.alpha(self.settings.background_dim),
        ));

        // draw cursor ripples
        self.cursor_manager.draw_ripples(&mut render_queue);

        // draw any gameplay managers
        for (manager, config) in self.gameplay_managers.values_mut() {
            let mut temp_render_queue = RenderableCollection::default();
            if config.draw_function.is_some() {
                std::mem::swap(&mut render_queue, &mut temp_render_queue);
            }

            manager.draw(&mut render_queue);

            if let Some(draw_function) = &config.draw_function {
                std::mem::swap(&mut render_queue, &mut temp_render_queue);

                (draw_function)(temp_render_queue);
            }
        }


        // menu
        self.ui_manager.draw_menu(
            &self.values,
            &mut render_queue,
            &mut self.text_layout_contexts,
        );

        // state
        match &mut self.current_state {
            GameState::Ingame(manager) => {
                manager.draw(&mut render_queue);
            }

            GameState::TransitionStarting {
                into: _,
                from,
                timer
            } => {
                if let Some(game) = from.get_ingame() {
                    game.draw(&mut render_queue);
                }

                // draw fade in rect
                let diff = elapsed - *timer;
                let alpha = diff / (TRANSITION_TIME / 2.0);

                render_queue.push(Rectangle::new(
                    Vector2::ZERO,
                    self.game.window_size,
                    Color::new(0.0, 0.0, 0.0, alpha),
                ));
            }
            GameState::TransitionEnding {
                state,
                timer,
            } => {
                if let Some(game) = state.get_ingame() {
                    game.draw(&mut render_queue);
                }

                let diff = elapsed - *timer;
                let alpha = 1.0 - diff / (TRANSITION_TIME / 2.0);

                render_queue.push(Rectangle::new(
                    Vector2::ZERO,
                    self.game.window_size,
                    Color::new(0.0, 0.0, 0.0, alpha),
                ));
            }

            _ => {}
        }

        // dialogs
        self.ui_manager.draw_dialogs(
            &self.values,
            &mut render_queue,
            &mut self.text_layout_contexts,
        );


        // draw fps's
        self.fps_display.draw(
            &mut render_queue,
            &mut self.text_layout_contexts
        );
        self.update_display.draw(
            &mut render_queue,
            &mut self.text_layout_contexts
        );
        self.render_display.draw(
            &mut render_queue,
            &mut self.text_layout_contexts
        );
        self.input_display.draw(
            &mut render_queue,
            &mut self.text_layout_contexts
        );

        // draw the download manager
        self.download_manager.draw(self.values.game.window_size, &mut render_queue);

        // draw the notification manager
        self.notification_manager.draw(self.values.game.window_size, &mut render_queue);

        // volume control
        self.volume_controller.draw(
            &mut render_queue, 
            &mut self.text_layout_contexts
        );

        // draw cursor
        self.cursor_manager.draw(&mut render_queue);
        
        // toss the items to the window to render
        let _ = self.window_proxy.send_event(WindowAction::RenderData(render_queue.take()));

        self.fps_display.increment();

        // let elapsed = timer.elapsed().as_secs_f32() * 1000.0;
        // if elapsed > 1000.0/144.0 {warn!("render took a while: {elapsed}")}
    }


    #[cfg(feature="graphics")]
    fn handle_inputs(&mut self) -> CurrentInputState {
        let mouse_pos = *self.mouse_position_receiver.read();
        let mouse_moved = mouse_pos != self.input_manager.mouse_pos;

        let mut controller_pause = false;
        let mods = self.input_manager.get_key_mods();
        let window_focus_changed = self.input_manager.get_changed_focus();
        let mut events = self.input_manager.events.take();

        events.retain(|event| {
            match event {
                InputType::MousePress(MouseButton::Left) => {
                    self.cursor_manager.left_pressed(true);

                    // check if a notif was clicked
                    if self.notification_manager.on_click(
                        self.values.game.window_size,
                        mouse_pos,
                        &mut self.actions
                    ) {
                        return false;
                    }
                }
                InputType::MousePress(MouseButton::Right) => {
                    self.cursor_manager.right_pressed(true);
                }
                InputType::MouseRelease(MouseButton::Left) => {
                    self.cursor_manager.left_pressed(false);
                }
                InputType::MouseRelease(MouseButton::Right) => {
                    self.cursor_manager.right_pressed(false);
                }

                InputType::MouseScroll(delta) => {
                    // check for volume change
                    if delta.y != 0.0
                    && let Some(action) = self.volume_controller.on_mouse_wheel(
                        delta.y / (self.settings.display_settings.scroll_sensitivity * 1.5),
                        mods,
                        &mut self.values.settings
                    ) {
                        self.actions.push(action.into());
                        return false;
                    }
                }

                InputType::ControllerPress(GamepadButton::Start, _, _) => {
                    controller_pause = true;
                }

                InputType::KeyPress(key) => {
                    let Some(key) = key.as_key()
                    else { return true };

                    if self.volume_controller.on_key_press(
                        &key,
                        mods,
                        &mut self.actions,
                        &mut self.values.settings
                    ) {
                        return false;
                    }

                    // check user panel
                    if key == self.settings.key_user_panel {
                        self.handle_make_userpanel();
                        return false;
                    }

                    match key {
                        // screenshot
                        Key::F12 => self.window_proxy.send_event(WindowAction::TakeScreenshot(ScreenshotInfo {
                            // if shift is pressed, upload to server, and get link
                            upload: mods.shift,
                        })).unwrap(),

                        // settings menu
                        Key::O if mods.ctrl => {
                            let is_ingame = self.current_state.is_ingame();
                            let allow_ingame = self.settings
                                .common_game_settings
                                .allow_ingame_settings;

                            if !is_ingame || allow_ingame {
                                self.handle_custom_dialog(
                                    "settings",
                                    DialogCreateOptions::default(),
                                );
                            }
                        }

                        // debug
                        Key::PageUp if mods.ctrl => {
                            debug!("{:#?}", self.values.values);
                        }

                        // custom menu list
                        Key::M if mods.ctrl && mods.shift => {
                            self.actions.push(MultiplayerAction::CreateLobby {
                                name: "a".to_string(),
                                password: String::new(),
                                private: false,
                                players: 5
                            }.into());

                            // self.actions.push(MenuAction::set_menu("menu_list"));
                        }

                        // debug
                        Key::T if mods.ctrl && mods.shift => {
                            self.ui_manager.root_tree.print(&self.values);
                        }

                        // console dialog
                        Key::Grave if !self.current_state.is_ingame() => {
                            // self.handle_custom_dialog(
                            //     "console_dialog",
                            //     DialogCreateOptions::default(),
                            //     BuildableInputArguments::default()
                            // );

                            // self.ui_manager.add_dialog(
                            //     ConsoleDialog::new().boxed(),
                            //     ConsoleDialog::DEFAULT_OPTIONS,
                            //     &mut self.values,
                            //     &mut self.actions,
                            // );
                        }


                        // close latest dialog
                        Key::Escape if self.ui_manager.close_latest(
                            &mut self.values,
                            &mut self.actions
                        ) => {}

                        // full refresh
                        Key::F5 if mods.ctrl => {
                            self.actions.push(Notification::new_text(
                                "Doing a full refresh, the game will freeze for a bit",
                                Color::RED,
                                5000.0
                            ).into());
                            self.values.values.beatmap_manager.full_refresh(
                                &self.values.values.settings
                            );
                        }

                        // reload custom menus
                        Key::R if mods.ctrl => {
                            debug!("Reloading custom menus/dialogs");
                            self.load_custom_menus();

                            debug!("Reloading current menu");
                            self.handle_custom_menu(
                                self.ui_manager.get_menu().clone(),
                            );
                        }


                        // playmode change keybind
                        // FIXME: move to menus??

                        Key::Key1
                        | Key::Key2
                        | Key::Key3
                        | Key::Key4
                        if mods.ctrl => {
                            let index = match key {
                                Key::Key1 => 0,
                                Key::Key2 => 1,
                                Key::Key3 => 2,
                                Key::Key4 => 3,
                                _ => unsafe { std::hint::unreachable_unchecked() },
                            };

                            let Some(mode) = self.global
                                .gamemode_infos
                                .by_num
                                .get(index)
                            else { return true };

                            let mode = mode.id;
                            self.actions.push(BeatmapAction::SetPlaymode(mode.to_string()).into());
                            self.actions.push(Notification::new_text(
                                format!("Playmode set to {mode}"),
                                Color::CYAN,
                                3000.0
                            ).into());
                        }

                        _ => return true,
                    }

                    return false;
                }

                _ => {}
            }

            true
        });

        if mouse_moved {
            events.push(InputType::MouseMove(mouse_pos));
            self.input_manager.mouse_pos = mouse_pos;
            self.volume_controller.on_mouse_move(mouse_pos);
        }

        CurrentInputState {
            mouse_pos,
            mouse_moved,
            window_focus_changed,
            controller_pause,
            mods,
            events,
        }
    }


    pub(super) fn handle_action(
        &mut self,
        action: impl Into<TatakuAction> + 'static
    ) {
        let action = action.into();
        // debug!("handling action: {action:?}");

        match action {
            TatakuAction::None => {},
            TatakuAction::Delayed(action, delay) => {
                self.task_manager.add_task(Box::new(DelayTask::new(
                    ActionTask::new(action),
                    delay
                )));
            }


            #[cfg(feature="gameplay")]
            TatakuAction::Online(action)
                => self.online_manager.handle_action(action),

            #[cfg(feature="graphics")]
            TatakuAction::Menu(action)
                => self.handle_menu_action(action),

            TatakuAction::Audio(action)
                => self.audio_manager.handle_action(
                    action,
                    &mut self.values,
                    #[cfg(feature="graphics")]
                    &mut self.skin_manager,
                ),
            TatakuAction::Beatmap(action)
                => self.handle_beatmap_action(action),
            TatakuAction::Game(action)
                => self.handle_game_action(*action),
            TatakuAction::Multiplayer(action)
                => self.handle_multiplayer_action(action),
            TatakuAction::Song(action)
                => self.handle_song_action(action),
            TatakuAction::Mods(action)
                => self.handle_mod_action(action),
            TatakuAction::Event(e)
                => self.handle_event(*e),
            TatakuAction::Download(dl)
                => self.download_manager.add_download(*dl),

            TatakuAction::OnlineContent(action)
                => self.online_content_manager.handle_action(
                    action,
                    &mut self.actions,
                    &mut self.values
                ),

            // task actions
            TatakuAction::Task(TaskAction::AddTask(task)) => {
                if !self.values.settings.enable_diffcalc
                    && task.get_id() == Cow::Borrowed("diff_calc")
                {
                    return
                }

                self.task_manager.add_task(task);
            },

            #[cfg(feature="graphics")]
            TatakuAction::CursorAction(action)
                => self.cursor_manager.handle_cursor_action(
                    action, 
                    &mut self.text_layout_contexts
                ),

            #[cfg(feature="graphics")]
            TatakuAction::WindowAction(action)
                => self.window_proxy.send_event(*action).nope(),

            #[cfg(feature="graphics")]
            TatakuAction::Ui(action) => self.ui_manager.handle_ui_action(
                action,
                &mut self.values,
                &mut self.actions,
            ),

            TatakuAction::Multiple(list) => {
                for i in list {
                    self.handle_action(i);
                }
            }

            #[cfg(not(feature="graphics"))]
            _ => {}
        }
    }

    #[cfg(feature="gameplay")]
    pub(super) fn queue_state_change(&mut self, state: GameState) {
        match state {
            #[cfg(feature="graphics")]
            GameState::SetMenu(menu) => {
                debug!("Changing menu to: {}", menu.name());
                self.queued_state = GameState::InMenu;
                self.ui_manager.set_root(
                    menu,
                    &mut self.values,
                    &mut self.actions,
                    &mut self.text_layout_contexts,
                );
                self.queued_events.push((TatakuEvent::MenuEnter, None));
                self.ui_manager.reload_skin(
                    &mut self.values,
                    &mut self.actions,
                    &mut self.skin_manager,
                    &mut self.text_layout_contexts,
                );
            }
            GameState::InMenu => {}
            mut state => {
                if let Some(game) = state.get_ingame() {
                    let meta = game.beatmap.get_beatmap_meta();
                    debug!(
                        "Starting/resuming game: {} ({})",
                        meta.version_string(),
                        meta.beatmap_hash
                    );
                }

                // set the menu to an empty element, hiding it
                #[cfg(feature="graphics")]
                self.ui_manager.set_root(
                    EmptyWidget::new_boxed(),
                    &mut self.values,
                    &mut self.actions,
                    &mut self.text_layout_contexts,
                );
                self.queued_state = state;
            }
        }
    }


    #[cfg(feature="graphics")]
    fn handle_make_userpanel(&mut self) {
        let mut user_panel_exists = false;
        let mut chat_exists = false;

        for i in self.ui_manager.dialogs.iter() {
            if i.get_node().name() == "user_panel" {
                user_panel_exists = true;
            }
            if i.get_node().name() == "chat" {
                chat_exists = true;
            }
            // if both exist, no need to continue looping
            if user_panel_exists && chat_exists { break }
        }

        if !user_panel_exists {
            // close existing chat window
            if chat_exists {
                self.ui_manager.dialogs.retain(
                    |d| d.get_node().name() != "chat"
                );
            }
            self.handle_custom_dialog(
                "user-panel",
                DialogCreateOptions::default(),
            );
        } else {
            self.ui_manager.dialogs.retain(
                |d| d.get_node().name() != "user_panel"
            );
        }
    }

    pub(super) fn handle_event(&mut self, event: TatakuIntegrationEvent) {
        for i in self.integrations.iter_mut() {
            i.handle_event(&event, &self.values, &mut self.actions);
        }
    }

    #[cfg(feature="graphics")]
    pub(super) fn resize_bg(&mut self) {
        let Some(bg) = &mut self.background_image
        else { return };

        bg.fit_to_bg_size(self.values.game.window_size);
    }

    /// Drag and Drop
    #[cfg(feature="graphics")]
    pub(super) fn handle_file_drop(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        info!("File dropped: {path:?}");

        if let Some(ext) = path.extension() {
            let ext = ext.to_str().unwrap();
            match ext {
                // osu | quaver | ptyping zipped set file
                "osz" | "qp" | "ptm" => {
                    match Zip::extract_single(
                        path,
                        SONGS_DIR,
                        true,
                        ArchiveDelete::Always
                    ) {
                        Err(e) => self.actions.push(
                            Notification::new_error(
                                "Error extracting file",
                                e
                            ).into()
                        ),

                        Ok(path) => {
                            // load the map
                            let Some(last) = self
                                .beatmap_manager
                                .check_folder(
                                path,
                                HandleDatabase::YesAndReturnNewMaps,
                            ).and_then(|l| l.last().cloned())
                            else {
                                warn!("didnt get any beatmaps from beatmap file drop");
                                return;
                            };

                            // set it as current map if wanted
                            let mut use_preview_time = true;
                            let change_map = match &self.current_state {
                                GameState::SetMenu(menu) => {
                                    if menu.name() == "main_menu" {
                                        use_preview_time = false;
                                    }
                                    true
                                }
                                _ => false,
                            };

                            if change_map {
                                self.actions.push(BeatmapAction::Set(
                                    last.beatmap_hash,
                                    SetBeatmapOptions::default()
                                        .use_preview_point(use_preview_time)
                                        .restart_song(false)
                                ).into());
                            }
                        }
                    }
                }

                // osu skin file
                "osk" => {
                    match Zip::extract_single(
                        path,
                        SKINS_FOLDER,
                        true,
                        ArchiveDelete::Never
                    ) {
                        Err(e) => self.actions.push(
                            Notification::new_error(
                                "Error extracting file",
                                e
                            ).into()
                        ),
                        Ok(path) => {
                            // set as current skin
                            if let Some(folder) = Path::new(&path).file_name() {
                                let name = folder.to_string_lossy().to_string();
                                self.values.settings.current_skin = name.clone();
                                self.actions.push(Notification::new_text(
                                    format!("Added skin {name}"),
                                    Color::BLUE,
                                    5000.0
                                ).into());
                            }
                        }
                    }
                }

                // tataku | osu replay
                "ttkr" | "osr" => {
                    match Self::read_replay_path(path, &self.global.gamemode_infos) {
                        Ok(score) => self.try_open_replay(score),
                        Err(e) => self.actions.push(
                            Notification::new_error(
                                "Error opening replay",
                                e
                            ).into()
                        ),
                    }
                }

                _ => {
                    self.actions.push(Notification::default()
                        .text("What is this?")
                        .color(Color::RED)
                        .duration(3_000.0)
                        .into()
                    );
                }
            }
        }
    }

    #[cfg(feature="graphics")]
    pub(super) fn try_open_replay(&mut self, score: Score) {
        if !self.beatmap_manager.has_hash(&score.beatmap_hash) {
            self.actions.push(
                Notification::default()
                .text("You don't have this beatmap!")
                .duration(5_000.0)
                .color(Color::RED)
                .into()
            );

            return;
        };

        let config = self.create_select_beatmap_config(
            true,
            true
        );

        self.set_current_beatmap(score.beatmap_hash, config);

        // move to a score menu with this as the score
        // let score = IngameScore::new(score, false, false);
        // let menu = ScoreMenu::new(&score, map, false, self.global.gamemode_infos.clone());
        // self.queued_state = GameState::SetMenu(Box::new(menu));

        let info = self.values.global
            .gamemode_infos
            .get_info(&score.playmode)
            .copied()
            .unwrap_or_default();

        self.values.values.score = ReflectScore::new(
            &IngameScore::new(score, false, false),
            &info
        );

        // show score menu
        self.values.impl_insert("var.score_menu.allow_retry".into(), Box::new(false)).unwrap();

        self.handle_custom_menu("score_menu");

    }


    #[cfg(feature="graphics")]
    pub(super) fn ingame_complete(&mut self, mut manager: Box<GameplayManager>) {
        trace!("beatmap complete");
        manager.on_complete();
        manager.score.time = chrono::Utc::now().timestamp() as u64;
        self.actions.push(TatakuIntegrationEvent::BeatmapEnded.into());
        self.actions.push(CursorAction::SetVisible(true).into());

        if manager.failed {
            trace!("player failed");
            if !manager.get_mode().is_multi() {
                self.pending_gameplay_manager = Some(manager);
                self.actions.push(MenuAction::SetMenu {
                    id: "fail_menu".into(),
                }.into());
                // self.queue_state_change(GameState::SetMenu(Box::new(PauseMenu::new(true))));
                return;
            }
        } else {
            let mut score = manager.score.clone();
            let info = self.global
                .gamemode_infos
                .get_info(&score.playmode)
                .unwrap();
            score.accuracy = info.calc_acc(&score);

            self.values.score = ReflectScore::new(&score, info);

            let mut score_submit = None;
            if manager.should_save_score() {
                // save score
                Database::save_score(&score);
                match save_replay(&score) {
                    Ok(_) => trace!("replay saved ok"),
                    Err(e) => self.actions.push(
                        Notification::new_error(
                            "error saving replay",
                            e
                        ).into()
                    ),
                }

                let Some(map) = self.beatmap_manager
                    .get_by_hash(&score.beatmap_hash)
                else { return warn!("no map ???") };

                // submit score
                let submit_task = UploadScoreTask::new(
                    (*score).clone(),
                    &map,
                    &self.settings
                );
                score_submit = Some(submit_task.get_path().to_owned());
                self.actions.push(TaskAction::AddTask(Box::new(submit_task)).into());
            }

            match manager.get_mode() {
                // go back to beatmap select
                GameplayModeInner::Replaying {..} => {
                    self.handle_custom_menu("beatmap_select");
                }
                GameplayModeInner::Multiplayer { .. } => {
                    debug!("multiplayer finished gameplay");

                    // FIXME: show the scores lmao
                    // go back to the lobby menu
                    self.handle_custom_menu("lobby_menu");
                }

                _ => {
                    // show score menu
                    self.values.impl_insert("var.score_menu.allow_retry".into(), Box::new(true)).unwrap();
                    self.values.impl_insert("var.score_menu.score_path".into(), Box::new(score_submit.unwrap_or_default())).unwrap();

                    self.handle_custom_menu("score_menu");

                    // let mut menu = ScoreMenu::new(&score, manager.metadata.clone(), true, self.global.gamemode_infos.clone());
                    // menu.score_submit = score_submit;
                    // self.queue_state_change(GameState::SetMenu(Box::new(menu)));
                }
            }
        }

        manager.cleanup_textures(&mut self.skin_manager);
    }

    #[cfg(feature="graphics")]
    pub(super) fn load_theme(&mut self) {
        let theme = match &self.settings.theme {
            SelectedTheme::Tataku => tataku_theme(),
            SelectedTheme::Osu => osu_theme(),
            SelectedTheme::Custom(path, _) => Io::read_file(path)
                .ok()
                .and_then(|b| serde_json::from_slice(&b).ok())
                .unwrap_or_default(),
        };

        self.values.theme = theme;
    }


    #[cfg(feature="graphics")]
    fn finish_screenshot(
        &mut self,
        bytes: Vec<u8>,
        [width, height]: [u32; 2],
        info: ScreenshotInfo
    ) -> TatakuResult {
        // create file
        let date = chrono::Local::now();
        let year = date.year();
        let month = date.month();
        let day = date.day();
        let hour = date.hour();
        let minute = date.minute();
        let second = date.second();

        let file = format!(
            "../Screenshots/{year}-{month}-{day}--{hour}-{minute}-{second}.png"
        );
        let path = Path::new(&file);

        std::fs::create_dir_all(path.parent().unwrap())?;

        // save as png
        image::save_buffer(
            path,
            &bytes,
            width,
            height,
            image::ExtendedColorType::Rgba8
        )?;

        // notify user
        let full_path = std::env::current_dir()
            .unwrap()
            .join(path)
            .to_string_lossy()
            .to_string();

        self.actions.push(GameAction::AddNotification(Notification::new(
            format!("Screenshot saved to {full_path}"),
            Color::BLUE,
            5000.0,
            NotificationOnClick::File(full_path.clone())
        )).into());

        if info.upload {
            self.task_manager.add_task(Box::new(
                UploadScreenshotTask::new(full_path))
            );
        }

        Ok(())
    }


    pub(super) fn update_playmode(&mut self, playmode: &str) {
        // ensure lowercase
        let playmode: ArcStr = playmode.to_lowercase().into();

        // ensure playmode exists
        let Ok(info) = self.global.gamemode_infos
            .get_info(&playmode)
            .cloned()
        else {
            return warn!("Trying to set invalid playmode: {playmode}")
        };

        // set playmode and playmode display
        self.values.global.update_playmode(playmode.clone());
        self.values.settings.last_played_mode = playmode.to_string();

        // determine the actual playmode

        // if we have a beatmap, get the override mode and update the playmode_actual values
        let actual_playmode = self.beatmap_manager
            .current_beatmap()
            .filter(|b| !info.can_load_beatmap(&b.beatmap_type))
            .map(|b| b.mode.clone())
            .unwrap_or(playmode)
            ;

        self.values.global.update_playmode_actual(actual_playmode);

        // TODO: update mods list as well?
    }

    pub(super) fn create_select_beatmap_config(
        &self,
        restart_song: bool,
        use_preview_time: bool,
    ) -> SelectBeatmapConfig {
        SelectBeatmapConfig::new(
            self.global.mods.clone(),
            self.global.playmode.clone(),
            restart_song,
            use_preview_time,
        )
    }

    pub fn read_replay_path(
        path: impl AsRef<Path>,
        infos: &GamemodeInfos,
    ) -> TatakuResult<Score> {
        let path = path.as_ref();

        match path.extension().and_then(|s| s.to_str()) {

            // tataku replay
            Some("ttkr") => {
                let replay = std::fs::read(path)?;
                Ok(
                    Replay::try_read_replay(&mut SerializationReader::new(replay))
                    .map_err(|e| TatakuError::String(format!("{e:?}")))?
                )
            },

            // osu replay
            Some("osr") => Ok(convert_osu_replay(path, infos)?),

            _ => Err(TatakuError::String("Unknown replay file".to_owned()))
        }
    }
}



impl Deref for Game {
    type Target = TatakuValues;
    fn deref(&self) -> &Self::Target {
        &self.values.values
    }
}
impl DerefMut for Game {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values.values
    }
}

#[derive(Default)]
pub(super) enum GameState {
    #[default] None,
    TransitionStarting {
        into: Box<Self>,
        from: Box<Self>,
        timer: f32,
    },
    TransitionEnding {
        state: Box<Self>,
        timer: f32,
    },

    Closing,
    Ingame(Box<GameplayManager>),
    #[cfg(feature="graphics")]
    /// need to transition to the provided menu
    SetMenu(Box<dyn Widget<TatakuAction>>),

    /// Currently in a menu (this doesnt actually work currently, but it doesnt really matter)
    InMenu,
}
impl GameState {
    pub fn is_ingame(&self) -> bool {
        matches!(self, Self::Ingame(_))
    }
    pub fn get_ingame(&mut self) -> Option<&mut Box<GameplayManager>> {
        match self {
            GameState::Ingame(manager) => Some(manager),
            _ => None
        }
    }
}

#[allow(unused)]
pub enum SpectatorWatchAction {
    FullMenu,
    OpenDialog,
    MultiSpec,
}
