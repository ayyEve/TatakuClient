use crate::prelude::*;
use chrono::{ Datelike, Timelike };

/// how long transitions between states should last
const TRANSITION_TIME:f32 = 500.0;

#[cfg(feature="dynamic_gamemodes")]
pub type IncomingGamemode = GamemodeLibrary;
#[cfg(not(feature="dynamic_gamemodes"))]
pub type IncomingGamemode = GamemodeInfo;

pub struct BuiltinMenus {
    pub menus: &'static [(&'static str, &'static [u8])],
    pub dialogs: &'static [(&'static str, &'static [u8])],
}

pub struct Game {
    // engine things
    pub actions: ActionQueue,
    runtime: Rc<tokio::runtime::Runtime>,
    
    #[cfg(feature="graphics")] 
    volume_controller: VolumeControl,
    current_state: GameState,
    queued_state: GameState,
    #[cfg(feature="graphics")]
    window_event_receiver: tokio::sync::mpsc::Receiver<WindowEvent>,

    #[cfg(feature="graphics")]
    window_proxy: winit::event_loop::EventLoopProxy<WindowAction>,

    // managers
    score_manager: ScoreManager,
    pub song_manager: SongManager,
    pub audio_manager: AudioManager,
    pub(crate) task_manager: TaskManager,
    difficulty_manager: DifficultyManager,
    online_content_manager: OnlineContentManager,

    #[cfg(feature="graphics")] ui_manager: UiManager,
    #[cfg(feature="graphics")] skin_manager: SkinManager,
    #[cfg(feature="gameplay")] input_manager: InputManager,
    #[cfg(feature="graphics")] cursor_manager: CursorManager,
    #[cfg(feature="graphics")] custom_menu_manager: CustomMenuManager,
    #[cfg(feature="graphics")] xml_test_manager: Option<XmlTestManager>,
    #[cfg(feature="graphics")] notification_manager: NotificationManager,
    #[cfg(feature="gameplay")] spectator_manager: Option<Box<SpectatorManager>>,
    #[cfg(feature="gameplay")] multiplayer_manager: Option<Box<MultiplayerManager>>,

    #[cfg(feature="graphics")] 
    gameplay_managers: HashMap<GameplayId, (GameplayManager, NewManager)>,
    pending_gameplay_manager: Option<Box<GameplayManager>>,


    integrations: Vec<Box<dyn TatakuIntegration>>,

    // fps
    #[cfg(feature="graphics")] fps_display: FpsDisplay,
    #[cfg(feature="graphics")] update_display: FpsDisplay,
    #[cfg(feature="graphics")] render_display: AsyncFpsDisplay,
    #[cfg(feature="graphics")] input_display: AsyncFpsDisplay,

    // misc
    game_start: TatakuInstant,

    #[cfg(feature="graphics")] builtin_menus: BuiltinMenus,
    #[cfg(feature="graphics")] background_image: Option<Image>,
    #[cfg(feature="graphics")] wallpapers: Vec<Image>,
    #[cfg(feature="graphics")] background_loader: Option<AsyncLoader<Option<Image>>>,
    // spec_watch_action: SpectatorWatchAction,

    #[cfg(feature="graphics")]
    pub queued_events: Vec<(TatakuEventType, Option<TatakuValue>)>,

    pub values: ValueCollection,
}
impl Game {
    pub fn new(
        #[cfg(feature="graphics")]
        game_event_receiver: tokio::sync::mpsc::Receiver<WindowEvent>,
        #[cfg(feature="graphics")]
        window_proxy: winit::event_loop::EventLoopProxy<WindowAction>,
        
        audio_engines: Vec<AudioApiInit>,
        gamemodes: Vec<IncomingGamemode>,

        #[cfg(feature="graphics")]
        builtin_menus: BuiltinMenus,
    ) -> Self {
        let settings = Settings::load();
        let infos = GamemodeInfos::new(gamemodes); 
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
            #[cfg(feature="gameplay")] input_manager: InputManager::new(),
            #[cfg(feature="graphics")] volume_controller: VolumeControl::new(),
            #[cfg(feature="graphics")] background_image: None,
            #[cfg(feature="graphics")] wallpapers: Vec::new(),
            #[cfg(feature="graphics")] builtin_menus,

            spectator_manager: None,
            multiplayer_manager: None,
            difficulty_manager: DifficultyManager,
            online_content_manager,

            song_manager: SongManager::new(),
            audio_manager: AudioManager::init_audio(audio_engines)
                .expect("failed to initialize audio engine!"),
            score_manager: ScoreManager::new(infos.clone()),
            task_manager: TaskManager::new(),

            #[cfg(feature="graphics")]
            custom_menu_manager: CustomMenuManager::default(),

            #[cfg(feature="graphics")]
            cursor_manager: CursorManager::new(
                skin_manager.skin().clone(), 
                settings.cursor_settings.clone()
            ),
            #[cfg(feature="graphics")] skin_manager,
            #[cfg(feature="graphics")] notification_manager: NotificationManager::default(),
            #[cfg(feature="graphics")] ui_manager: UiManager::new(),
            #[cfg(feature="graphics")] xml_test_manager: None,
            #[cfg(feature="graphics")] gameplay_managers: HashMap::new(),
            pending_gameplay_manager: None,

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
            #[cfg(feature="graphics")] window_event_receiver: game_event_receiver,

            #[cfg(feature="graphics")] queued_events: Vec::new(),

            values: ValueCollection {
                values: TatakuValues::new(&infos, online_content_engines, &settings),
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
        ).is_err() {
            self.ui_manager.set_root(
                EmptyWidget::new_boxed(), 
                &mut self.values, 
                &mut self.actions
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

        // macro to help
        macro_rules! load_entry {
            (menu, $self:ident, $bytes: expr, $name:expr) => {{
                let result = $self.custom_menu_manager.load_entry_bytes(
                    $bytes,
                    None,
                    CustomMenuSource::Game,
                    CustomEntryType::Menu
                );

                if let Err(e) = result {
                    error!("Error loading custom menu {}: {e}", $name);
                }
            }};

            (dialog, $self:ident, $bytes: expr, $name: expr) => {{
                let result = $self.custom_menu_manager.load_entry_bytes(
                    $bytes,
                    None,
                    CustomMenuSource::Game,
                    CustomEntryType::Dialog
                );

                if let Err(e) = result {
                    error!("Error loading custom dialog {}: {e}", $name);
                }
            }};
        }

        // menus
        for (name, data) in self.builtin_menus.menus {
            load_entry!(menu, self, data, name);
        }
        {
            // use tataku_resources::menus::*;
            // load_entry!(menu, self, MAIN_MENU);
            // load_entry!(menu, self, BEATMAP_SELECT);
            // load_entry!(menu, self, MENU_LIST);
            // load_entry!(menu, self, LOBBY_SELECT);
            // load_entry!(menu, self, LOBBY_MENU);
            // load_entry!(menu, self, PAUSE_MENU);
            // load_entry!(menu, self, FAIL_MENU);
            // load_entry!(menu, self, SCORE_MENU);
            // load_entry!(menu, self, LOADING_MENU);
            // load_entry!(menu, self, ONLINE_CONTENT);
        }
        // dialogs 
        for (name, data) in self.builtin_menus.dialogs {
            load_entry!(dialog, self, data, name);
        }
        {
            // use tataku_resources::dialogs::*;
            // load_entry!(dialog, self, BEATMAP_CONTEXT_MENU);
            // load_entry!(dialog, self, CHAT);
            // load_entry!(dialog, self, CREATE_LOBBY);
            // load_entry!(dialog, self, JOIN_LOBBY);
            // load_entry!(dialog, self, MODS);
            // load_entry!(dialog, self, USER_PANEL);
            // load_entry!(dialog, self, "../dialogs/console.xml", CONSOLE);
        }

        self.custom_menu_manager.update_values(&mut self.values);
        debug!("Done loading custom menus");
    }

    fn init_online(&mut self) {
        self.values.values.online_manager.start(
            &self.values.values.settings, 
            &self.runtime
        );
    }

    fn init(&mut self) {
        let now = std::time::Instant::now();
        
        #[cfg(feature="graphics")]
        self.load_custom_menus();

        #[cfg(feature="graphics")] 
        self.load_theme();

        self.init_online();

        // setup double tap protection
        #[cfg(feature="gameplay")]
        self.input_manager.set_double_tap_protection(
            self.settings.enable_double_tap_protection
                .then_some(self.settings.double_tap_protection_duration)
        );

        // new beatmap check task
        self.actions.push(TaskAction::AddTask(Box::new(
            BeatmapDownloadsCheckTask::default()
        )));

        let mut settings = self.settings.clone();
        settings.gamemode_settings.build(self.values.global.gamemode_infos.clone());
        settings.init(&mut self.values, "settings".to_string());
        self.settings = settings;

        debug!("game init took {:.2}", now.elapsed().as_secs_f32() * 1000.0);




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

        self.actions.push(InitGameTask::default());
        #[cfg(feature="graphics")]
        self.handle_custom_menu("loading_menu", None);
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
            if let Some(last_update) = last_setting_update {
                if last_update.as_millis() > 500.0 {
                    self.settings.clone().save();
                    last_setting_update = None;
                }
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
        #[cfg(feature="graphics")] 
        for (i, _) in self.gameplay_managers.values_mut() {
            i.cleanup_textures(&mut self.skin_manager);
        }
        #[cfg(feature="graphics")] 
        if let Some(i) = self.current_state.get_ingame() {
            i.cleanup_textures(&mut self.skin_manager);
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
                    ));
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

                _ => {}
            }
        }
        // since window resizes can spam and laying out the ui can be slow, 
        // sometimes they happen too fast for us to keep up with.
        // so this should make sure things arent delayed because of the spam
        #[cfg(feature="graphics")] 
        if self.values.game.window_size != window_size {
            self.resize_bg();
            self.ui_manager.window_size_changed(window_size, &self.values);
            self.values.game.window_size = window_size;

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
        if let Some(loader) = self.background_loader.clone() {
            if let Some(image) = loader.check() {
                self.background_loader = None;

                // unload the old image so the atlas can reuse the space
                if let Some(old_img) = self.background_image.take() {
                    GameWindow::free_texture(*old_img.tex);
                }

                self.background_image = image;

                if self.background_image.is_none() && !self.wallpapers.is_empty() {
                    self.background_image = Some(self.wallpapers[0].clone());
                }

                self.resize_bg();
            }
        }

        #[cfg(feature="graphics")] self.update_display.increment();

        // update counters
        #[cfg(feature="graphics")] self.fps_display.update();
        #[cfg(feature="graphics")] self.update_display.update();
        #[cfg(feature="graphics")] self.render_display.update();
        #[cfg(feature="graphics")] self.input_display.update();

        // read input events
        let mouse_pos = self.input_manager.mouse_pos;
        let mut mouse_down = self.input_manager.get_mouse_down();
        let mouse_up = self.input_manager.get_mouse_up();
        let mouse_moved = self.input_manager.get_mouse_moved();
        // TODO: do we want this here or only in menus?
        let mut scroll_delta = self.input_manager.get_scroll_delta() 
            * self.settings.display_settings.scroll_sensitivity;

        let mut keys_down = self.input_manager.get_keys_down();
        let keys_up = self.input_manager.get_keys_up();
        let mods = self.input_manager.get_key_mods();
        let window_focus_changed = self.input_manager.get_changed_focus();

        let controller_down = self
            .input_manager
            .get_controller_down();
        let controller_up = self
            .input_manager
            .get_controller_up();
        let controller_axis = self
            .input_manager
            .get_controller_axis();

        // update the cursor
        #[cfg(feature="graphics")] 
        self.cursor_manager.update(
            elapsed, 
            self.input_manager.mouse_pos
        );

        // update cursor
        #[cfg(feature="graphics")] 
        if mouse_down.contains(&MouseButton::Left) {
            self.cursor_manager.left_pressed(true);
        } else if mouse_up.contains(&MouseButton::Left) {
            self.cursor_manager.left_pressed(false);
        }
        #[cfg(feature="graphics")] 
        if mouse_down.contains(&MouseButton::Right) {
            self.cursor_manager.right_pressed(true);
        } else if mouse_up.contains(&MouseButton::Right) {
            self.cursor_manager.right_pressed(false);
        }

        let controller_pause = controller_down
            .iter()
            .any(|(_, a)| 
                a.contains(&ControllerButton::Start)
            );

        // check if a notif was clicked
        #[cfg(feature="graphics")] 
        if !mouse_down.is_empty() && self.notification_manager.on_click(
            self.values.game.window_size, 
            mouse_pos, 
            &mut self.actions
        ) {
            mouse_down.clear();
        }

        // check for volume change
        #[cfg(feature="graphics")] 
        if mouse_moved { self.volume_controller.on_mouse_move(mouse_pos); }
        #[cfg(feature="graphics")] 
        if scroll_delta != 0.0 {
            if let Some(action) = self.volume_controller.on_mouse_wheel(
                scroll_delta / (self.settings.display_settings.scroll_sensitivity * 1.5), 
                mods, 
                &mut self.values.settings
            ) {
                scroll_delta = 0.0;
                self.actions.push(action);
            }
        }
        #[cfg(feature="graphics")] 
        self.volume_controller.on_key_press(
            &mut keys_down, 
            mods, 
            &mut self.values.settings
        );

        self.check_keys(&mut keys_down, &mods);


        // update our global values
        {
            let values = &mut self.values;
            values.song.position = self.song_manager.position();

            if let Some(audio) = self.song_manager.instance() {
                if self.values.song.set_state(audio.get_state()) {
                    let action = match self.values.song.state {
                        AudioState::Stopped 
                        | AudioState::Unknown => TatakuEventType::SongEnd,
                        AudioState::Playing => TatakuEventType::SongStart,
                        AudioState::Paused => TatakuEventType::SongPause,
                    };
                    self.actions.push(action);
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
        #[cfg(feature="graphics")] 
        let mut input_state = CurrentInputState {
            mouse_pos,
            mouse_moved,
            scroll_delta,
            mouse_down,
            mouse_up,
            keys_down,
            keys_up,
            mods,

            controller_axes: controller_axis
                .into_iter()
                .flat_map(|(info, axes)| 
                    axes
                    .clone()
                    .into_iter()
                    .filter_map(move |(axis, state)| 
                        state.changed.then_some((
                            axis, 
                            state.value, 
                            info.id, 
                            info.name.clone()
                        ))
                    )
                )
                .collect(),

            controller_down: controller_down
                .into_iter()
                .flat_map(|(info, buttons)| 
                    buttons
                    .into_iter()
                    .map(move |b| (b, info.id, info.name.clone()))
                )
                .collect(),
            
            controller_up: controller_up
                .into_iter()
                .flat_map(|(info, buttons)| 
                    buttons
                    .into_iter()
                    .map(move |b| (b, info.id, info.name.clone()))
                )
                .collect(),
        };
        #[cfg(feature="graphics")] 
        self.ui_manager.update(
            &mut input_state,
            self.queued_events.take(),
            &mut self.values,
            &mut self.actions,
            &mut self.skin_manager,
        );

        #[cfg(feature="graphics")] 
        if let Some(tester) = &mut self.xml_test_manager {
            tester.update(
                &mut self.ui_manager,
                &mut self.values,
                &mut self.actions,
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
                if let Some(got_focus) = window_focus_changed {
                    if self.settings.display_settings.pause_on_focus_lost {
                        manager.window_focus_changed(got_focus);
                    }
                }

                if !manager.failed && manager.can_pause() 
                    && (manager.should_pause || controller_pause)
                {
                    manager.pause();
                    let actions = manager.actions.take();
                    self.handle_actions(Some(actions));

                    self.pending_gameplay_manager = Some(manager);
                    #[cfg(feature="graphics")] 
                    self.actions.push(MenuAction::SetMenu { 
                        id: "pause_menu".into(), 
                        input: Box::new(BuildableInputArguments::default()),
                    });
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
                            let name = menu.name().into_owned();
                            self.ui_manager.set_root(
                                menu, 
                                &mut self.values, 
                                &mut self.actions
                            );
                            self.ui_manager.reload_skin(
                                &mut self.values, 
                                &mut self.actions, 
                                &mut self.skin_manager,
                            );
                            
                            let elapsed = self.game_start.as_millis();
                            self.current_state = GameState::TransitionEnding { 
                                state: Box::new(GameState::InMenu(name)), 
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

                        // make sure it has the latest window size
                        #[cfg(feature="graphics")] 
                        manager.window_size_changed(self.values.game.window_size);
                        #[cfg(feature="graphics")] 
                        manager.reload_skin(
                            &mut self.skin_manager, 
                            &self.values.settings
                        );
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
                        self.actions.push(GameAction::UpdateBackground);
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
                match &self.current_state {
                    GameState::None => do_transition = false,
                    #[cfg(feature="graphics")] 
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

        let mut render_queue = RenderableCollection::new();

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
            let mut temp_render_queue = RenderableCollection::new();
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
        self.ui_manager.draw_menu(&self.values, &mut render_queue);

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
        self.ui_manager.draw_dialogs(&self.values, &mut render_queue);
        


        // draw fps's
        self.fps_display.draw(&mut render_queue);
        self.update_display.draw(&mut render_queue);
        self.render_display.draw(&mut render_queue);
        self.input_display.draw(&mut render_queue);

        // draw the download manager
        self.download_manager.draw(self.values.game.window_size, &mut render_queue);

        // draw the notification manager
        self.notification_manager.draw(self.values.game.window_size, &mut render_queue);

        // volume control
        self.volume_controller.draw(&mut render_queue);

        // draw cursor
        self.cursor_manager.draw(&mut render_queue);

        // toss the items to the window to render
        let _ = self.window_proxy.send_event(WindowAction::RenderData(render_queue.take()));

        self.fps_display.increment();

        // let elapsed = timer.elapsed().as_secs_f32() * 1000.0;
        // if elapsed > 1000.0/144.0 {warn!("render took a while: {elapsed}")}
    }

    fn check_keys(
        &mut self, 
        keys_down: &mut KeyCollection,
        mods: &KeyModifiers,
    ) {

        // check user panel
        #[cfg(feature="graphics")] 
        if keys_down.has_and_remove(self.settings.key_user_panel) {
            self.handle_make_userpanel();
        }


        // screenshot
        #[cfg(feature="graphics")] 
        if keys_down.has_and_remove(Key::F12) {
            self.window_proxy.send_event(WindowAction::TakeScreenshot(ScreenshotInfo {
                // if shift is pressed, upload to server, and get link
                upload: mods.shift,
            })).unwrap();
        }

        // settings menu
        if keys_down.has_key(Key::O) && mods.ctrl {
            keys_down.remove_key(Key::O);

            let is_ingame = self.current_state.is_ingame();
            let allow_ingame = self.settings
                .common_game_settings
                .allow_ingame_settings;

            #[cfg(feature="graphics")] 
            if !is_ingame || allow_ingame {
                self.handle_custom_dialog(
                    "settings", 
                    DialogCreateOptions::default(), 
                    BuildableInputArguments::default(),
                );
                // self.ui_manager.add_dialog(
                //     Box::new(SettingsMenu::new(&self.values.settings)), 
                //     SettingsMenu::DEFAULT_OPTIONS,
                //     &mut self.values, 
                //     &mut self.actions,
                // );z
            }
        }

        // debug
        if keys_down.has_key(Key::PageUp) && mods.ctrl {
            keys_down.remove_key(Key::PageUp);
            debug!("{:#?}", self.values.values);
        }

        // custom menu list
        if keys_down.has_key(Key::M) && mods.ctrl && mods.shift {
            keys_down.remove_key(Key::M);
            
            self.actions.push(MultiplayerAction::CreateLobby { 
                name: "a".to_string(), 
                password: String::new(), 
                private: false, 
                players: 5
            });

            // self.actions.push(MenuAction::set_menu("menu_list"));
        }
        // #[cfg(feature="graphics")] 
        // if keys_down.has_key(Key::H) && mods.ctrl && mods.shift {
        //     keys_down.remove_key(Key::H);
        //     warn!("{:#?}", self.ui_manager.root_tree.print());
        // }
        
        #[cfg(feature="graphics")] 
        if keys_down.has_key(Key::T) && mods.ctrl && mods.shift {
            keys_down.remove_key(Key::T);
            self.ui_manager.root_tree.print(&self.values);
        }

        #[cfg(feature="graphics")] 
        if !self.current_state.is_ingame() && keys_down.has_key(Key::Grave) {
            keys_down.remove_key(Key::Grave);

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


        // update any dialogs
        #[cfg(feature="graphics")] 
        if keys_down.has_key(Key::Escape) 
            && self.ui_manager.close_latest(&mut self.values, &mut self.actions) {
            keys_down.remove_key(Key::Escape);
        }

        if keys_down.has_key(Key::F5) && mods.ctrl {
            keys_down.remove_key(Key::F5);
            self.actions.push(Notification::new_text(
                "Doing a full refresh, the game will freeze for a bit", 
                Color::RED, 
                5000.0
            ));
            let settings = self.settings.clone();
            self.beatmap_manager.full_refresh(&settings);
        }

        // FIXME: move to menus??
        if mods.ctrl {
            for (key, index) in [
                (Key::Key1, 0),
                (Key::Key2, 1),
                (Key::Key3, 2),
                (Key::Key4, 3),
            ] {
                if !keys_down.has_key(key) { continue }
                let Some(mode) = self.global
                    .gamemode_infos
                    .by_num.get(index) 
                else { continue };

                let mode = mode.id;
                self.actions.push(BeatmapAction::SetPlaymode(mode.to_string()));
                self.actions.push(Notification::new_text(
                    format!("Playmode set to {mode}"), 
                    Color::CYAN,
                    3000.0
                ));
            }
        }

        // reload custom menus
        #[cfg(feature="graphics")] 
        if keys_down.has_key(Key::R) && mods.ctrl {
            keys_down.remove_key(Key::R);
            self.load_custom_menus();

            debug!("Reloading current menu");
            self.handle_custom_menu(
                self.ui_manager.get_menu().clone(), 
                None
            );
        }

    }


    #[cfg(feature="graphics")]
    fn handle_previous_menu(&mut self, current_menu: &str)  {
        let in_multi = self.multiplayer_manager.is_some();
        let in_spec = self.spectator_manager.is_some();

        if in_multi { 
            return self.handle_custom_menu("lobby_menu", None);
        }
        if in_spec { 
            return self.handle_custom_menu("beatmap_select", None); 
        }

        match current_menu {
            // score menu with no multi or spec is the beatmap select menu
            "score_menu" => self.handle_custom_menu("beatmap_select", None), 

            // beatmap menu with no multi or spec is the main menu
            "beatmap_select" => self.handle_custom_menu("main_menu", None),

            _ => {
                error!("unhandled previous menu request for menu {current_menu}");
            }
        }
    }

    fn handle_actions(&mut self, actions: Option<Vec<TatakuAction>>) {
        if let Some(actions) = actions {
            self.actions.extend(actions);
        }

        for action in self.actions.take() {
            self.handle_action(action);
        }
    }


    fn handle_action(
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
                => self.cursor_manager.handle_cursor_action(action),

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

    #[cfg(feature="graphics")] 
    fn next_gameplay_id(&self) -> GameplayId {
        Arc::new(self.gameplay_managers
            .keys()
            .max()
            .map(|a| **a + 1)
            .unwrap_or_default()
        )
    }

    #[cfg(feature="graphics")]
    fn handle_custom_menu(
        &mut self, 
        id: impl ToString,
        input: Option<BuildableInputArguments>,
    ) {
        let selector = (
            id.to_string(), 
            CustomMenuSource::Any
        );

        // let menu = self.custom_menus.iter().rev().find(|cm| cm.id == id);
        if let Some(menu) = self.custom_menu_manager.get_menu(selector) {
            match menu.build(&mut self.values, input.unwrap_or_default()) {
                Ok(m) => self.queue_state_change(
                    GameState::SetMenu(Box::new(m))
                ),

                Err(e) 
                    => error!("Error building dialog: {e:?}"),
            }
        } else {
            let id = id.to_string();
            match &*id {
                "none" => {}
                "main_menu" 
                    => panic!("Main menu could not be loaded. did eve fuck up the main_menu.xml?"),
                _ => {
                    error!("custom menu not found! {id}, going to main menu instead");
                    self.actions.push(MenuAction::set_menu("main_menu"));
                }
            }
        }
    }

    #[cfg(feature="graphics")]
    fn handle_custom_dialog(
        &mut self, 
        id: impl ToString, 
        options: DialogCreateOptions,
        input: BuildableInputArguments,
    ) {
        let id = id.to_string();
        let Some(dialog) = self.custom_menu_manager
            .get_dialog((id.clone(), CustomMenuSource::Any))
        else {
            // if id == "settings" {
            //     self.ui_manager.add_dialog(
            //         Box::new(SettingsMenu::new(&self.values.settings)), 
            //         SettingsMenu::DEFAULT_OPTIONS,
            //         &mut self.values, 
            //         &mut self.actions,
            //     );
            // } else {
                error!("unknown dialog id: {id}");
            // }

            return;
        };

        match dialog.build(&mut self.values, input) {
            Ok(d) => self.ui_manager.add_dialog(
                Box::new(d), 
                DialogCreateOptions::merge(
                    options, 
                    dialog.options()
                ),
                &mut self.values, 
                &mut self.actions,
            ),
            Err(e) => {
                error!("Error building dialog: {e:?}");
            }
        }

    }

    #[cfg(feature="gameplay")]
    fn queue_state_change(&mut self, state: GameState) {
        match state {
            #[cfg(feature="graphics")] 
            GameState::SetMenu(menu) => {
                debug!("Changing menu to: {}", menu.name());
                self.queued_state = GameState::InMenu(menu.name().into_owned());
                self.ui_manager.set_root(
                    menu, 
                    &mut self.values, 
                    &mut self.actions
                );
                self.queued_events.push((TatakuEventType::MenuEnter, None));
                self.ui_manager.reload_skin(
                    &mut self.values, 
                    &mut self.actions, 
                    &mut self.skin_manager
                );
            }
            GameState::InMenu(_) => {}
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
                    &mut self.actions
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
                BuildableInputArguments::default(),
            );
        } else {
            self.ui_manager.dialogs.retain(
                |d| d.get_node().name() != "user_panel"
            );
        }
    }

    fn handle_event(&mut self, event: TatakuIntegrationEvent) {
        for i in self.integrations.iter_mut() {
            i.handle_event(&event, &self.values, &mut self.actions);
        }
    }

    #[cfg(feature="graphics")]
    fn resize_bg(&mut self) {
        let Some(bg) = &mut self.background_image 
        else { return };

        bg.fit_to_bg_size(self.values.game.window_size);
    }

    /// Drag and Drop
    #[cfg(feature="graphics")]
    fn handle_file_drop(&mut self, path: impl AsRef<Path>) {
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
                            )
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
                                    last.clone(),
                                    SetBeatmapOptions::new()
                                        .use_preview_point(use_preview_time)
                                        .restart_song(false)
                                ));
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
                            )
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
                                ));
                            }
                        }
                    }
                }

                // tataku | osu replay
                "ttkr" | "osr" => {
                    match read_replay_path(path, &self.global.gamemode_infos) {
                        Ok(score) => self.try_open_replay(score),
                        Err(e) => self.actions.push(
                            Notification::new_error(
                                "Error opening replay", 
                                e
                            )
                        ),
                    }
                }

                _ => {
                    self.actions.push(Notification::default()
                        .text("What is this?")
                        .color(Color::RED)
                        .duration(3_000.0)
                    );
                }
            }
        }
    }

    #[cfg(feature="graphics")]
    fn try_open_replay(&mut self, score: Score) {
        let Some(map) = self
            .beatmap_manager
            .get_by_hash(&score.beatmap_hash) 
        else {
            self.actions.push(
                Notification::default()
                .text("You don't have this beatmap!")
                .duration(5_000.0)
                .color(Color::RED)
            );
            return;
        };

        let config = self.create_select_beatmap_config(
            true, 
            true
        );
        
        self.values.beatmap_manager.set_current_beatmap(
            &map,
            config,
            &mut self.difficulty_manager,
        );

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
        self.handle_custom_menu(
            "score_menu",
            Some(BuildableInputArguments::default()
                .add("allow_retry", false)
            )
        );

    }


    #[cfg(feature="graphics")]
    fn ingame_complete(&mut self, mut manager: Box<GameplayManager>) {
        trace!("beatmap complete");
        manager.on_complete();
        manager.score.time = chrono::Utc::now().timestamp() as u64;
        self.actions.push(TatakuIntegrationEvent::BeatmapEnded);
        self.actions.push(CursorAction::SetVisible(true));

        if manager.failed {
            trace!("player failed");
            if !manager.get_mode().is_multi() {
                self.pending_gameplay_manager = Some(manager);
                self.actions.push(MenuAction::SetMenu {
                    id: "fail_menu".into(),
                    input: Box::new(BuildableInputArguments::default())
                });
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
                        )
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
                self.actions.push(TaskAction::AddTask(Box::new(submit_task)));
            }

            match manager.get_mode() {
                // go back to beatmap select
                GameplayModeInner::Replaying {..} => {
                    self.handle_custom_menu("beatmap_select", None);
                }
                GameplayModeInner::Multiplayer { .. } => {
                    debug!("multiplayer finished gameplay");

                    // FIXME: show the scores lmao
                    // go back to the lobby menu
                    self.handle_custom_menu("lobby_menu", None);
                }

                _ => {
                    // show score menu
                    self.handle_custom_menu(
                        "score_menu",
                        Some(BuildableInputArguments::default()
                            .add("allow_retry", true)
                            .add("score_path", score_submit.unwrap_or_default())
                        )
                    );

                    // let mut menu = ScoreMenu::new(&score, manager.metadata.clone(), true, self.global.gamemode_infos.clone());
                    // menu.score_submit = score_submit;
                    // self.queue_state_change(GameState::SetMenu(Box::new(menu)));
                }
            }
        }

        manager.cleanup_textures(&mut self.skin_manager);
    }

    #[cfg(feature="graphics")] 
    fn load_theme(&mut self) {
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
        )));

        if info.upload {
            self.task_manager.add_task(Box::new(
                UploadScreenshotTask::new(full_path))
            );
        }

        Ok(())
    }


    fn update_playmode(&mut self, playmode: &str) {
        // ensure lowercase
        let playmode: Arc<str> = playmode.to_lowercase().into();

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
            .current_beatmap.as_ref()
            .filter(|b| !info.can_load_beatmap(&b.beatmap_type))
            .map(|b| b.mode.clone())
            .unwrap_or(playmode)
            ;

        self.values.global.update_playmode_actual(actual_playmode);

        // TODO: update mods list as well?
    }


    fn create_select_beatmap_config(
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
}

// action handlers. here bc they're so big
impl Game {

    #[cfg(feature="graphics")]
    fn handle_menu_action(&mut self, action: MenuAction) {
        match action {
            MenuAction::SetMenu { 
                id, 
                input 
            } => self.handle_custom_menu(id, Some(*input)),

            MenuAction::PreviousMenu(current_menu) 
                => self.handle_previous_menu(&current_menu),

            MenuAction::AddDialog {
                id, 
                options, 
                input
            } => self.handle_custom_dialog(id.to_string(), *options, *input),

            MenuAction::AddDialogRaw {
                dialog, 
                options
            } => {
                self.ui_manager.add_dialog(
                    dialog, 
                    *options, 
                    &mut self.values, 
                    &mut self.actions,
                );
            }
        }
    }

    fn handle_song_action(&mut self, action: SongAction) {
        match action {
            // needs to be before trying to get the audio because audio might be none when this is run
            SongAction::Set(action) => {
                if let Err(e) = self.song_manager.handle_song_set_action(
                    action, 
                    &mut self.actions, 
                    &mut self.audio_manager,
                    &self.values.settings
                ) {
                    error!("Error handling SongMenuSetAction: {e:?}");
                }

                let Some(audio) = self.song_manager.instance() 
                else { return };
                
                let Some(current) = &self.beatmap_manager.current_beatmap
                else { return };

                self.actions.push(TatakuIntegrationEvent::SongChanged { 
                    artist: current.artist.clone(), 
                    title: current.title.clone(), 
                    image_path: current.image_filename.clone(), 
                    elapsed: audio.get_position(), 
                    duration: audio.get_duration()
                });
            }
            #[cfg(feature="graphics")] 
            SongAction::HookFFT(hook) => self.song_manager.hook_fft(hook),
            
            other => {
                let Some(audio) = self.song_manager.instance() 
                else { return };

                match other {
                    SongAction::Play => audio.play(false),
                    SongAction::Restart => audio.play(true),
                    SongAction::Pause => audio.pause(),
                    SongAction::Stop => audio.stop(),
                    SongAction::Toggle if audio.is_playing() => audio.pause(),
                    SongAction::Toggle => audio.play(false),
                    SongAction::SeekBy(seek) 
                        => audio.set_position(audio.get_position() + seek),
                    SongAction::SetPosition(pos) => audio.set_position(pos),
                    SongAction::SetRate(rate) => audio.set_rate(rate),
                    SongAction::SetVolume(vol) => audio.set_volume(vol),
                    
                    // handled above
                    _other => unreachable!()
                }
            }
        }

        // update song state
        self.values.song.update(self.song_manager.instance());
    }

    fn handle_mod_action(&mut self, action: ModAction) {
        let mods = &mut self.values.global.mods;
        match action {
            ModAction::AddMod(mod_name) => mods.add_mod(mod_name).nope(),
            ModAction::RemoveMod(mod_name) => mods.remove_mod(mod_name),
            ModAction::ToggleMod(mod_name) => mods.toggle_mod(mod_name).nope(),
            ModAction::SetSpeed(speed) => mods.set_speed(speed),
            ModAction::AddSpeed(speed) => mods.set_speed(mods.get_speed() + speed),
            ModAction::SetMods(new_mods) => mods.mods = new_mods,
        }
        self.values.global.update_mods();

        // update the song's rate
        self.actions.push(SongAction::SetRate(
            self.values.global.mods.get_speed()
        ));

        // apply mods to all gameplay managers
        #[cfg(feature="graphics")] 
        for (m, i) in self.gameplay_managers.values_mut() {
            if i.mods.is_some() { continue }
            m.apply_mods(self.values.global.mods.clone());
        }

        // update the beatmap groupings to update the diffs
        let mods = self.global.mods.clone();
        let playmode = self.global.playmode.clone();
        let sort_by = self.values.settings.sort_by;
        self.values.beatmap_manager.apply_filter(
            &mods, 
            &playmode, 
            sort_by,
            &mut self.difficulty_manager,
        );
    }

    fn handle_beatmap_action(&mut self, action: BeatmapAction) {
        match action {
            #[cfg(feature="gameplay")]
            BeatmapAction::PlaySelected => {
                let Some(map) = self
                    .beatmap_manager
                    .current_beatmap
                    .clone() 
                else { return };

                let mods = self.global.mods.clone();
                let mode = self.global.playmode.clone();

                match manager_from_playmode(
                    &self.global.gamemode_infos,
                    &mode, 
                    &map, 
                    mods.clone(),
                    &self.settings,
                ) {
                    Ok(mut manager) => {
                        let start_time = manager.start_time as u64;

                        manager.handle_action(
                            GameplayAction::ApplyMods(mods), 
                            &self.settings
                        );

                        self.queue_state_change(
                            GameState::Ingame(Box::new(manager))
                        );

                        let multiplayer = self.multiplayer_manager
                            .as_ref()
                            .map(|a| a.lobby.id)
                            .and_then(|i| self.online_manager.lobby(i).cloned())
                            ;

                        self.handle_event(TatakuIntegrationEvent::BeatmapStarted { 
                            start_time, 
                            beatmap: map.map.clone(), 
                            playmode: mode, 
                            multiplayer, 
                            spectator: self.spectator_manager
                                .as_ref()
                                .map(|s| s.host_username.clone())
                        });
                    }
                    Err(e) => self.actions.push(Notification::new_error(
                        "Error loading beatmap", 
                        e
                    )),
                }
            }

            #[cfg(feature="gameplay")]
            BeatmapAction::ConfirmSelected => {
                if let Some(multi) = &mut self.multiplayer_manager {
                    // go back to the lobby before any checks
                    // this way if for some reason something down below fails, the user is in the lobby and not stuck in limbo
                    #[cfg(feature="graphics")] 
                    self.actions.push(MenuAction::set_menu("lobby_menu"));

                    if !multi.is_host() { 
                        return warn!("trying to set lobby beatmap while not the host ??");
                    };

                    let Some(map) = self.values
                        .beatmap_manager
                        .current_beatmap.clone() 
                    else { return };

                    let playmode = &self.values.values.global.playmode;
                    self.values.values.online_manager.update_lobby_beatmap(
                        &map, 
                        playmode.to_string()
                    );

                } else if let Some(spec_man) = self
                    .spectator_manager
                    .as_mut() 
                {
                    let Some(map) = self.values
                        .beatmap_manager
                        .current_beatmap.clone() 
                    else { return };

                    self.values.online_manager.handle_action(OnlineAction::ChatAction(
                        ChatAction::SendMessage { 
                            channel: spec_man.host_username.to_string(), 
                            message: BeatmapLink {
                                beatmap_hash: map.beatmap_hash.to_string(),
                                beatmap_title: map.version_string(),
                                download_link: None,
                            }.to_string()
                        }
                    ));
                } else {
                    // play map
                    self.handle_beatmap_action(BeatmapAction::PlaySelected);
                }
            }

            BeatmapAction::Set(
                beatmap, 
                options
            ) => {
                self.handle_beatmap_action(BeatmapAction::SetFromHash(
                    beatmap.beatmap_hash, 
                    options
                ));
            }
            BeatmapAction::SetFromHash(hash, options) => {
                if let Some(beatmap) = self
                    .beatmap_manager
                    .get_by_hash(&hash) 
                {
                    let config = self.create_select_beatmap_config(
                        options.restart_song,
                        options.use_preview_point,
                    );
                    self.values.beatmap_manager.set_current_beatmap(
                        &beatmap,
                        config,
                        &mut self.difficulty_manager,
                    );
                    return;
                }

                #[cfg(feature="gameplay")]
                if self.multiplayer_manager.is_some() {
                    // if we're in a multiplayer lobby, and the map doesnt exist, remove the map
                    self.handle_beatmap_action(BeatmapAction::Remove);
                    return
                }

                match options.if_none {
                    MapActionIfNone::ContinueCurrent => {},
                    MapActionIfNone::SetNone 
                        => self.handle_beatmap_action(BeatmapAction::Remove), 
                    MapActionIfNone::Random(preview) => {
                        let Some(map) = self
                            .beatmap_manager
                            .random_beatmap() 
                        else { return };

                        self.handle_beatmap_action(BeatmapAction::SetFromHash(
                            map.beatmap_hash, 
                            options.use_preview_point(preview)
                        ));
                    }
                }

            }

            BeatmapAction::SetPlaymode(new_mode) 
                => self.update_playmode(&new_mode),

            BeatmapAction::Random(use_preview) => {
                let Some(random) = self
                    .beatmap_manager
                    .random_beatmap() 
                else { return };

                let config = self.create_select_beatmap_config(
                    true,
                    use_preview
                );
                self.values.beatmap_manager.set_current_beatmap(
                    &random,
                    config,
                    &mut self.difficulty_manager,
                );
            }
            BeatmapAction::Remove => {
                self.beatmap_manager.remove_current_beatmap();
                // warn!("removing beatmap");
                #[cfg(feature="graphics")] {
                    self.background_image = None;
                }
            }

            BeatmapAction::Delete(hash) => {
                let config = self.create_select_beatmap_config(
                    true, true
                );

                self.values.beatmap_manager.delete_beatmap(
                    hash,
                    PostDelete::Next,
                    config,
                    &mut self.difficulty_manager,
                );
            }
            BeatmapAction::DeleteCurrent(post_delete) => {
                let Some(map_hash) = self.values
                    .current_beatmap_prop(|b| b.beatmap_hash) 
                else { return };

                let config = self.create_select_beatmap_config(
                    true, 
                    true
                );
                
                self.values.beatmap_manager.delete_beatmap(
                    map_hash,
                    post_delete,
                    config,
                    &mut self.difficulty_manager,
                );
            }
            BeatmapAction::Next => {
                let config = self.create_select_beatmap_config(
                    true, 
                    false
                );

                self.values.beatmap_manager.next_beatmap(
                    config, 
                    &mut self.difficulty_manager,
                );
            }
            BeatmapAction::Previous(if_none) => {
                let mut config = self
                    .create_select_beatmap_config(
                        true, 
                        false
                    );

                if self.values.beatmap_manager.previous_beatmap(
                    config.clone(), 
                    &mut self.difficulty_manager,
                ) { return }

                // no previous map availble, handle accordingly
                match if_none {
                    MapActionIfNone::ContinueCurrent => return,
                    MapActionIfNone::Random(use_preview) => {
                        config.use_preview_time = use_preview;

                        let Some(random) = self
                            .beatmap_manager
                            .random_beatmap() 
                        else { return };

                        self.values.beatmap_manager.set_current_beatmap(
                            &random, 
                            config, 
                            &mut self.difficulty_manager,
                        );
                    }
                    MapActionIfNone::SetNone 
                        => self.beatmap_manager.remove_current_beatmap(),
                }
            }

            BeatmapAction::InitializeManager => {
                self.values.values.beatmap_manager.initialize(
                    self.values.values.settings.sort_by, 
                    &self.values.values.global.mods, 
                    &self.values.values.global.playmode, 
                    &mut self.difficulty_manager,
                );
            }
            BeatmapAction::AddBeatmap { map, add_to_db } => {
                self.beatmap_manager.add_beatmap(
                    &map, 
                    add_to_db,
                );

                self.values.values.beatmap_manager.refresh_maps(
                    &self.values.values.global.mods, 
                    &self.values.values.global.playmode, 
                    self.values.values.settings.sort_by,
                    &mut self.difficulty_manager,
                );
            }


            // beatmap list actions
            BeatmapAction::ListAction(list_action) => {
                match list_action {
                    BeatmapListAction::Refresh => {
                        let mods = self.global.mods.clone();
                        let playmode = self.global.playmode.clone();
                        let sort_by = self.values.settings.sort_by;
                        self.values.beatmap_manager.refresh_maps(
                            &mods, 
                            &playmode, 
                            sort_by,
                            &mut self.difficulty_manager,
                        );
                    }

                    BeatmapListAction::ApplyFilter { filter } => {
                        self.beatmap_manager.filter_text = filter.unwrap_or_default();
                        let mods = self.global.mods.clone();
                        let playmode = self.global.playmode.clone();
                        let sort_by = self.values.settings.sort_by;
                        self.values.beatmap_manager.apply_filter(
                            &mods,
                            &playmode,
                            sort_by,
                            &mut self.difficulty_manager
                        );
                    }
                    BeatmapListAction::NextMap => self.beatmap_manager.next_map(),
                    BeatmapListAction::PrevMap => self.beatmap_manager.prev_map(),
                    BeatmapListAction::NextSet => self.beatmap_manager.next_set(),
                    BeatmapListAction::PrevSet => self.beatmap_manager.prev_set(),

                    BeatmapListAction::SelectSet(set_id) 
                        => self.beatmap_manager.select_set(set_id),
                }
            }

            #[cfg(not(feature="gameplay"))]
            _ => {}
        }

        // handle beatmap manager actions
        let bm_actions = self.beatmap_manager.actions.take();
        for i in bm_actions {
            self.handle_action(i);
        }
    }

    fn handle_current_game_action(&mut self, action: CurrentGameAction) {
        if matches!(action, CurrentGameAction::Pause {..}) {

            #[cfg(feature="graphics")]
            let CurrentGameAction::Pause { 
                id, 
                input 
            } = action else { unreachable!() };
            
            if !self.current_state.is_ingame() {
                warn!("got pause for current gameplay but no current gameplay");
                return
            }

            let GameState::Ingame(gameplay) = self
                .current_state
                .take() 
            else { unreachable!() };

            self.current_state = GameState::None;
            #[cfg(feature="graphics")] 
            self.handle_menu_action(MenuAction::SetMenu {
                id: id.into(),
                input: Box::new(input),
            });
            
            // make sure it has the latest window size
            self.pending_gameplay_manager = Some(gameplay);
            return;
        }

        let Some(mut manager) = self.pending_gameplay_manager
            .take() 
        else { 
            warn!("Got action {action:?} but no pending gameplay manager");
            return 
        };

        match action {
            CurrentGameAction::Start => {
                manager.start();
                self.queue_state_change(GameState::Ingame(manager));
            }
            CurrentGameAction::Resume => {
                self.queue_state_change(GameState::Ingame(manager));
            }
            CurrentGameAction::Restart => {
                manager.reset();
                self.queue_state_change(GameState::Ingame(manager));
            }
            CurrentGameAction::Free => {
                #[cfg(feature="graphics")] 
                manager.cleanup_textures(&mut self.skin_manager);
            }

            CurrentGameAction::Pause { .. } => unreachable!(),
        }
    }
    
    fn handle_game_action(&mut self, action: GameAction) {
        match action {
            #[cfg(feature="gameplay")]
            GameAction::Quit => self.queue_state_change(GameState::Closing),
            GameAction::RestartOnline => self.init_online(),

            GameAction::CurrentGameAction(action) 
                => self.handle_current_game_action(action),

            #[cfg(feature="gameplay")]
            GameAction::WatchReplay(score) => {
                let map = score.beatmap_hash;
                let mode = &score.playmode;

                let Some(beatmap) = self.beatmap_manager
                    .get_by_hash(&map) 
                else {
                    self.actions.push(
                        Notification::default()
                        .text("You don't have that map!")
                        .duration(5000.0)
                        .color(Color::RED)
                    );
                    return;
                };

                let mods = self.values.global.mods.clone();

                match manager_from_playmode_path_hash(
                    &self.global.gamemode_infos,
                    mode, 
                    &beatmap.file_path, 
                    beatmap.beatmap_hash, 
                    mods, 
                    &self.settings
                ) {
                    Ok(mut manager) => {
                        manager.set_mode(GameplayMode::Replay(score).into());
                        self.queue_state_change(GameState::Ingame(Box::new(
                            manager
                        )));
                    }
                    Err(e) => self.actions.push(
                        Notification::new_error(
                            "Error loading beatmap", 
                            e
                        )
                    ),
                }
            }
            GameAction::SetValue(key, value) => {
                let values = self.values.as_dyn_mut();
                let a = format!("{value:?}");

                let r = match value {
                    TatakuValue::F32(n) 
                        => values.reflect_insert(&key, n),
                    TatakuValue::U32(n) 
                        => values.reflect_insert(&key, n),
                    TatakuValue::U64(n) 
                        => values.reflect_insert(&key, n),
                    TatakuValue::Bool(b) 
                        => values.reflect_insert(&key, b),
                    TatakuValue::String(s) 
                        => values.reflect_insert(&key, s),
                    TatakuValue::Reflect(reflect) 
                        => values.impl_insert(
                            ReflectPath::new(&key), 
                            reflect
                        ),
                    
                    other => {
                        warn!("OTHER NOT HANDLED!!! {other:?}");
                        Ok(())
                    }
                };
                if let Err(e) = r {
                    error!("Error updating value '{key}' with {a}: {e:?}");
                }
            }
            #[cfg(feature="graphics")]
            GameAction::ViewScore(score) => {
                if let Some(beatmap) = self
                    .beatmap_manager
                    .get_by_hash(&score.beatmap_hash)
                {
                    self.values.values.beatmap_manager.set_current_beatmap(
                        &beatmap, 
                        SelectBeatmapConfig::new(
                            ModManager {
                                speed: score.speed,
                                mods: score.mods
                                    .iter()
                                    .map(|i| i.name.clone())
                                    .collect()
                            },
                            score.playmode.clone().into(),
                            false,
                            true
                        ),
                        &mut self.difficulty_manager,
                    );

                    let info = self.values.global
                        .gamemode_infos
                        .get_info(&score.playmode)
                        .copied()
                        .unwrap_or_default();

                    self.values.values.score = ReflectScore::new(&score, &info);

                    // show score menu
                    self.handle_custom_menu(
                        "score_menu",
                        Some(BuildableInputArguments::default()
                            .add("allow_retry", false)
                        )
                    );
                } else {
                    error!("Could not find map from score!");
                }
            }
            #[cfg(feature="graphics")]
            GameAction::HandleMessage(message) 
                => self.ui_manager.add_message(message),

            GameAction::RefreshScores => self.score_manager.force_update = true,
            #[cfg(feature="graphics")] 
            GameAction::ViewScoreId(id) => {
                if let Some(score) = self.score_manager.get_score(id) {
                    self.handle_game_action(GameAction::ViewScore(score.clone()));
                }
            }
            #[cfg(feature="graphics")]
            GameAction::HandleEvent(
                event, 
                param
            ) => self.queued_events.push((event, param)),
            
            #[cfg(feature="graphics")] 
            GameAction::AddNotification(
                notif
            ) => self.notification_manager.add_notification(notif),

            #[cfg(feature="graphics")]
            GameAction::UpdateBackground => {
                let Some(filename) = self.values.current_beatmap_prop(
                    |b| b.image_filename.clone()
                ) 
                else { return };
                
                self.background_image = self.skin_manager.get_texture(
                    &filename, 
                    &TextureSource::Raw, 
                    SkinUsage::Background, 
                    false
                );

                if let Some(i) = &mut self.background_image {
                    i.origin = Vector2::ZERO;
                }

                self.resize_bg();
            },
            #[cfg(feature="graphics")]
            GameAction::CopyToClipboard(text) => { 
                let _ = self.window_proxy.send_event(
                    WindowAction::CopyToClipboard(text)
                ); 
            }

            GameAction::RefreshPlaymodeValues => {
                let playmode = self.global.playmode.clone();
                self.update_playmode(&playmode);
            }
            GameAction::UpdatePlaymodeActual(actual) => {
                self.values.global.update_playmode_actual(actual);
            }

            GameAction::RefreshSkins => {
                SkinManager::refresh_skins();
            }


            #[cfg(feature="graphics")]
            GameAction::NewGameplayManager(config) => {
                match match &config {
                    NewManager {
                        mods,
                        map_hash: Some(map_hash),
                        path: Some(path),
                        playmode,
                        ..
                    } => {
                        let playmode = playmode.clone()
                            .unwrap_or_else(|| self.values.global.playmode.clone());
                        let mods = mods.clone()
                            .unwrap_or_else(|| self.values.global.mods.clone());
                        
                        manager_from_playmode_path_hash(
                            &self.global.gamemode_infos,
                            &playmode, 
                            path, 
                            *map_hash, 
                            mods, 
                            &self.settings,
                        )
                    }
                    NewManager {
                        mods,
                        map_hash,
                        playmode,
                        ..
                    } => {
                        let map_hash = map_hash.unwrap_or_else(
                            || self.values.current_beatmap_prop(
                                |b| b.beatmap_hash
                                )
                                .unwrap_or_default()
                        );

                        let Some(meta) = self
                            .beatmap_manager
                            .get_by_hash(&map_hash) 
                        else { return };
                        let playmode = playmode.clone().unwrap_or_else(
                            || self.values.global.playmode_actual.clone()
                        );

                        let mods = mods.clone().unwrap_or_else(
                            || self.values.global.mods.clone()
                        );

                        manager_from_playmode(
                            &self.global.gamemode_infos,
                            &playmode, 
                            &meta, 
                            mods,
                            &self.settings,
                        )
                    }
                } {
                    Ok(mut manager) => {
                        manager.reload_skin(
                            &mut self.skin_manager, 
                            &self.values.settings
                        );

                        if let Some(mode) = config.gameplay_mode.clone() {
                            manager.set_mode(mode.into());
                        }
                        
                        manager.window_size_changed(self.values.game.window_size);
                        
                        if let Some(bounds) = config.area {
                            manager.handle_action(
                                GameplayAction::FitToArea(bounds), 
                                &self.settings
                            );
                        }
                        manager.reset();

                        let id = self.next_gameplay_id();
                        self.ui_manager.add_message(Message::new(
                            config.owner, 
                            "gameplay_manager_create", 
                            MessageValue::Custom(id.clone())
                        ));
                        manager.set_id(id.clone());

                        self.gameplay_managers.insert(id, (manager, config));
                    }

                    Err(e) 
                        => error!("Error creating gameplay manager: {e}"),
                }
            }

            #[cfg(feature="graphics")]
            GameAction::DropGameplayManager(id) => {
                self.gameplay_managers.remove(&id);
            }

            #[cfg(feature="graphics")]
            GameAction::GameplayAction(id, action) => {
                let Some(gameplay) = (if *id == u32::MAX {
                    trace!("gameplay is state");
                    self.current_state
                        .get_ingame()
                        .map(|a| &mut **a)
                } else {
                    self.gameplay_managers
                        .get_mut(&id)
                        .map(|a| &mut a.0)
                })
                else { return };

                
                if let &GameplayAction::RequestDifficulty = &action {
                    gameplay.update_difficulty(&mut self.difficulty_manager);
                } else {
                    gameplay.handle_action(action, &self.values.settings);
                }
            }


            GameAction::UpdateSettings(run) => {
                (run)(&mut self.values.settings);
            }

            #[cfg(not(feature="graphics"))] _ => {} 
        }
    }

    fn handle_multiplayer_action(&mut self, action: MultiplayerAction) {
        match action {
            #[cfg(feature="graphics")]
            MultiplayerAction::StartMultiplayer => {
                self.online_manager.add_lobby_listener();
                self.handle_custom_menu("lobby_select", None);
            },

            #[cfg(feature="graphics")]
            MultiplayerAction::ExitMultiplayer => {
                self.handle_multiplayer_action(MultiplayerAction::LeaveLobby);

                // if ingame, dont change state. this way the user can keep playing the map
                if !self.current_state.is_ingame() {
                    self.handle_custom_menu("main_menu", None);
                }

                self.online_manager.remove_lobby_listener();
            }

            #[cfg(feature="gameplay")]
            MultiplayerAction::CreateLobby { 
                name, 
                password, 
                private, 
                players 
            } => {
                self.online_manager.multiplayer_data.lobby_creation_pending = true;
                info!("sending create");

                self.online_manager.send_packet(
                    MultiplayerPacket::Client_CreateLobby { 
                        name, 
                        password, 
                        private, 
                        players 
                    }
                );
            }
            #[cfg(feature="gameplay")]
            MultiplayerAction::LeaveLobby => {
                self.multiplayer_manager = None;
                self.online_manager.send_packet(MultiplayerPacket::Client_LeaveLobby);
                #[cfg(feature="graphics")] 
                self.handle_custom_menu("lobby_select", None);
            }
            #[cfg(feature="gameplay")]
            MultiplayerAction::JoinLobby { lobby_id, password } => {
                self.online_manager.multiplayer_data.lobby_join_pending = true;
                if let Some(multi_manager) 
                    = &mut self.multiplayer_manager 
                {
                    // if we're already in this lobby, dont do anything
                    if multi_manager.lobby.id == lobby_id { return }

                    // otherwise, leave our current lobby
                    self.handle_multiplayer_action(MultiplayerAction::LeaveLobby);
                }

                self.online_manager.send_packet(MultiplayerPacket::Client_JoinLobby { 
                    lobby_id, 
                    password 
                });
            }

            #[cfg(feature="gameplay")]
            MultiplayerAction::SetBeatmap { hash, mode } => {
                let Some(map) = self.beatmap_manager.get_by_hash(&hash) 
                else { return };

                let mode = mode
                    .unwrap_or_else(|| self.values.global.playmode_actual.clone());

                self.online_manager.update_lobby_beatmap(&map, mode.to_string());
            }

            #[cfg(feature="gameplay")]
            MultiplayerAction::InviteUser { user_id } => {
                self.online_manager.invite_user(user_id);
            }

            // lobby actions
            #[cfg(feature="gameplay")]
            MultiplayerAction::LobbyAction(LobbyAction::Leave) => {
                self.handle_multiplayer_action(MultiplayerAction::LeaveLobby);
            }
            #[cfg(feature="gameplay")]
            MultiplayerAction::LobbyAction(action) => {
                let Some(multi_manager) 
                    = &mut self.multiplayer_manager 
                else { return };

                multi_manager.handle_lobby_action(
                    action, 
                    &self.values.settings, 
                    &mut self.actions
                );
            }

            // ignore unhandled messages when either of these features arent enabled
            #[cfg(any(not(feature="gameplay"), not(feature="graphics")))] _ => {}
        }
    }

    #[cfg(feature="gameplay")]
    fn handle_multiplayer_packet(
        &mut self, 
        packet: MultiplayerPacket
    ) -> TatakuResult {
        // if we have a multi manager, pass the packet onto it as well
        if let Some(multi_manager) 
            = &mut self.multiplayer_manager 
        {
            let ig_manager = self
                .current_state
                .get_ingame();

            let manager_maybe = multi_manager.handle_packet(
                &mut self.values, 
                &packet, 
                ig_manager,
                &mut self.actions,
            )?;

            if let Some(manager) = manager_maybe {
                // start the manager
                debug!("multi starting gameplay");
                self.queue_state_change(GameState::Ingame(Box::new(manager)));
            }
        }

        match packet {
            MultiplayerPacket::Server_LobbyList { lobbies } => {
                self.online_manager.lobbies = lobbies;
            }

            MultiplayerPacket::Server_LobbyInvite { 
                inviter_id, 
                lobby 
            } => {
                let username = if let Some(user) 
                    = self.online_manager.users.get(&inviter_id)
                {
                    user.username.clone()
                } else {
                    "A user".to_owned()
                };

                // mark the lobby we have saved as without password, so the user isnt prompted to enter the password
                // since we were invited, we can join without the password
                if let Some(l) = self.online_manager.lobby(lobby.id) { 
                    l.has_password = false;
                }

                self.actions.push(
                    Notification::default()
                    .text(format!("{username} has invited you to a multiplayer match"))
                    .color(Color::PURPLE_AMETHYST)
                    .duration(10_000.0)
                    .onclick(NotificationOnClick::MultiplayerLobby(lobby.id))
                );
            }

            MultiplayerPacket::Server_CreateLobby { 
                success, 
                lobby 
            } => {
                let Some(lobby) = lobby.filter(|_| success) 
                else { 
                    warn!("no success or lobby"); 
                    return Ok(()) 
                };

                if !self.online_manager.multiplayer_data.lobby_creation_pending { 
                    warn!("no join pending"); 
                    return Ok(()) 
                }

                let our_id = self.online_manager.user_id;
                if our_id == 0 { warn!("user_id == 0"); return Ok(()) }


                let mut info = CurrentLobbyInfo::new(
                    lobby, 
                    our_id
                );
                self.online_manager.update_usernames(&mut info);

                let manager = MultiplayerManager::new(
                    info, 
                    self.global.gamemode_infos.clone(),
                    &mut self.actions,
                );
                manager.update_values(&mut self.values);
                self.multiplayer_manager = Some(Box::new(manager));
                #[cfg(feature="graphics")]
                self.handle_custom_menu("lobby_menu", None);


                // try to update the server with our current map and mode
                let Some(map_hash) = self.values.current_beatmap_prop(
                    |b| b.beatmap_hash
                ) 
                else { return Ok(()) };

                let Some(map) = self.beatmap_manager
                    .get_by_hash(&map_hash) 
                else { return Ok(()) };

                let mode = self.global.playmode.clone();
                self.online_manager.update_lobby_beatmap(&map, mode.to_string());
            }
            MultiplayerPacket::Server_JoinLobby { 
                success, 
                lobby 
            } => {
                let Some(lobby) = lobby.filter(|_| success) 
                else { 
                    return Ok(()) 
                };
                if !self.online_manager.multiplayer_data.lobby_join_pending { 
                    return Ok(()) 
                }

                let our_id = self.online_manager.user_id;
                if our_id == 0 { return Ok (()) }

                let mut info = CurrentLobbyInfo::new(
                    lobby, 
                    our_id
                );
                self.online_manager.update_usernames(&mut info);

                let manager = MultiplayerManager::new(
                    info, 
                    self.global.gamemode_infos.clone(),
                    &mut self.actions,
                );
                manager.update_values(&mut self.values);
                self.multiplayer_manager = Some(Box::new(manager));
                #[cfg(feature="graphics")]
                self.handle_custom_menu("lobby_menu", None);
            }


            MultiplayerPacket::Server_LobbyCreated { lobby } => {
                self.online_manager.lobbies.push(lobby);
            }
            MultiplayerPacket::Server_LobbyDeleted { lobby_id } => {
                self.online_manager
                    .lobbies
                    .iter()
                    .enumerate()
                    .find(|(_, l)| l.id == lobby_id)
                    .map(|(i, _)| i)
                    .map(|index| self.online_manager.lobbies.remove(index));
            }

            MultiplayerPacket::Server_LobbyUserJoined { 
                lobby_id, 
                user_id 
            } => {
                if let Some(l) = self.online_manager.lobby(lobby_id) { 
                    l.players.push(user_id);
                }
            }

            MultiplayerPacket::Server_LobbyUserLeft { 
                lobby_id, 
                user_id 
            } => {
                if let Some(l) = self.online_manager.lobby(lobby_id) { 
                    l.players.retain(|u| u != &user_id);
                }

                if let Some(manager) = &self
                    .multiplayer_manager 
                {
                    if manager.lobby.our_user_id == user_id {
                        self.multiplayer_manager = None;
                        self.actions.push(
                            Notification::default()
                            .text("You have been kicked from the match")
                            .duration(3000.0)
                            .color(Color::PURPLE)
                        );
                    }
                }
            }


            MultiplayerPacket::Server_LobbyMapChange { 
                lobby_id, 
                new_map
            } => {
                if let Some(l) = self.online_manager.lobby(lobby_id) { 
                    l.current_beatmap = Some(new_map.title.clone());
                };
            }

            MultiplayerPacket::Server_LobbyStateChange { 
                lobby_id, 
                new_state 
            } => {
                if let Some(l) = self.online_manager.lobby(lobby_id) { 
                    l.state = new_state;
                }
            }

            _ => {}
        }

        Ok(())
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
pub enum GameState {
    #[default]
    None, // use this as the inital game mode, but be sure to change it after
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
    SetMenu(Box<dyn Widget>),

    /// Currently in a menu (this doesnt actually work currently, but it doesnt really matter)
    InMenu(String),
}
impl GameState {
    fn is_ingame(&self) -> bool {
        matches!(self, Self::Ingame(_))
    }
    fn get_ingame(&mut self) -> Option<&mut Box<GameplayManager>> {
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
