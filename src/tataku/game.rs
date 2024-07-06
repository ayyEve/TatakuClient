use crate::prelude::*;
use chrono::{ Datelike, Timelike };

/// how long transitions between states should last
const TRANSITION_TIME:f32 = 500.0;

pub struct Game {
    // engine things
    input_manager: InputManager,
    volume_controller: VolumeControl,
    current_state: GameState,
    queued_state: GameState,
    game_event_receiver: tokio::sync::mpsc::Receiver<Window2GameEvent>,

    window_proxy: winit::event_loop::EventLoopProxy<Game2WindowEvent>,


    // managers

    /// if some, will handle spectator stuff
    spectator_manager: Option<Box<SpectatorManager>>,
    multiplayer_manager: Option<Box<MultiplayerManager>>,
    multiplayer_data: MultiplayerData,
    
    skin_manager: SkinManager,
    beatmap_manager: BeatmapManager,
    song_manager: SongManager,
    score_manager: ScoreManager,
    task_manager: TaskManager,
    custom_menu_manager: CustomMenuManager,

    gameplay_managers: HashMap<GameplayId, (GameplayManager, NewManager)>,


    // fps
    fps_display: FpsDisplay,
    update_display: FpsDisplay,
    render_display: AsyncFpsDisplay,
    input_display: AsyncFpsDisplay,

    // transition
    transition: Option<GameState>,
    transition_last: Option<GameState>,
    transition_timer: f32,

    // misc
    game_start: Instant,
    background_image: Option<Image>,
    wallpapers: Vec<Image>,
    // register_timings: (f32,f32,f32),


    settings: SettingsHelper,
    window_size: WindowSizeHelper,
    cursor_manager: CursorManager,
    last_skin: String,

    background_loader: Option<AsyncLoader<Option<Image>>>,
    spec_watch_action: SpectatorWatchAction,
    ui_manager: UiManager,

    pub actions: ActionQueue,
    pub queued_events: Vec<(TatakuEventType, Option<TatakuValue>)>,

    pub values: ValueCollection,

    song_state: AudioState,
}
impl Game {
    pub async fn new(
        game_event_receiver: tokio::sync::mpsc::Receiver<Window2GameEvent>,
        window_proxy: winit::event_loop::EventLoopProxy<Game2WindowEvent>,
    ) -> Game {
        GlobalValueManager::update::<DirectDownloadQueue>(Arc::new(Vec::new()));
        let settings = Settings::get();
        let skin_manager = SkinManager::new(&settings);
        let skin = skin_manager.skin().clone();

        let mut g = Game {
            // engine
            window_proxy,
            input_manager: InputManager::new(),
            volume_controller: VolumeControl::new().await,
            // dialogs: Vec::new(),
            background_image: None,
            wallpapers: Vec::new(),
            settings: SettingsHelper::new(),
            window_size: WindowSizeHelper::new(),
            spectator_manager: None,
            multiplayer_manager: None,
            multiplayer_data: MultiplayerData::default(),

            beatmap_manager: BeatmapManager::new(),
            song_manager: SongManager::new(),
            score_manager: ScoreManager::new(),
            task_manager: TaskManager::new(),
            custom_menu_manager: CustomMenuManager::new(),
            skin_manager,
            cursor_manager: CursorManager::new(skin).await,
            gameplay_managers: HashMap::new(),

            // menus: HashMap::new(),
            current_state: GameState::None,
            queued_state: GameState::None,
            spec_watch_action: SpectatorWatchAction::FullMenu,

            // fps
            render_display: AsyncFpsDisplay::new("fps", 3, RENDER_COUNT.clone(), RENDER_FRAMETIME.clone()).await,
            fps_display: FpsDisplay::new("draws/s", 2).await,
            update_display: FpsDisplay::new("updates/s", 1).await,
            input_display: AsyncFpsDisplay::new("inputs/s", 0, INPUT_COUNT.clone(), INPUT_FRAMETIME.clone()).await,

            // transition
            transition: None,
            transition_last: None,
            transition_timer: 0.0,

            // misc
            game_start: Instant::now(),
            // register_timings: (0.0,0.0,0.0),
            game_event_receiver,
            last_skin: String::new(),
            background_loader: None,

            ui_manager: UiManager::new(),
            actions: ActionQueue::new(),
            queued_events: Vec::new(),

            values: ValueCollection::new(),
            song_state: AudioState::Unknown,
        };

        g.init().await;

        g
    }

    fn load_custom_menus(&mut self) {
        if self.custom_menu_manager.clear_menus(CustomMenuSource::Any) {
            debug!("Reloading custom menus");
        }

        // macro to help
        macro_rules! load_menu {
            ($self:ident, $path: expr) => {{
                let result;
                #[cfg(any(debug_assertions, load_internal_menus_from_file))] {
                    result = $self.custom_menu_manager.load_menu($path.to_owned(), CustomMenuSource::Game);
                }
                #[cfg(not(any(debug_assertions, load_internal_menus_from_file)))] {
                    const BYTES:&[u8] = include_bytes!(concat!("../", $path));
                    result = $self.custom_menu_manager.load_menu_from_bytes(BYTES, $path.to_owned(), CustomMenuSource::Game);
                }

                if let Err(e) = result {
                    error!("error loading custom menu {}: {e}", $path);
                }
            }}
        }

        load_menu!(self, "../menus/menu_list.lua");

        load_menu!(self, "../menus/main_menu.lua");
        load_menu!(self, "../menus/beatmap_select_menu.lua");
        load_menu!(self, "../menus/lobby_select.lua");
        load_menu!(self, "../menus/lobby_menu.lua");


        self.custom_menu_manager.update_values(&mut self.values);
        debug!("Done loading custom menus");
    }

    /// initialize all the values in our value collection
    /// doubles as a list of available values because i know i'm going to forget to put them in the doc at some point
    fn init_value_collection(&mut self) {
        use TatakuVariableAccess as Access;

        let values = &mut self.values;

        // game values
        {
            let mut game = ValueCollectionMapHelper::default();
            game.set("time", TatakuVariable::new_game(0.0));

            values.set("game", TatakuVariable::new_game(game.finish()));
        }

        // global variables
        {
            let mut global = ValueCollectionMapHelper::default();

            // playmode, we want this to be writable by the game only, since we also need to update the display value on change
            global.set("playmode", TatakuVariable::new_game("osu").display("Osu"));

            // playmode with map's mode override
            global.set("playmode_actual", TatakuVariable::new_game("osu").display("Osu"));

            global.set("mods", TatakuVariable::new_game(ModManager::new()));
            global.set("username", TatakuVariable::new_game("Guest"));
            global.set("user_id", TatakuVariable::new_game(0u32));
            global.set("new_map_hash", TatakuVariable::new_game(String::new()));
            global.set("lobbies", TatakuVariable::new_game(TatakuValue::List(Vec::new())));
            global.set("menu_list", TatakuVariable::new_game(TatakuValue::List(Vec::new())));

            values.set("global", TatakuVariable::new_game(global.finish()));
        }

        // settings
        {
            let mut settings = ValueCollectionMapHelper::default();
            settings.set("sort_by", TatakuVariable::new_any(self.settings.last_sort_by));
            settings.set("group_by", TatakuVariable::new_any(GroupBy::Set));
            settings.set("score_method", TatakuVariable::new_any(self.settings.last_score_retreival_method));

            values.set("settings", TatakuVariable::new_any(settings.finish()).display("Settings"));
        }

        // enums (for use with dropdowns)
        {
            // technically just lists but whatever
            let mut enums = ValueCollectionMapHelper::default();
            enums.set("sort_by", TatakuVariable::new((Access::ReadOnly, SortBy::list())));
            enums.set("group_by", TatakuVariable::new((Access::ReadOnly, GroupBy::list())));
            enums.set("score_methods", TatakuVariable::new((Access::ReadOnly, ScoreRetreivalMethod::list())));

            let playmodes = AVAILABLE_PLAYMODES
                .iter()
                .map(|m| TatakuVariable::new(*m).display(gamemode_display_name(*m)))
                .collect::<Vec<_>>();
            enums.set("playmodes", TatakuVariable::new(TatakuValue::List(playmodes)));
        
            values.set("enums", TatakuVariable::new_game(enums.finish()));
        }

        // song values
        {
            let mut song = ValueCollectionMapHelper::default();
            song.set("exists", TatakuVariable::new_game(false));
            song.set("playing", TatakuVariable::new_game(false));
            song.set("paused", TatakuVariable::new_game(false));
            song.set("stopped", TatakuVariable::new_game(false));
            song.set("position", TatakuVariable::new_game(0.0));

            values.set("song", TatakuVariable::new_game(song.finish()));
        }

        values.set("beatmap_list", TatakuVariable::new_game(TatakuValue::Map(Default::default())));

        // // map is set in BeatmapManager
        // values.set("new_map", TatakuValue::None);

        // score values
        values.set("score", TatakuVariable::new_game(&Score::default()));
        // values.set("score.score", 0.0);
        // values.set("score.combo", 0.0);
        // values.set("score.max_combo", 0.0);
        // values.set("score.accuracy", 0.0);
        // values.set("score.performance", 0.0);
        // values.set("score.placing", 0);
        // values.set("score.health", 0.0);

    }

    pub async fn init(&mut self) {

        self.load_custom_menus();

        // init value collection
        self.init_value_collection();
        
        // init audio
        AudioManager::init_audio().expect("error initializing audio");

        let now = std::time::Instant::now();

        // online loop
        tokio::spawn(async move {
            loop {
                OnlineManager::start().await;
                tokio::time::sleep(Duration::from_millis(1_000)).await;
            }
        });

        Self::load_theme(&self.settings.theme);

        // set the current leaderboard filter
        // this is here so it happens before anything else
        let settings = SettingsHelper::new();
        self.last_skin = settings.current_skin.clone();

        // setup double tap protection
        self.input_manager.set_double_tap_protection(settings.enable_double_tap_protection.then(|| settings.double_tap_protection_duration));

        // beatmap manager loop
        self.actions.push(TaskAction::AddTask(Box::new(BeatmapDownloadsCheckTask::new())));
        // BeatmapManager::download_check_loop();

        // == menu setup ==
        let mut loading_menu = LoadingMenu::new().await;
        loading_menu.load().await;

        // // check git updates
        // self.add_dialog(Box::new(ChangelogDialog::new().await));

        // load background images
        match std::fs::read_dir("resources/wallpapers") {
            Ok(list) => {
                for wall_file in list {
                    if let Ok(file) = wall_file {
                        if let Some(wallpaper) = self.skin_manager.get_texture_noskin(file.path().to_str().unwrap(), false).await {
                            self.wallpapers.push(wallpaper)
                        }
                    }
                }
            }
            Err(_e) => {
                // NotificationManager::add_error_notification("Error loading wallpaper", e).await
            }
        }

        debug!("game init took {:.2}", now.elapsed().as_secs_f32() * 1000.0);

        self.queue_state_change(GameState::SetMenu(Box::new(loading_menu)));
    }
    
    pub async fn game_loop(mut self) {
        let mut update_timer = Instant::now();
        let mut draw_timer = Instant::now();
        let mut last_draw_offset = 0.0;

        let game_start = std::time::Instant::now();
        let mut last_setting_update = None;

        let mut display_settings = self.settings.display_settings.clone();
        let mut integrations = self.settings.integrations.clone();

        let mut render_rate   = 1.0 / display_settings.fps_target as f64;
        let mut update_target = 1.0 / display_settings.update_target as f64;



        loop {
            while let Ok(e) = self.game_event_receiver.try_recv() {
                match e {
                    Window2GameEvent::FileDrop(path) => self.handle_file_drop(path).await,
                    Window2GameEvent::Closed => { return self.close_game(); }
                    Window2GameEvent::ScreenshotComplete(bytes, size, info) => if let Err(e) = self.finish_screenshot(bytes, size, info).await {
                        self.actions.push(GameAction::AddNotification(Notification::new_error("Screenshot Error", e)));
                    }
                    e => self.input_manager.handle_events(e),
                }
            }

            
            // update our settings
            let last_master_vol = self.settings.master_vol;
            let last_music_vol = self.settings.music_vol;
            let last_effect_vol = self.settings.effect_vol;
            let last_theme = self.settings.theme.clone();
            let last_server_url = self.settings.server_url.clone();
            
            if self.settings.update() {

                if self.settings.display_settings != display_settings {
                    display_settings = self.settings.display_settings.clone();
                    render_rate = 1.0 / display_settings.fps_target as f64;
                    update_target = 1.0 / display_settings.update_target as f64;
                    self.window_proxy.send_event(Game2WindowEvent::SettingsUpdated(display_settings.clone())).unwrap();
                }

                let audio_changed = 
                    last_master_vol != self.settings.master_vol
                    || last_music_vol != self.settings.music_vol
                    || last_effect_vol != self.settings.effect_vol;

                // dont save when audio is changed, that would spam too hard
                if !audio_changed && !self.settings.skip_autosaveing {
                    // // save the settings
                    // self.settings.save().await;
                    last_setting_update = Some(Instant::now());
                }


                let skin_changed = self.settings.current_skin != self.last_skin;
                if skin_changed {
                    self.skin_manager.change_skin(self.settings.current_skin.clone()).await;
                    self.last_skin = self.settings.current_skin.clone();


                    for (i, _) in self.gameplay_managers.values_mut() {
                        i.reload_skin(&mut self.skin_manager).await;
                    }
                }

                if self.settings.theme != last_theme {
                    Self::load_theme(&self.settings.theme)
                }

                if self.settings.server_url != last_server_url {
                    OnlineManager::restart();
                }

                if integrations != self.settings.integrations {

                    // update discord
                    match (integrations.discord, self.settings.integrations.discord) {
                        (true, false) => OnlineManager::get_mut().await.discord = None,
                        (false, true) => OnlineManager::init_discord().await,
                        _ => {}
                    }

                    integrations = self.settings.integrations.clone();
                    self.window_proxy.send_event(Game2WindowEvent::IntegrationsChanged(integrations.clone())).unwrap();
                }


                // update doubletap protection
                self.input_manager.set_double_tap_protection(self.settings.enable_double_tap_protection.then(|| self.settings.double_tap_protection_duration));

                // update game mode with new information
                match &mut self.current_state {
                    GameState::Ingame(igm) => {
                        if skin_changed { igm.reload_skin(&mut self.skin_manager).await; }
                        igm.force_update_settings().await;
                    }
                    // GameState::Spectating(sm) => if let Some(igm) = &mut sm.game_manager { 
                    //     if skin_changed { igm.reload_skin().await; }
                    //     igm.force_update_settings().await;
                    // }
                    _ => {}
                }

                {
                    let values = &mut self.values;
                    // values.set("global.playmode", CurrentPlaymodeHelper::new().0.clone());
                    values.update("settings.sort_by", TatakuVariableWriteSource::Game, self.settings.last_sort_by);
                    // values.set("settings.sort_by", format!("{:?}", self.settings.last_group_by));
                }

                for (i, _) in self.gameplay_managers.values_mut() {
                    i.force_update_settings().await;
                }
            }

            // wait 100ms before writing settings changes
            if let Some(last_update) = last_setting_update {
                if last_update.as_millis() > 100.0 {
                    self.settings.save().await;
                    last_setting_update = None;
                }
            }

            // update our instant's time
            set_time(game_start.elapsed());
            let mut now = Instant::now();

            let update_elapsed = now.duration_since(update_timer).as_secs_f64();
            if update_elapsed >= update_target {
                update_timer = now;
                self.update().await;

                // // update values
                // self.value_checker.check(&mut self.values).await;
                
                // re-update the time
                set_time(game_start.elapsed());
                now = Instant::now();
            }

            if let GameState::Closing = &self.current_state {
                self.close_game();
                return;
            }

            
            #[cfg(feature="graphics")] {
                const DRAW_DAMPENING_FACTOR:f64 = 0.9;
                let elapsed = now.duration_since(draw_timer).as_secs_f64();
                if elapsed + last_draw_offset >= render_rate {
                    draw_timer = now;
                    last_draw_offset = (elapsed - render_rate).clamp(-5.0, 5.0) * DRAW_DAMPENING_FACTOR;
                    self.draw().await;
                }
            }

        }

    }

    /// use this for cleanup, not to tell the game to close
    /// to tell the game to close, set the state to GameState::Closing
    fn close_game(&mut self) {
        warn!("stopping game");
    }

    async fn update(
        &mut self
    ) {
        let elapsed = self.game_start.as_millis();
        self.values.update("game.time", TatakuVariableWriteSource::Game, elapsed);

        // check bg loaded
        if let Some(loader) = self.background_loader.clone() {
            if let Some(image) = loader.check().await {
                self.background_loader = None;

                // unload the old image so the atlas can reuse the space
                if let Some(old_img) = std::mem::take(&mut self.background_image) {
                    GameWindow::free_texture(old_img.tex);
                }

                self.background_image = image;
                
                if self.background_image.is_none() && self.wallpapers.len() > 0 {
                    self.background_image = Some(self.wallpapers[0].clone());
                }

                self.resize_bg();
            }
        }

        // check window size
        let window_size_updated = self.window_size.update();
        if window_size_updated { 
            self.resize_bg(); 
        }

        // let timer = Instant::now();
        self.update_display.increment();
        let mut current_state = std::mem::take(&mut self.current_state);

        // update counters
        self.fps_display.update().await;
        self.update_display.update().await;
        self.render_display.update().await;
        self.input_display.update().await;

        // read input events
        let mouse_pos = self.input_manager.mouse_pos;
        let mut mouse_down = self.input_manager.get_mouse_down();
        let mouse_up = self.input_manager.get_mouse_up();
        let mouse_moved = self.input_manager.get_mouse_moved();
        // TODO: do we want this here or only in menus?
        let mut scroll_delta = self.input_manager.get_scroll_delta() * self.settings.display_settings.scroll_sensitivity;

        let mut keys_down = self.input_manager.get_keys_down();
        let keys_up = self.input_manager.get_keys_up();
        let mods = self.input_manager.get_key_mods();
        let text = self.input_manager.get_text();
        let window_focus_changed = self.input_manager.get_changed_focus();

        let controller_down = self.input_manager.get_controller_down();
        let controller_up = self.input_manager.get_controller_up();
        let controller_axis = self.input_manager.get_controller_axis();
        
        // update the cursor
        self.cursor_manager.update(elapsed, self.input_manager.mouse_pos).await;
        
        // update cursor
        if mouse_down.contains(&MouseButton::Left) {
            self.cursor_manager.left_pressed(true);
        } else if mouse_up.contains(&MouseButton::Left) {
            self.cursor_manager.left_pressed(false);
        }
        if mouse_down.contains(&MouseButton::Right) {
            self.cursor_manager.right_pressed(true);
        } else if mouse_up.contains(&MouseButton::Right) {
            self.cursor_manager.right_pressed(false);
        }

        let controller_pause = controller_down.iter().find(|(_,a)|a.contains(&ControllerButton::Start)).is_some();

        // prevent the list from building up and just wasting memory.
        // not nuking the code because it might be a useful stat in the future
        let _register_timings = self.input_manager.get_register_delay();
        // if keys_up.len()+keys_down.len() > 0 {
        //     info!("register times: min:{:.2}, max: {:.2}, avg:{:.2}", _register_timings.0, _register_timings.1, _register_timings.2);
        // }

        if mouse_down.len() > 0 {
            // check notifs
            if NOTIFICATION_MANAGER.write().await.on_click(mouse_pos, self).await {
                mouse_down.clear();
            }
        }

        // check for volume change
        if mouse_moved { self.volume_controller.on_mouse_move(mouse_pos) }
        if scroll_delta != 0.0 {
            if let Some(action) = self.volume_controller.on_mouse_wheel(scroll_delta / (self.settings.display_settings.scroll_sensitivity * 1.5), mods).await { 
                scroll_delta = 0.0;
                self.actions.push(action);
            }
        } 
        self.volume_controller.on_key_press(&mut keys_down, mods).await;
        
        // check user panel
        if keys_down.has_and_remove(self.settings.key_user_panel) {
            self.ui_manager.application().handle_make_userpanel().await;
        }


        // screenshot
        if keys_down.has_and_remove(Key::F12) {
            self.window_proxy.send_event(Game2WindowEvent::TakeScreenshot(ScreenshotInfo {
                // if shift is pressed, upload to server, and get link
                upload: mods.shift,
            })).unwrap();
        }

        // if keys_down.contains(&Key::D1) && mods.ctrl {
        //     GlobalValueManager::update(Arc::new(CurrentTheme(tataku_theme())))
        // }
        // if keys_down.contains(&Key::D2) && mods.ctrl {
        //     GlobalValueManager::update(Arc::new(CurrentTheme(osu_theme())))
        // }

        // // direct downloads
        // if keys_down.contains(&Key::D) && mods.ctrl {
        //     self.queue_state_change(GameState::InMenu(Box::new(DirectMenu::new("osu".to_string()).await)));
        //     // self.add_dialog(Box::new(NotificationsDialog::new().await), false);
        // }
        // // direct downloads dialog
        // if keys_down.contains(&Key::J) && mods.ctrl {
        //     self.add_dialog(Box::new(DirectDownloadDialog::new()), false);
        // }

        // notfications menu
        if keys_down.has_key(Key::B) && mods.ctrl {
            keys_down.remove_key(Key::B);
            self.add_dialog(Box::new(NotificationsDialog::new().await), false);
        }

        // settings menu
        if keys_down.has_key(Key::O) && mods.ctrl {
            keys_down.remove_key(Key::O);
            let allow_ingame = self.settings.common_game_settings.allow_ingame_settings;
            let is_ingame = self.current_state.is_ingame();

            // im sure theres a way to do this in one statement (without the ||) but i'm tired so too bad
            if !is_ingame || (is_ingame && allow_ingame) {
                self.add_dialog(Box::new(SettingsMenu::new().await), false);
            }
        }

        // meme
        if keys_down.has_key(Key::PageUp) && mods.ctrl {
            keys_down.remove_key(Key::PageUp);
            debug!("{:#?}", self.values);
            // self.add_dialog(Box::new(DraggableDialog::new(Vector2::ZERO, Box::new(StupidDialog::new().await))), true);
        }

        // custom menu list
        if keys_down.has_key(Key::M) && mods.ctrl && mods.shift {
            keys_down.remove_key(Key::M);
            self.actions.push(MenuMenuAction::SetMenu("menu_list".to_owned()));
            // self.add_dialog(Box::new(DraggableDialog::new(Vector2::ZERO, Box::new(StupidDialog::new().await))), true);
        }
        if keys_down.has_key(Key::H) && mods.ctrl && mods.shift {
            keys_down.remove_key(Key::H);
            warn!("{:#?}", self.values);
        }


        // update any dialogs
        if keys_down.has_key(Key::Escape) && self.ui_manager.application().dialog_manager.close_latest().await {
            keys_down.remove_key(Key::Escape);
        }

        if keys_down.has_key(Key::F5) && mods.ctrl {
            keys_down.remove_key(Key::F5);
            NotificationManager::add_text_notification("Doing a full refresh, the game will freeze for a bit", 5000.0, Color::RED).await;
            self.beatmap_manager.full_refresh(&mut self.values).await;
            // tokio::spawn(async {
            //     BEATMAP_MANAGER.write().await.full_refresh().await;
            // });
        }



        // reload custom menus
        if keys_down.has_key(Key::R) && mods.ctrl {
            keys_down.remove_key(Key::R);
            self.load_custom_menus();
            if let MenuType::Custom(name) = self.ui_manager.get_menu() {
                debug!("Reloading current menu");
                self.handle_custom_menu(name).await;
            }
        }
        
        // update our global values
        {
            let values = &mut self.values;
            values.update("song.position", TatakuVariableWriteSource::Game, self.song_manager.position());

            if let Some(audio) = self.song_manager.instance() {
                let song_state = audio.get_state();
                if self.song_state != song_state {
                    self.song_state = song_state;

                    match self.song_state {
                        AudioState::Stopped | AudioState::Unknown => self.actions.push(GameAction::HandleEvent(TatakuEventType::SongEnd, None)),
                        AudioState::Playing => self.actions.push(GameAction::HandleEvent(TatakuEventType::SongStart, None)),
                        AudioState::Paused => self.actions.push(GameAction::HandleEvent(TatakuEventType::SongPause, None)),
                    }

                    values.update_multiple(TatakuVariableWriteSource::Game, [
                        ("song.playing", audio.is_playing()),
                        ("song.paused", audio.is_paused()),
                        ("song.stopped", audio.is_stopped()),
                    ].into_iter());
                }

            } else {
                values.update_multiple(TatakuVariableWriteSource::Game, [
                    ("song.playing", false),
                    ("song.paused", false),
                    ("song.stopped", false),
                ].into_iter());
            }
        }

        // update any ingame managers
        self.gameplay_managers.retain(|a, _| Arc::strong_count(a) > 1);
        for (manager, _config) in self.gameplay_managers.values_mut() {
            manager.update(&mut self.values).await;

            if manager.completed {
                manager.on_complete()
            }
        }


        // update the ui
        for key in keys_down.0.iter().filter_map(|i| i.as_key()) {
            self.queued_events.push((TatakuEventType::KeyPress(CustomMenuKeyEvent {
                key,
                control: mods.ctrl,
                alt: mods.alt,
                shift: mods.shift,
            }), None));
        }
        for key in keys_up.0.iter().filter_map(|i| i.as_key()) {
            self.queued_events.push((TatakuEventType::KeyRelease(CustomMenuKeyEvent {
                key,
                control: mods.ctrl,
                alt: mods.alt,
                shift: mods.shift,
            }), None));
        }

        let (mut menu_actions, sy_values) = self.ui_manager.update(
            CurrentInputState {
                mouse_pos,
                mouse_moved,
                scroll_delta,
                mouse_down: &mouse_down,
                mouse_up: &mouse_up,
                keys_down: &keys_down,
                keys_up: &keys_up,
                mods,
            }, 
            self.queued_events.take(),
            self.values.take()
        ).await;
        self.values = sy_values;

        // update spec and multi managers
        if let Some(spec) = &mut self.spectator_manager { 
            let manager = current_state.get_ingame();
            menu_actions.extend(spec.update(manager, &mut self.values).await);
        }
        if let Some(multi) = &mut self.multiplayer_manager { 
            let manager = current_state.get_ingame();
            menu_actions.extend(multi.update(manager, &mut self.values).await);
        }


        // update score manager
        self.score_manager.update(&mut self.values).await;

        // handle menu actions
        let game_state = TaskGameState {
            ingame: self.current_state.is_ingame(),
            game_time: self.game_start.as_millis() as u64,
        };
        menu_actions.extend(self.task_manager.update(&mut self.values, game_state).await);
        self.handle_actions(menu_actions).await;

        // run update on current state
        match current_state.take() {
            GameState::Ingame(mut manager) => {
                if window_size_updated {
                    manager.window_size_changed((*self.window_size).clone()).await;
                }

                // pause button, or focus lost, only if not replaying
                if let Some(got_focus) = window_focus_changed {
                    if self.settings.display_settings.pause_on_focus_lost {
                        manager.window_focus_lost(got_focus)
                    }
                }
                
                if !manager.failed && manager.can_pause() && (manager.should_pause || controller_pause) {
                    manager.pause();
                    let actions = manager.actions.take();
                    self.handle_actions(actions).await;

                    let menu = PauseMenu::new(manager, false).await;
                    self.queue_state_change(GameState::SetMenu(Box::new(menu)));
                } else {
                    // inputs
                    // mouse
                    if mouse_moved { manager.mouse_move(mouse_pos).await }
                    for btn in mouse_down { manager.mouse_down(btn).await }
                    for btn in mouse_up { manager.mouse_up(btn).await }
                    if scroll_delta != 0.0 { manager.mouse_scroll(scroll_delta).await }

                    // kb
                    for k in keys_down.0 { manager.key_down(k, mods).await }
                    for k in keys_up.0 { manager.key_up(k).await }
                    if text.len() > 0 { manager.on_text(&text, &mods).await }

                    // controller
                    for (c, buttons) in controller_down {
                        for b in buttons {
                            manager.controller_press(&c, b).await;
                        }
                    }
                    for (c, buttons) in controller_up {
                        for b in buttons {
                            manager.controller_release(&c, b).await;
                        }
                    }
                    for (c, b) in controller_axis {
                        manager.controller_axis(&c, b).await;
                    }


                    // update, then check if complete
                    let actions = manager.update(&mut self.values).await;
                    self.handle_actions(actions).await;
                    if manager.completed {
                        self.ingame_complete(manager).await;
                        // a menu is queued up, we dont need to reapply current_state
                    } else {
                        current_state = GameState::Ingame(manager);
                    }
                }
            }
            
            GameState::None => {
                // might be transitioning
                if self.transition.is_some() && elapsed - self.transition_timer > TRANSITION_TIME / 2.0 {
                    let trans = self.transition.take();
                    self.queue_state_change(trans.unwrap());
                    self.transition_timer = elapsed;
                }
            }

            other => current_state = other
        }
        
        // update game mode
        match &self.queued_state {
            // queued mode didnt change, set the unlocked's mode to the updated mode
            GameState::None => self.current_state = current_state,
            GameState::Closing => {
                self.settings.save().await;
                self.current_state = GameState::Closing;
                let _ = self.window_proxy.send_event(Game2WindowEvent::CloseGame);

                // send logoff
                OnlineManager::set_action(SetAction::Closing, None);
            }


            _ => {
                // force close all dialogs
                self.ui_manager.application().dialog_manager.force_close_all().await;

                // // handle cleaup of the old state
                // match &mut current_state {
                //     GameState::SetMenu(menu) => menu.on_change(false).await,
                //     // GameState::Spectating(spectator_manager) => spectator_manager.stop(),
                //     _ => {}
                // }

                match &mut self.queued_state {
                    GameState::Ingame(manager) => {
                        // reset the song position 
                        if let Some(song) = self.song_manager.instance() {
                            song.pause();
                            if !manager.started {
                                song.set_position(0.0);
                            }
                        }
                        manager.reload_skin(&mut self.skin_manager).await;
                        manager.start().await;


                        let m = manager.metadata.clone();
                        let start_time = manager.start_time;
                        self.set_background_beatmap().await;
                        let action;
                        if let Some(manager) = &self.spectator_manager {
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

                        OnlineManager::set_action(action, Some(m.mode.clone()));
                    }
                    GameState::SetMenu(_) => OnlineManager::set_action(SetAction::Idle, None),
                    
                    _ => {}
                }

                let mut do_transition = true;
                match &current_state {
                    GameState::None => do_transition = false,
                    GameState::SetMenu(menu) if menu.get_name() == "pause" => do_transition = false,
                    _ => {}
                }

                if do_transition {
                    // do a transition

                    let qm = std::mem::take(&mut self.queued_state);
                    self.transition = Some(qm);
                    self.transition_timer = elapsed;
                    self.transition_last = Some(current_state);
                    self.queued_state = GameState::None;
                    self.current_state = GameState::None;
                } else {
                    // old mode was none, or was pause menu, transition to new mode
                    std::mem::swap(&mut self.queued_state, &mut self.current_state);

                    // if let GameState::SetMenu(menu) = &mut self.current_state {
                    //     menu.on_change(true).await;
                    // }
                }
            }
        }

        // update the notification manager
        NOTIFICATION_MANAGER.write().await.update(self).await;

        let mut multi_packets = Vec::new();
        if let Some(mut manager) = OnlineManager::try_get_mut() {

            //TODO: not run this all the time
            if manager.logged_in && manager.user_id > 0 {
                self.values.update("global.user_id", TatakuVariableWriteSource::Game, manager.user_id);
            }


            for host_id in manager.spectator_info.spectate_pending.take() {
                trace!("Speccing {host_id}");
                manager.spectator_info.outgoing_frames.clear();
                manager.spectator_info.incoming_frames.insert(host_id, Vec::new());

                match self.spec_watch_action {
                    SpectatorWatchAction::FullMenu => {
                        // stop spectating everyone else
                        for other_host_id in manager.spectator_info.currently_spectating() {
                            if other_host_id == host_id { continue }
                            OnlineManager::stop_spectating(other_host_id);
                        }
                        let username = if let Some(u) = manager.users.get(&host_id) {
                            u.lock().await.username.clone()
                        } else {
                            "Host".to_owned()
                        };
                        self.spectator_manager = Some(Box::new(SpectatorManager::new(host_id, username).await));

                        // self.queue_state_change(GameState::Spectating(Box::new()));
                    },
                    _ => {}
                };
            }
            
            multi_packets = manager.multiplayer_packet_queue.take()
        }
        
        for packet in multi_packets {
            if let Err(e) = self.handle_multiplayer_packet(packet).await {
                error!("Error handling multiplayer packet: {e:?}");
            }
        }

        
        // let elapsed = timer.elapsed().as_secs_f32() * 1000.0;
        // if elapsed > 1.0 {warn!("update took a while: {elapsed}");}
    }


    #[cfg(feature="graphics")]
    async fn draw(&mut self) {
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
            self.window_size.0,
            Color::BLACK.alpha(self.settings.background_dim),
            None
        ));

        // draw cursor ripples
        self.cursor_manager.draw_ripples(&mut render_queue);

        // draw any gameplay managers
        for (manager, config) in self.gameplay_managers.values_mut() {
            let mut temp_render_queue = RenderableCollection::new();
            if config.draw_function.is_some() {
                std::mem::swap(&mut render_queue, &mut temp_render_queue);
            }

            manager.draw(&mut render_queue).await;

            if let Some(draw_action) = &config.draw_function {
                std::mem::swap(&mut render_queue, &mut temp_render_queue);
                
                let group = TransformGroup::from_collection(Vector2::ZERO, temp_render_queue);
                (draw_action)(group);
            }
        }
        

        // mode
        self.ui_manager.draw(&mut render_queue).await;
        match &mut self.current_state {
            GameState::Ingame(manager) => manager.draw(&mut render_queue).await,
            // GameState::InMenu(menu) => menu.draw(&mut render_queue).await,
            // GameState::Spectating(manager) => manager.draw(&mut render_queue).await,
            _ => {}
        }

        // transition
        if self.transition_timer > 0.0 && elapsed - self.transition_timer < TRANSITION_TIME {
            // probably transitioning

            // // draw old mode
            // match (&self.current_state, &mut self.transition_last) {
            //     // (GameState::None, Some(GameState::InMenu(menu))) => menu.draw(&mut render_queue).await,
            //     _ => {}
            // }
            
            // draw fade in rect
            let diff = elapsed - self.transition_timer;

            let mut alpha = diff / (TRANSITION_TIME / 2.0);
            if self.transition.is_none() {alpha = 1.0 - diff / TRANSITION_TIME}

            render_queue.push(Rectangle::new(
                Vector2::ZERO,
                self.window_size.0,
                Color::new(0.0, 0.0, 0.0, alpha),
                None
            ));

        }

        // // draw any dialogs
        // let mut dialog_list = std::mem::take(&mut self.dialogs);
        // for d in dialog_list.iter_mut() { //.rev() {
        //     d.draw(Vector2::ZERO, &mut render_queue).await;
        // }
        // self.dialogs = dialog_list;

        // draw fps's
        self.fps_display.draw(&mut render_queue);
        self.update_display.draw(&mut render_queue);
        self.render_display.draw(&mut render_queue);
        self.input_display.draw(&mut render_queue);

        // volume control
        self.volume_controller.draw(&mut render_queue).await;

        // draw the notification manager
        NOTIFICATION_MANAGER.read().await.draw(&mut render_queue);

        // draw cursor
        self.cursor_manager.draw(&mut render_queue);

        // toss the items to the window to render
        self.window_proxy.send_event(Game2WindowEvent::RenderData(render_queue.take())).unwrap();
        
        self.fps_display.increment();

        // let elapsed = timer.elapsed().as_secs_f32() * 1000.0;
        // if elapsed > 1000.0/144.0 {warn!("render took a while: {elapsed}")}
    }
    
    async fn handle_previous_menu(&mut self, current_menu: &str)  {
        let in_multi = self.multiplayer_manager.is_some();
        let in_spec = self.spectator_manager.is_some();

        if in_multi { return self.handle_custom_menu("lobby_menu").await } //self.queue_state_change(GameState::SetMenu(Box::new(LobbyMenu::new().await))) }
        if in_spec { return self.queue_state_change(GameState::SetMenu(Box::new(SpectatorMenu::new()))) }

        match current_menu {
            // score menu with no multi or spec is the beatmap select menu
            "score" => self.handle_custom_menu("beatmap_select").await, //self.queue_state_change(GameState::SetMenu(Box::new(BeatmapSelectMenu::new().await))),

            // beatmap menu with no multi or spec is the main menu
            "beatmap_select" => self.handle_custom_menu("main_menu").await, //self.queue_state_change(GameState::SetMenu(Box::new(MainMenu::new().await))),

            _ => { 
                error!("unhandled previous menu request for menu {current_menu}")
            }
        }
    }

    pub async fn handle_actions(&mut self, actions: Vec<TatakuAction>) {
        self.actions.extend(actions);
        for action in self.actions.take() {
            self.handle_action(action).await
        }
    }

    // this should never recurse, but we need this here because the compiler doesnt know that lol
    #[async_recursion::async_recursion]
    pub async fn handle_action(&mut self, action: impl Into<TatakuAction> + Send + 'static) {
        let action = action.into();
        debug!("performing action: {action:?}");
        match action {
            TatakuAction::None => return,
            
            // menu actions
            TatakuAction::Menu(MenuMenuAction::SetMenu(id)) => self.handle_custom_menu(id).await,

            TatakuAction::Menu(MenuMenuAction::PreviousMenu(current_menu)) => self.handle_previous_menu(current_menu).await,
            
            // TatakuAction::Menu(MenuMenuAction::AddDialog(dialog, allow_duplicates)) => self.add_dialog(dialog, allow_duplicates),
            TatakuAction::Menu(MenuMenuAction::AddDialogCustom(dialog, allow_duplicates)) => self.handle_custom_dialog(dialog, allow_duplicates).await,
            
            // beatmap actions
            TatakuAction::Beatmap(action) => {
                match action {
                    BeatmapAction::PlaySelected => {
                        let Ok(map_hash) = self.values.try_get::<Md5Hash>("map.hash") else { return };
                        let Ok(mode) = self.values.get_string("global.playmode") else { return };
                        // let Some(map) = self.beatmap_manager.get_by_hash(&map_hash) else { return };
                        let mods = self.values.try_get::<ModManager>("global.mods").unwrap_or_default();

                        // play the map
                        let Ok(map_path) = self.values.get_string("map.path") else { return };

                        match manager_from_playmode_path_hash(mode, map_path, map_hash, mods.clone()).await {
                            Ok(mut manager) => {
                                manager.handle_action(GameplayAction::ApplyMods(mods)).await;
                                self.queue_state_change(GameState::Ingame(Box::new(manager)))
                            }
                            Err(e) => NotificationManager::add_error_notification("Error loading beatmap", e).await
                        }
                    }

                    BeatmapAction::ConfirmSelected => {
                        // TODO: could we use this to send map requests from ingame to the spec host?

                        if let Some(multi) = &mut self.multiplayer_manager {
                            // go back to the lobby before any checks
                            // this way if for some reason something down below fails, the user is in the lobby and not stuck in limbo
                            self.actions.push(MenuMenuAction::SetMenu("lobby_menu".to_owned()));

                            if !multi.is_host() { return warn!("trying to set lobby beatmap while not the host ??") };
                            
                            let Ok(map_hash) = self.values.try_get::<Md5Hash>("map.hash") else { return warn!("no/bad map.hash") };
                            let Ok(playmode) = self.values.get_string("global.playmode") else { return warn!("no/bad global.playmode") };
                            let Some(map) = self.beatmap_manager.get_by_hash(&map_hash) else { return warn!("no map?") };

                            tokio::spawn(OnlineManager::update_lobby_beatmap(map, playmode));
                        } else {
                            // play map
                            self.handle_action(BeatmapAction::PlaySelected).await
                        }
                    }

                    BeatmapAction::Set(beatmap, options) => {
                        // self.beatmap_manager.set_current_beatmap(&mut self.values, &beatmap, options.use_preview_point, options.restart_song).await;
                        // warn!("setting beatmap: {}", beatmap.version_string());
                        self.handle_action(BeatmapAction::SetFromHash(beatmap.beatmap_hash, options)).await;
                    }
                    BeatmapAction::SetFromHash(hash, options) => {
                        if let Some(beatmap) = self.beatmap_manager.get_by_hash(&hash) {
                            self.beatmap_manager.set_current_beatmap(&mut self.values, &beatmap, options.use_preview_point, options.restart_song).await;
                        
                        } else if self.multiplayer_manager.is_some() {
                            // if we're in a multiplayer lobby, and the map doesnt exist, remove the map
                            // self.beatmap_manager.remove_current_beatmap(&mut self.values).await;
                            self.handle_action(BeatmapAction::Remove).await;
                        } else {
                            match options.if_none {
                                MapActionIfNone::ContinueCurrent => {},
                                MapActionIfNone::SetNone => self.handle_action(BeatmapAction::Remove).await, //self.beatmap_manager.remove_current_beatmap(&mut self.values).await,
                                MapActionIfNone::Random(preview) => {
                                    let Some(map) = self.beatmap_manager.random_beatmap() else { return };
                                    self.handle_action(BeatmapAction::SetFromHash(map.beatmap_hash, options.use_preview_point(preview))).await;
                                }
                            }
                        }

                    }
                    
                    BeatmapAction::SetPlaymode(new_mode) => self.update_playmode(new_mode).await,

                    BeatmapAction::Random(use_preview) => {
                        let Some(random) = self.beatmap_manager.random_beatmap() else { return };
                        self.beatmap_manager.set_current_beatmap(&mut self.values, &random, use_preview, true).await;
                    }
                    BeatmapAction::Remove => {
                        self.beatmap_manager.remove_current_beatmap(&mut self.values).await;
                        // warn!("removeing beatmap");
                        self.remove_background_beatmap().await;
                    }

                    BeatmapAction::Delete(hash) => {
                        self.beatmap_manager.delete_beatmap(hash, &mut self.values, PostDelete::Next).await;
                    }
                    BeatmapAction::DeleteCurrent(post_delete) => {
                        let Ok(map_hash) = self.values.try_get::<Md5Hash>("map.hash") else { return };

                        self.beatmap_manager.delete_beatmap(map_hash, &mut self.values, post_delete).await;
                    }
                    BeatmapAction::Next => {
                        self.beatmap_manager.next_beatmap(&mut self.values).await;
                    }
                    BeatmapAction::Previous(if_none) => {
                        if self.beatmap_manager.previous_beatmap(&mut self.values).await { return }
                        
                        // no previous map availble, handle accordingly
                        match if_none {
                            MapActionIfNone::ContinueCurrent => return,
                            MapActionIfNone::Random(use_preview) => {
                                let Some(random) = self.beatmap_manager.random_beatmap() else { return };
                                self.beatmap_manager.set_current_beatmap(&mut self.values, &random, use_preview, true).await;
                            }
                            MapActionIfNone::SetNone => self.beatmap_manager.remove_current_beatmap(&mut self.values).await,
                        }
                    }

                    BeatmapAction::InitializeManager => { 
                        self.beatmap_manager.initialize(&mut self.values).await; 
                    }
                    BeatmapAction::AddBeatmap { map, add_to_db } => {
                        self.beatmap_manager.add_beatmap(&map, add_to_db, &mut self.values).await;
                    }
                    

                    // beatmap list actions
                    BeatmapAction::ListAction(list_action) => {
                        match list_action {
                            BeatmapListAction::Refresh => self.beatmap_manager.refresh_maps(&mut self.values).await,
                            
                            BeatmapListAction::ApplyFilter { filter } => {
                                self.values.update("beatmap_list.search_text", TatakuVariableWriteSource::Game, filter.unwrap_or_default());
                                self.beatmap_manager.refresh_maps(&mut self.values).await;
                            }
                            BeatmapListAction::NextMap => self.beatmap_manager.next_map(&mut self.values),
                            BeatmapListAction::PrevMap => self.beatmap_manager.prev_map(&mut self.values),

                            BeatmapListAction::NextSet => self.beatmap_manager.next_set(&mut self.values),
                            BeatmapListAction::PrevSet => self.beatmap_manager.prev_set(&mut self.values),

                            BeatmapListAction::SelectSet(set_id) => self.beatmap_manager.select_set(set_id, &mut self.values),
                        }
                    }

                }

                // if self.value_checker.beatmap.check(&self.values) {
                //     let hash = self.values.try_get::<Md5Hash>("map.hash");
                //     if let Ok(_hash) = hash {
                //         self.set_background_beatmap().await;
                //     } else {
                //         // map was removed
                //         self.remove_background_beatmap().await;
                //     }
                // }

                // handle beatmap manager actions
                let bm_actions = self.beatmap_manager.actions.take();
                for i in bm_actions {
                    self.handle_action(i).await;
                }
            }

            
            // game actions
            TatakuAction::Game(GameAction::Quit) => self.queue_state_change(GameState::Closing),

            TatakuAction::Game(GameAction::ResumeMap(manager)) => {
                self.queue_state_change(GameState::Ingame(manager));
            }
            TatakuAction::Game(GameAction::StartGame(mut manager)) => {
                manager.start().await;
                self.queue_state_change(GameState::Ingame(manager));
            }
            TatakuAction::Game(GameAction::WatchReplay(replay)) => {
                let Some((map, mode)) = replay.score_data.as_ref().map(|s|(s.beatmap_hash, s.playmode.clone())) else {
                    NotificationManager::add_text_notification("Replay has no score data", 5000.0, Color::RED).await;
                    return;
                };

                let Some(beatmap) = self.beatmap_manager.get_by_hash(&map) else {
                    NotificationManager::add_text_notification("You don't have that map!", 5000.0, Color::RED).await;
                    return;
                };

                let mods = self.values.try_get::<ModManager>("global.mods").unwrap_or_default();
                
                match manager_from_playmode_path_hash(mode, beatmap.file_path.clone(), beatmap.beatmap_hash, mods).await {
                    Ok(mut manager) => {
                        manager.set_mode(GameplayMode::replay(*replay));
                        self.queue_state_change(GameState::Ingame(Box::new(manager)));
                    }
                    Err(e) => NotificationManager::add_error_notification("Error loading beatmap", e).await
                }
            }
            TatakuAction::Game(GameAction::SetValue(key, value)) => {
                self.values.update_or_insert(&key, TatakuVariableWriteSource::Menu, value, || TatakuVariable::new_any(TatakuValue::None));
                // self.values.try_insert(&key, || TatakuVariable::new(value, None, true, TatakuVariableAccess::Any));
                // self.values.update(&key, TatakuVariableWriteSource::Menu, value);
            }
            TatakuAction::Game(GameAction::ViewScore(score)) => {
                if let Some(beatmap) = self.beatmap_manager.get_by_hash(&score.beatmap_hash) {
                    let menu = ScoreMenu::new(&score, beatmap, false);
                    self.queue_state_change(GameState::SetMenu(Box::new(menu)))
                } else {
                    error!("Could not find map from score!")
                }
            }
            TatakuAction::Game(GameAction::HandleMessage(message)) => self.ui_manager.add_message(message),
            TatakuAction::Game(GameAction::RefreshScores) => self.score_manager.force_update = true,
            TatakuAction::Game(GameAction::ViewScoreId(id)) => {
                if let Some(score) = self.score_manager.get_score(id) {
                    self.handle_action(GameAction::ViewScore(score.clone())).await;
                }
            }
            TatakuAction::Game(GameAction::HandleEvent(event, param)) => self.queued_events.push((event, param)),
            TatakuAction::Game(GameAction::AddNotification(notif)) => NotificationManager::add_notification(notif).await,
            
            TatakuAction::Game(GameAction::UpdateBackground) => self.set_background_beatmap().await,
            TatakuAction::Game(GameAction::CopyToClipboard(text)) => { let _ = self.window_proxy.send_event(Game2WindowEvent::CopyToClipboard(text)); } 


            TatakuAction::Game(GameAction::NewGameplayManager(config)) => {
                match match &config {
                    NewManager { 
                        mods, 
                        map_hash: Some(map_hash), 
                        path: Some(path), 
                        playmode, 
                        ..
                    } => {
                        let playmode = playmode.clone().unwrap_or_else(|| self.values.get_string("global.playmode_actual").ok().unwrap_or_else(|| format!("osu")));
                        let mods = mods.clone().unwrap_or_else(|| self.values.try_get::<ModManager>("global.mods").unwrap_or_default());
                        manager_from_playmode_path_hash(playmode, path.clone(), *map_hash, mods).await
                    }
                    NewManager { 
                        mods, 
                        map_hash, 
                        playmode, 
                        ..
                    } => {
                        let map_hash = map_hash.unwrap_or_else(|| self.values.try_get::<Md5Hash>("map.hash").unwrap_or_default());
                        let Some(meta) = self.beatmap_manager.get_by_hash(&map_hash) else { return };
                        let playmode = playmode.clone().unwrap_or_else(|| self.values.get_string("global.playmode_actual").ok().unwrap_or_else(|| format!("osu")));
                        let mods = mods.clone().unwrap_or_else(|| self.values.try_get::<ModManager>("global.mods").unwrap_or_default());
                        manager_from_playmode(playmode, &meta, mods).await
                    }
                } {
                    Ok(mut manager) => {
                        manager.reload_skin(&mut self.skin_manager).await;
                        if let Some(mode) = config.gameplay_mode.clone() {
                            manager.set_mode(mode);
                        }
                        if let Some(bounds) = config.area {
                            manager.handle_action(GameplayAction::FitToArea(bounds)).await;
                        }
                        manager.reset().await;

                        let id = Arc::new(self.gameplay_managers.keys().max().map(|a| **a + 1).unwrap_or_default());
                        self.ui_manager.add_message(Message::new(
                            config.owner, "gameplay_manager_create", MessageType::GameplayManagerId(id.clone())
                        ));

                        self.gameplay_managers.insert(id.clone(), (manager, config));
                    }

                    Err(e) => error!("Error creating gameplay manager: {e}"),
                }
            }

            TatakuAction::Game(GameAction::DropGameplayManager(id)) => {
                self.gameplay_managers.remove(&id);
            }

            TatakuAction::Game(GameAction::GameplayAction(id, action)) => {
                let Some((gameplay, _)) = self.gameplay_managers.get_mut(&id) else { return };
                gameplay.handle_action(action).await;
            }
 

            // song actions
            TatakuAction::Song(song_action) => {
                match song_action {
                    // needs to be before trying to get the audio because audio might be none when this is run
                    SongAction::Set(action) => {
                        if let Err(e) = self.song_manager.handle_song_set_action(action) {
                            error!("Error handling SongMenuSetAction: {e:?}");
                        }
                    }

                    other => {
                        let Some(audio) = self.song_manager.instance() else { return }; 

                        match other {
                            SongAction::Play => audio.play(false),
                            SongAction::Restart => audio.play(true),
                            SongAction::Pause => audio.pause(),
                            SongAction::Stop => audio.stop(),
                            SongAction::Toggle if audio.is_playing() => audio.pause(),
                            SongAction::Toggle => audio.play(false),
                            SongAction::SeekBy(seek) => audio.set_position(audio.get_position() + seek),
                            SongAction::SetPosition(pos) => audio.set_position(pos),
                            SongAction::SetRate(rate) => audio.set_rate(rate),
                            SongAction::SetVolume(vol) => audio.set_volume(vol),
                            // handled above
                            SongAction::Set(_) => {}
                        }
                    }
                }

                // update discord presence
                // if let Some(song) = AudioManager::get_song().await {
                //     OnlineManager::set_action(SetAction::Listening { 
                //         artist: map.artist.clone(), 
                //         title: map.title.clone(),
                //         elapsed: song.get_position(),
                //         duration: song.get_duration()
                //     }, None);
                // }

                // update song state
                if let Some(audio) = self.song_manager.instance() {
                    self.values.update_multiple(TatakuVariableWriteSource::Game, [
                        ("song.exists", true),
                        ("song.playing", audio.is_playing()),
                        ("song.paused", audio.is_paused()),
                        ("song.stopped", audio.is_stopped()),
                    ].into_iter());
                } else {
                    self.values.update_multiple(TatakuVariableWriteSource::Game, [
                        ("song.exists", false),
                        ("song.playing", false),
                        ("song.paused", false),
                        ("song.stopped", false),
                    ].into_iter());
                }
            }

            // multiplayer actions
            TatakuAction::Multiplayer(MultiplayerAction::ExitMultiplayer) => {
                self.handle_action(MultiplayerAction::LeaveLobby).await;

                // TODO: check if ingame, and if yes, dont change state
                if !self.current_state.is_ingame() {
                    self.handle_custom_menu("main_menu").await;
                    // self.queue_state_change(GameState::SetMenu(Box::new(MainMenu::new().await)));
                }
            }
            TatakuAction::Multiplayer(MultiplayerAction::StartMultiplayer) => {
                // TODO: move to custom menu
                self.handle_custom_menu("lobby_select").await;
                // self.queue_state_change(GameState::SetMenu(Box::new(LobbyMenu::new().await)));
            }
            TatakuAction::Multiplayer(MultiplayerAction::CreateLobby { name, password, private, players }) => {
                self.multiplayer_data.lobby_creation_pending = true;

                OnlineManager::send_packet_static(MultiplayerPacket::Client_CreateLobby { name, password, private, players });
            }
            TatakuAction::Multiplayer(MultiplayerAction::LeaveLobby) => {
                self.multiplayer_manager = None;
                OnlineManager::send_packet_static(MultiplayerPacket::Client_LeaveLobby);
                self.handle_action(MenuMenuAction::SetMenu("lobby_select".to_owned())).await;
            }
            TatakuAction::Multiplayer(MultiplayerAction::JoinLobby { lobby_id, password }) => {
                self.multiplayer_data.lobby_join_pending = true;
                if let Some(multi_manager) = &mut self.multiplayer_manager {
                    // if we're already in this lobby, dont do anything
                    if multi_manager.lobby.id == lobby_id { return }

                    // otherwise, leave our current lobby
                    self.handle_action(MultiplayerAction::LeaveLobby).await;
                }

                OnlineManager::send_packet_static(MultiplayerPacket::Client_JoinLobby { lobby_id, password });
            }

            TatakuAction::Multiplayer(MultiplayerAction::SetBeatmap { hash, mode }) => {
                let Some(map) = self.beatmap_manager.get_by_hash(&hash) else { return };
                let mode = mode.unwrap_or_default();
                tokio::spawn(OnlineManager::update_lobby_beatmap(map, mode));
            }

            TatakuAction::Multiplayer(MultiplayerAction::InviteUser { user_id }) => {
                tokio::spawn(OnlineManager::invite_user(user_id));
            }

            // lobby actions
            TatakuAction::Multiplayer(MultiplayerAction::LobbyAction(LobbyAction::Leave)) => {
                self.handle_action(MultiplayerAction::LeaveLobby).await;
            }
            TatakuAction::Multiplayer(MultiplayerAction::LobbyAction(action)) => {
                let Some(multi_manager) = &mut self.multiplayer_manager else { return };
                multi_manager.handle_lobby_action(action).await;
            }


            // task actions
            TatakuAction::Task(TaskAction::AddTask(task)) => self.task_manager.add_task(task),
            
            // UI operation
            TatakuAction::PerformOperation(op) => self.ui_manager.add_operation(op),
        }
    }

    async fn handle_custom_menu(&mut self, id: impl ToString) {

        // let menu = self.custom_menus.iter().rev().find(|cm| cm.id == id);
        if let Some(menu) = self.custom_menu_manager.get_menu((id.to_string(), CustomMenuSource::Any)) {
            let menu = Box::new(menu.build(&mut self.skin_manager).await);
            self.queue_state_change(GameState::SetMenu(menu));
        } else {
            let id = id.to_string();
            match &*id {
                "none" => {}
                "main_menu" => panic!("Main menu could not be loaded. did eve fuck up the main_menu.lua?"),
                // "beatmap_select" => self.queue_state_change(GameState::SetMenu(Box::new(BeatmapSelectMenu::new().await))),
                _ => {
                    error!("custom menu not found! {id}, going to main menu instead");
                    self.actions.push(MenuMenuAction::SetMenu(format!("main_menu")));
                }
            }
        }
    }

    async fn handle_custom_dialog(&mut self, id: String, _allow_duplicates: bool) {
        match &*id {
            "settings" => self.add_dialog(Box::new(SettingsMenu::new().await), false),
            "create_lobby" => self.add_dialog(Box::new(CreateLobbyDialog::new()), false),
            "mods" => {
                let mut groups = Vec::new();
                let playmode = self.values
                    .get_string("global.playmode_actual")
                    .unwrap_or_else(|_| format!("osu"));

                if let Some(info) = get_gamemode_info(&playmode) {
                    groups = info.get_mods();
                }

                self.add_dialog(Box::new(ModDialog::new(groups).await), false);
            }

            _ => error!("unknown dialog id: {id}"),
        }
    }

    pub fn queue_state_change(&mut self, state:GameState) { 
        match state {
            GameState::SetMenu(menu) => {
                self.queued_state = GameState::InMenu(MenuType::from_menu(&menu));
                debug!("Changing menu to: {} ({:?})", menu.get_name(), menu.get_custom_name());
                self.ui_manager.set_menu(menu);
                self.queued_events.push((TatakuEventType::MenuEnter, None));
            }
            GameState::InMenu(_) => {}
            state => {
                // set the menu to an empty menu, hiding it
                self.ui_manager.set_menu(Box::new(EmptyMenu::new()));
                self.queued_state = state;
            }
        }
    }

    /// shortcut for setting the game's background texture to a beatmap's image
    pub async fn set_background_beatmap(&mut self) {
        let Ok(filename) = self.values.get_string("map.image_filename") else { return };
        // let f = self.skin_manager.get_texture_noskin(&filename, false);
        // self.background_loader = Some(AsyncLoader::new(f));
        self.background_image = self.skin_manager.get_texture_noskin(&filename, false).await;
        self.background_image.ok_do_mut(|i| {
            i.origin = Vector2::ZERO;
        });

        self.resize_bg();
    }
    /// shortcut for removing the game's background texture
    pub async fn remove_background_beatmap(&mut self) {
        self.background_image = None;
    }

    fn resize_bg(&mut self) {   
        let Some(bg) = &mut self.background_image else { return };
        bg.fit_to_bg_size(self.window_size.0, false);
    }

    pub fn add_dialog(&mut self, dialog: Box<dyn Dialog>, allow_duplicates: bool) {
        let dialog_manager = &mut self.ui_manager.application().dialog_manager;

        if !allow_duplicates {
            // check if said dialog already exists, if so, dont add it
            let name = dialog.name();
            if dialog_manager.dialogs.iter().find(|n|n.name() == name).is_some() { return }
        }

        debug!("adding dialog: {}", dialog.name());
        dialog_manager.add_dialog(dialog)
    }

    pub async fn handle_file_drop(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();

        if let Some(ext) = path.extension() {
            let ext = ext.to_str().unwrap();
            match *&ext {
                // osu | quaver | ptyping zipped set file
                "osz" | "qp" | "ptm" => {
                    match Zip::extract_single(path.to_path_buf(), SONGS_DIR, true, ArchiveDelete::Always).await {
                        Err(e) => NotificationManager::add_error_notification("Error extracting file",  e).await,
                        Ok(path) => {
                            // load the map
                            let Some(last) = self.beatmap_manager.check_folder(path, HandleDatabase::YesAndReturnNewMaps, &mut self.values).await.and_then(|l|l.last().cloned()) else { warn!("didnt get any beatmaps from beatmap file drop"); return };
                            // set it as current map if wanted
                            let mut use_preview_time = true;
                            let change_map = match &self.current_state {
                                GameState::SetMenu(menu) => {
                                    if menu.get_name() == "main_menu" { use_preview_time = false; }
                                    true
                                }
                                _ => false,
                            };
                            if change_map {
                                self.beatmap_manager.set_current_beatmap(&mut self.values, &last, use_preview_time, false).await;
                            }
                        }
                    }
                }

                // osu skin file
                "osk" => {
                    match Zip::extract_single(path.to_path_buf(), SKINS_FOLDER, true, ArchiveDelete::Never).await {
                        Err(e) => NotificationManager::add_error_notification("Error extracting file",  e).await,
                        Ok(path) => {
                            // set as current skin
                            if let Some(folder) = Path::new(&path).file_name() {
                                let name = folder.to_string_lossy().to_string();
                                Settings::get_mut().current_skin = name.clone();
                                NotificationManager::add_text_notification(format!("Added skin {name}"), 5000.0, Color::BLUE).await
                            }
                        }
                    }
                }

                // tataku | osu replay
                "ttkr" | "osr" => {
                    match read_replay_path(path).await {
                        Ok(replay) => self.try_open_replay(replay).await,
                        Err(e) => NotificationManager::add_error_notification("Error opening replay", e).await,
                    }
                }

                _ => {
                    NotificationManager::add_text_notification(
                        &format!("What is this?"), 
                        3_000.0, 
                        Color::RED
                    ).await;
                }
            }
        }
    }

    pub async fn try_open_replay(&mut self, replay: Replay) {
        let Some(score) = &replay.score_data else {
            NotificationManager::add_text_notification("Replay does not contain score data (too old?)", 5_000.0, Color::RED).await;
            return;
        };

        let Some(map) = self.beatmap_manager.get_by_hash(&score.beatmap_hash) else {
            NotificationManager::add_text_notification("You don't have this beatmap!", 5_000.0, Color::RED).await;
            return;
        };

        self.beatmap_manager.set_current_beatmap(&mut self.values, &map, true, true).await;

        // move to a score menu with this as the score
        let score = IngameScore::new(score.clone(), false, false);
        let mut menu = ScoreMenu::new(&score, map, false);
        menu.replay = Some(replay);
        self.queued_state = GameState::SetMenu(Box::new(menu));
    }


    pub async fn ingame_complete(&mut self, mut manager: Box<GameplayManager>) {
        trace!("beatmap complete");
        manager.on_complete();
        manager.score.time = chrono::Utc::now().timestamp() as u64;

        if manager.failed {
            trace!("player failed");
            if !manager.get_mode().is_multi() {
                self.queue_state_change(GameState::SetMenu(Box::new(PauseMenu::new(manager, true).await)));
            }

        } else {
            let mut score = manager.score.clone();
            score.accuracy = get_gamemode_info(&score.playmode).unwrap().calc_acc(&score);

            let mut replay = manager.replay.clone();
            replay.score_data = Some(score.score.clone());


            let mut score_submit = None;
            if manager.should_save_score() {
                // save score
                Database::save_score(&score).await;
                match save_replay(&replay, &score) {
                    Ok(_)=> trace!("replay saved ok"),
                    Err(e) => NotificationManager::add_error_notification("error saving replay", e).await,
                }

                let Some(map) = self.beatmap_manager.get_by_hash(&score.beatmap_hash) else { return warn!("no map ???") };

                // submit score
                let submit = ScoreSubmitHelper::new(
                    replay.clone(), 
                    &self.settings,
                    &map
                );

                submit.clone().submit();
                score_submit = Some(submit);
            }

            match manager.get_mode() {
                // go back to beatmap select
                GameplayMode::Replaying {..} => {
                    self.handle_custom_menu("beatmap_select").await;
                    // let menu = BeatmapSelectMenu::new().await; 
                    // self.queue_state_change(GameState::SetMenu(Box::new(menu)));
                }
                GameplayMode::Multiplayer { .. } => {}

                _ => {
                    // show score menu
                    let mut menu = ScoreMenu::new(&score, manager.metadata.clone(), true);
                    menu.replay = Some(replay.clone());
                    menu.score_submit = score_submit;
                    self.queue_state_change(GameState::SetMenu(Box::new(menu)));
                }
            }
        }
    }


    fn load_theme(theme: &SelectedTheme) {
        let theme = match &theme {
            SelectedTheme::Tataku => tataku_theme(),
            SelectedTheme::Osu => osu_theme(),
            SelectedTheme::Custom(path, _) => Io::read_file(path).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default(),
        };

        GlobalValueManager::update(Arc::new(CurrentTheme(theme)));
    }


    async fn finish_screenshot(&mut self, bytes: Vec<u8>, [width, height]: [u32; 2], info: ScreenshotInfo) -> TatakuResult {
        
        // create file
        let date = chrono::Local::now();
        let year = date.year();
        let month = date.month();
        let day = date.day();
        let hour = date.hour();
        let minute = date.minute();
        let second = date.second();

        let file = format!("../Screenshots/{year}-{month}-{day}--{hour}-{minute}-{second}.png");
        let path = Path::new(&file);

        std::fs::create_dir_all(path.parent().unwrap())?;
        let file = std::fs::File::create(path)?;

        // save as png
        let w = &mut std::io::BufWriter::new(file);
        let mut encoder = png::Encoder::new(w, width, height);
        encoder.set_color(png::ColorType::Rgba);

        let mut writer = encoder.write_header().map_err(|e| TatakuError::String(format!("{e}")))?;
        writer.write_image_data(&bytes).map_err(|e| TatakuError::String(format!("{e}")))?;

        // notify user
        let full_path = std::env::current_dir().unwrap().join(path).to_string_lossy().to_string();
        self.actions.push(GameAction::AddNotification(Notification::new(
            format!("Screenshot saved to {full_path}"), 
            Color::BLUE, 
            5000.0, 
            NotificationOnClick::File(full_path.clone())
        )));

        if info.upload {
            self.task_manager.add_task(Box::new(UploadScreenshotTask::new(full_path)));
        }

        Ok(())
    }


    async fn handle_multiplayer_packet(&mut self, packet: MultiplayerPacket) -> TatakuResult {
        // if we have a multi manager, pass the packet onto it as well
        if let Some(multi_manager) = &mut self.multiplayer_manager {
            let ig_manager = self.current_state.get_ingame();
            multi_manager.handle_packet(&mut self.values, &packet, ig_manager).await?;
        }

        match packet {
            MultiplayerPacket::Server_LobbyList { lobbies } => {
                self.multiplayer_data.lobbies = lobbies.into_iter().map(|l|(l.id, l)).collect();
                // TODO: update our values
            }

            MultiplayerPacket::Server_CreateLobby { success, lobby } => {
                let Some(lobby) = lobby.filter(|_| success) else { warn!("no success or lobby"); return Ok(()) };
                if !self.multiplayer_data.lobby_creation_pending { warn!("no join pending"); return Ok(()) }
                let Ok(our_id) = self.values.get_u32("global.user_id") else { warn!("no global.user_id"); return Ok(()) };
                if our_id == 0 { warn!("user_id == 0"); return Ok(()) }


                let mut info = CurrentLobbyInfo::new(lobby, our_id);
                info.update_usernames().await;

                let manager = MultiplayerManager::new(info);
                manager.update_values(&mut self.values);
                self.multiplayer_manager = Some(Box::new(manager));
                self.handle_custom_menu("lobby_menu").await;


                // try to update the server with our current map and mode
                let Ok(map_hash) = self.values.try_get("map.hash") else { return Ok(()) };
                let Some(map) = self.beatmap_manager.get_by_hash(&map_hash) else { return Ok(()) };

                let Ok(mode) = self.values.get_string("global.playmode") else { return Ok(()) };
                OnlineManager::update_lobby_beatmap(map, mode).await;
            }
            MultiplayerPacket::Server_JoinLobby { success, lobby } => {
                let Some(lobby) = lobby.filter(|_| success) else { return Ok(()) };
                if !self.multiplayer_data.lobby_join_pending { return Ok(()) }
                let Ok(our_id) = self.values.get_u32("global.user_id") else { return Ok(()) };
                if our_id == 0 { return Ok (()) }

                let mut info = CurrentLobbyInfo::new(lobby, our_id);
                info.update_usernames().await;
                
                let manager = MultiplayerManager::new(info);
                manager.update_values(&mut self.values);
                self.multiplayer_manager = Some(Box::new(manager));
                self.handle_custom_menu("lobby_menu").await;
            }


            MultiplayerPacket::Server_LobbyCreated { lobby } => {
                self.multiplayer_data.lobbies.insert(lobby.id, lobby.clone());
            }
            MultiplayerPacket::Server_LobbyDeleted { lobby_id } => {
                self.multiplayer_data.lobbies.remove(&lobby_id);
            }
            
            MultiplayerPacket::Server_LobbyUserJoined { lobby_id, user_id } => {
                self.multiplayer_data.lobbies.get_mut(&lobby_id)
                    .ok_do_mut(|l| l.players.push(user_id));
            }
            
            MultiplayerPacket::Server_LobbyUserLeft { lobby_id, user_id } => {
                self.multiplayer_data.lobbies.get_mut(&lobby_id).map(|l| l.players.retain(|u|u != &user_id));
            
                if let Some(manager) = &self.multiplayer_manager {
                    if manager.lobby.our_user_id == user_id {
                        self.multiplayer_manager = None;
                        NotificationManager::add_text_notification("You have been kicked from the match", 3000.0, Color::PURPLE).await;
                    }
                }
            }

            
            MultiplayerPacket::Server_LobbyMapChange { lobby_id, new_map } => {
                self.multiplayer_data.lobbies.get_mut(&lobby_id).ok_do_mut(|l| l.current_beatmap = Some(new_map.title.clone()));
            }

            MultiplayerPacket::Server_LobbyStateChange { lobby_id, new_state } => {
                self.multiplayer_data.lobbies.get_mut(&lobby_id).ok_do_mut(|l| l.state = new_state);
            }
            
            MultiplayerPacket::Server_LobbyInvite { inviter_id, lobby } => {
                self.multiplayer_data.lobbies.get_mut(&lobby.id).ok_do_mut(|l| l.has_password = false);
                
                let Some(inviter) = OnlineManager::get().await.users.get(&inviter_id).cloned() else { return Ok(()) };
                let inviter = inviter.lock().await;
                let text = format!("{} has invited you to a multiplayer match", inviter.username);

                let notif = Notification::new(text, Color::PURPLE_AMETHYST, 10_000.0, NotificationOnClick::MultiplayerLobby(lobby.id));
                NotificationManager::add_notification(notif).await;
            }

            _ => {}
        }

        Ok(())
    }


    async fn update_playmode(&mut self, playmode: String) {

        // ensure lowercase
        let mut playmode = playmode.to_lowercase();
        // warn!("setting playmode: {new_mode}");

        // ensure playmode exists
        if !AVAILABLE_PLAYMODES.contains(&&*playmode) { return warn!("Trying to set invalid playmode: {playmode}") }

        // set playmode and playmode display
        let Some(mut info) = get_gamemode_info(&playmode) else { return };
        self.values.update_display("global.playmode", TatakuVariableWriteSource::Game, &playmode, Some(info.display_name()));

        // if we have a beatmap, get the override mode and update the playmode_actual values
        if let Some(map) = &self.beatmap_manager.current_beatmap {
            playmode = map.check_mode_override(playmode);
            let Some(info2) = get_gamemode_info(&playmode) else { return };
            info = info2;
        }
        
        self.values.update_display("global.playmode_actual", TatakuVariableWriteSource::Game, &playmode, Some(info.display_name()));
    

        // update mods list as well
    
    }
}


#[derive(Default)]
pub enum GameState {
    #[default]
    None, // use this as the inital game mode, but be sure to change it after
    Closing,
    Ingame(Box<GameplayManager>),
    /// need to transition to the provided menu
    SetMenu(Box<dyn AsyncMenu>),
    /// Currently in a menu (this doesnt actually work currently, but it doesnt really matter)
    InMenu(MenuType),
}
impl GameState {
    /// spec_check means if we're spectator, check the inner game
    fn is_ingame(&self) -> bool {
        let Self::Ingame(_) = self else { return false };
        true
        // match self {
        //     Self::Ingame(_) => true,
        //     // Self::Spectating(s) if spec_check => s.game_manager.is_some(),
        //     Self::SetMenu(menu) if menu.get_name() == "multi_lobby" && multi_check => {false},
            
        //     _ => false
        // }
    }
    // fn to_string(&self) -> String {
    //     match self {
    //         Self::None => "None".to_owned(),
    //         Self::Closing => "Closing".to_owned(),
    //         Self::Ingame(_) => "Ingame".to_owned(),
    //         Self::SetMenu(m) => format!("Set Menu: {}", m.get_name()),
    //         Self::InMenu(m) => format!("In Menu: {m:?}")
    //     }
    // }

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

#[derive(Clone, Debug)]
pub enum MenuType {
    Internal(&'static str),
    Custom(String)
}
impl MenuType {
    pub fn from_menu(menu: &Box<dyn AsyncMenu>) -> Self {
        let Some(custom) = menu.get_custom_name() else { return Self::Internal(menu.get_name()) };
        Self::Custom(custom.clone())
    }
}

