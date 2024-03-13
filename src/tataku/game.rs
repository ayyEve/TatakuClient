use crate::prelude::*;
use chrono::{ Datelike, Timelike };

/// how long transitions between states should last
const TRANSITION_TIME:f32 = 500.0;

pub struct Game {
    // engine things
    input_manager: InputManager,
    volume_controller: VolumeControl,
    pub current_state: GameState,
    queued_state: GameState,
    game_event_receiver: tokio::sync::mpsc::Receiver<Window2GameEvent>,
    render_queue_sender: TripleBufferSender<RenderData>,

    /// if some, will handle spectator stuff
    spectator_manager: Option<Box<SpectatorManager>>,
    multiplayer_manager: Option<Box<MultiplayerManager>>,

    // pub dialogs: Vec<Box<dyn Dialog>>,

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

    custom_menus: Vec<CustomMenu>,

    pub shunting_yard_values: ShuntingYardValues,
    pub song_manager: SongManager,
}
impl Game {
    pub async fn new(render_queue_sender: TripleBufferSender<RenderData>, game_event_receiver: tokio::sync::mpsc::Receiver<Window2GameEvent>) -> Game {
        GlobalValueManager::update(Arc::new(CurrentBeatmap::default()));
        GlobalValueManager::update(Arc::new(CurrentPlaymode("osu".to_owned())));
        GlobalValueManager::update::<DirectDownloadQueue>(Arc::new(Vec::new()));

        let mut g = Game {
            // engine
            input_manager: InputManager::new(),
            volume_controller: VolumeControl::new().await,
            // dialogs: Vec::new(),
            background_image: None,
            wallpapers: Vec::new(),
            settings: SettingsHelper::new(),
            window_size: WindowSizeHelper::new(),
            spectator_manager: None,
            multiplayer_manager: None,

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
            render_queue_sender,
            cursor_manager: CursorManager::new().await,
            last_skin: String::new(),
            background_loader: None,

            ui_manager: UiManager::new(),
            custom_menus: Vec::new(),

            shunting_yard_values: ShuntingYardValues::new(),

            song_manager: SongManager::new(),
        };
        g.load_custom_menus();

        g.init().await;

        g
    }

    fn load_custom_menus(&mut self) {
        if !self.custom_menus.is_empty() {
            debug!("Reloading custom menus")
        }

        let mut parser = CustomMenuParser::new();
        parser.load("../custom_menus/main_menu.lua").unwrap();
        parser.load("../custom_menus/beatmap_select_menu.lua").unwrap();
        self.custom_menus = parser.get_menus();

        debug!("Done loading custom menus");
    }

    /// initialize all the values in our value collection
    /// doubles as a list of available values because i know i'm going to forget to put them in the doc at some point
    fn init_value_collection(&mut self) {
        let values = &mut self.shunting_yard_values;

        // game values
        values.set("game.time", 0.0);

        // song values
        values.set("song.exists", false);
        values.set("song.playing", false);
        values.set("song.paused", false);
        values.set("song.stopped", false);
        values.set("song.position", 0.0);

        // map values
        values.set("map.artist", String::new());
        values.set("map.title", String::new());
        values.set("map.creator", String::new());
        values.set("map.version", String::new());
        values.set("map.playmode", String::new());
        values.set("map.game", String::new());
        values.set("map.diff_rating", 0.0);
        values.set("map.hash", String::new());
        values.set("map.audio_path", String::new());
        values.set("map.preview_time", 0.0);

        // score values
        values.set("score.score", 0.0);
        values.set("score.combo", 0.0);
        values.set("score.max_combo", 0.0);
        values.set("score.accuracy", 0.0);
        values.set("score.performance", 0.0);
        values.set("score.placing", 0);
        values.set("score.health", 0.0);

    }

    pub async fn init(&mut self) {

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

        // make sure we have a value in the mod manager global store
        GlobalValueManager::update(Arc::new(ModManager::new()));
        GlobalValueManager::update(Arc::new(LatestBeatmap(Default::default())));

        Self::load_theme(&self.settings.theme);

        // set the current leaderboard filter
        // this is here so it happens before anything else
        let settings = SettingsHelper::new();
        SCORE_HELPER.write().await.current_method = settings.last_score_retreival_method;
        self.last_skin = settings.current_skin.clone();

        // setup double tap protection
        self.input_manager.set_double_tap_protection(settings.enable_double_tap_protection.then(|| settings.double_tap_protection_duration));

        // beatmap manager loop
        BeatmapManager::download_check_loop();

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
                        if let Some(wallpaper) = load_image(file.path().to_str().unwrap(), false, Vector2::ONE).await {
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


        let mut last_render_target = self.settings.fps_target as f64;
        let mut last_update_target = self.settings.update_target as f64;

        let mut render_rate   = 1.0 / last_render_target;
        let mut update_target = 1.0 / last_update_target;

        loop {
            while let Ok(e) = self.game_event_receiver.try_recv() {
                match e {
                    Window2GameEvent::FileDrop(path) => self.handle_file_drop(path).await,
                    Window2GameEvent::Closed => { return self.close_game(); }
                    e => self.input_manager.handle_events(e),
                }
            }

            
            // update our settings
            let last_master_vol = self.settings.master_vol;
            let last_music_vol = self.settings.music_vol;
            let last_effect_vol = self.settings.effect_vol;
            let last_theme = self.settings.theme.clone();
            let last_server_url = self.settings.server_url.clone();
            let last_discord_enabled = self.settings.integrations.discord;
            
            if self.settings.update() {
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

                if self.settings.fps_target as f64 != last_render_target {
                    last_render_target = self.settings.fps_target as f64;
                    render_rate = 1.0 / last_render_target;
                }
                if self.settings.update_target as f64 != update_target {
                    last_update_target = self.settings.update_target as f64;
                    update_target = 1.0 / last_update_target;
                }

                let skin_changed = self.settings.current_skin != self.last_skin;
                if skin_changed {
                    SkinManager::change_skin(self.settings.current_skin.clone()).await;
                    self.last_skin = self.settings.current_skin.clone();
                }

                if self.settings.theme != last_theme {
                    Self::load_theme(&self.settings.theme)
                }

                if self.settings.server_url != last_server_url {
                    OnlineManager::restart();
                }

                // update discord
                match (last_discord_enabled, self.settings.integrations.discord) {
                    (true, false) => OnlineManager::get_mut().await.discord = None,
                    (false, true) => OnlineManager::init_discord().await,
                    _ => {}
                }


                // update doubletap protection
                self.input_manager.set_double_tap_protection(self.settings.enable_double_tap_protection.then(|| self.settings.double_tap_protection_duration));

                // update game mode with new information
                match &mut self.current_state {
                    GameState::Ingame(igm) => {
                        if skin_changed { igm.reload_skin().await; }
                        igm.force_update_settings().await;
                    }
                    // GameState::Spectating(sm) => if let Some(igm) = &mut sm.game_manager { 
                    //     if skin_changed { igm.reload_skin().await; }
                    //     igm.force_update_settings().await;
                    // }
                    _ => {}
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

    pub fn close_game(&mut self) {
        warn!("stopping game");
    }

    async fn update(&mut self) {
        let elapsed = self.game_start.as_millis();
        self.shunting_yard_values.set("game.time", elapsed);

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
        let mut scroll_delta = self.input_manager.get_scroll_delta() * self.settings.scroll_sensitivity;

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
            if let Some(action) = self.volume_controller.on_mouse_wheel(scroll_delta / (self.settings.scroll_sensitivity * 1.5), mods).await { 
                scroll_delta = 0.0;
                self.handle_menu_actions(vec![action.into()]).await;
            }
        } 
        self.volume_controller.on_key_press(&mut keys_down, mods).await;
        
        // check user panel
        if keys_down.contains(&self.settings.key_user_panel) {
            self.ui_manager.application().handle_make_userpanel().await;
        }

        // screenshot
        if keys_down.contains(&Key::F12) {
            let (f, b) = tokio::sync::oneshot::channel();
            GameWindow::send_event(Game2WindowEvent::TakeScreenshot(f));

            tokio::spawn(async move {
                if let Err(e) = Self::await_screenshot(b, mods).await {
                    NotificationManager::add_error_notification("Error saving screenshot", e).await;
                }
            });

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
        if keys_down.contains(&Key::B) && mods.ctrl {
            self.add_dialog(Box::new(NotificationsDialog::new().await), false);
        }

        // settings menu
        if keys_down.contains(&Key::O) && mods.ctrl {
            let allow_ingame = self.settings.common_game_settings.allow_ingame_settings;
            let is_ingame = self.current_state.is_ingame(false, true);

            // im sure theres a way to do this in one statement (without the ||) but i'm tired so too bad
            if !is_ingame || (is_ingame && allow_ingame) {
                self.add_dialog(Box::new(SettingsMenu::new().await), false);
            }
        }

        // meme
        if keys_down.contains(&Key::PageUp) && mods.ctrl {
            self.add_dialog(Box::new(DraggableDialog::new(Vector2::ZERO, Box::new(StupidDialog::new().await))), true);
        }

        // update any dialogs
        if keys_down.contains(&Key::Escape) && self.ui_manager.application().dialog_manager.close_latest().await {
            keys_down.remove_item(Key::Escape)
        }



        // reload custom menus
        if keys_down.contains(&Key::R) && mods.ctrl {
            self.load_custom_menus();
            if let MenuType::Custom(name) = self.ui_manager.get_menu() {
                debug!("Reloading current menu");
                self.handle_custom_menu(name).await;
            }
        }
        
        // update our global values
        {
            let values = &mut self.shunting_yard_values;
            values.set("song.position", self.song_manager.position());

            if let Some(audio) = self.song_manager.instance() {
                values.set_multiple([
                    ("song.playing", audio.is_playing()),
                    ("song.paused", audio.is_paused()),
                    ("song.stopped", audio.is_stopped()),
                ].into_iter());
            } else {
                values.set_multiple([
                    ("song.playing", false),
                    ("song.paused", false),
                    ("song.stopped", false),
                ].into_iter());
            }
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
                text: &text,
                mods,
            }, 
            self.shunting_yard_values.take()
        ).await;
        self.shunting_yard_values = sy_values;

        // update spec and multi managers
        if let Some(spec) = &mut self.spectator_manager { 
            let manager = match &mut current_state {
                GameState::Ingame(manager) => Some(manager),
                _ => None
            };
            menu_actions.extend(spec.update(manager).await);
        }
        if let Some(multi) = &mut self.multiplayer_manager { 
            let manager = match &mut current_state {
                GameState::Ingame(manager) => Some(manager),
                _ => None
            };
            menu_actions.extend(multi.update(manager).await);
        }

        // handle menu actions
        self.handle_menu_actions(menu_actions).await;

        // run update on current state
        match &mut current_state {
            GameState::Ingame(manager) => {
                if window_size_updated {
                    manager.window_size_changed((*self.window_size).clone()).await;
                }

                // pause button, or focus lost, only if not replaying
                if let Some(got_focus) = window_focus_changed {
                    if self.settings.pause_on_focus_lost {
                        manager.window_focus_lost(got_focus)
                    }
                }
                
                if !manager.failed && manager.can_pause() && (manager.should_pause || controller_pause) {
                    manager.pause();
                    let actions = manager.actions.take();
                    let menu = PauseMenu::new(manager.take(), false).await;
                    self.queue_state_change(GameState::SetMenu(Box::new(menu)));
                    self.handle_menu_actions(actions).await;
                } else {

                    // inputs
                    // mouse
                    if mouse_moved {manager.mouse_move(mouse_pos).await}
                    for btn in mouse_down {manager.mouse_down(btn).await}
                    for btn in mouse_up {manager.mouse_up(btn).await}
                    if scroll_delta != 0.0 {manager.mouse_scroll(scroll_delta).await}

                    // kb
                    for k in keys_down.iter() {manager.key_down(*k, mods).await}
                    for k in keys_up.iter() {manager.key_up(*k).await}
                    if text.len() > 0 {
                        manager.on_text(&text, &mods).await
                    }

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
                    let actions = manager.update(&mut self.shunting_yard_values).await;
                    self.handle_menu_actions(actions).await;
                    if manager.completed {
                        self.ingame_complete(manager).await;
                    }
                }
            }
            
            // GameState::SetMenu(menu) => {

            //     // // menu input events
            //     // if window_size_updated {
            //     //     menu.window_size_changed((*self.window_size).clone()).await;
            //     // }
            //         // let events = Events


            //     // // clicks
            //     // for b in mouse_down { 
            //     //     menu.on_click(mouse_pos, b, mods, self).await;
            //     // }
            //     // for b in mouse_up { 
            //     //     menu.on_click_release(mouse_pos, b, self).await;
            //     // }

            //     // // mouse move
            //     // if mouse_moved {menu.on_mouse_move(mouse_pos, self).await}
            //     // // mouse scroll
            //     // if scroll_delta.abs() > 0.0 {menu.on_scroll(scroll_delta, self).await}


            //     // // TODO: this is temp
            //     // if keys_up.contains(&Key::S) && mods.ctrl { self.add_dialog(Box::new(SkinSelect::new().await), false) }
            //     // // TODO: this too
            //     // if keys_up.contains(&Key::G) && mods.ctrl { self.add_dialog(Box::new(GameImportDialog::new().await), false) }

            //     // // check keys down
            //     // for key in keys_down {menu.on_key_press(key, self, mods).await}
            //     // // check keys up
            //     // for key in keys_up {menu.on_key_release(key, self).await}


            //     // // controller
            //     // for (c, buttons) in controller_down {
            //     //     for b in buttons {
            //     //         menu.controller_down(self, &c, b).await;
            //     //     }
            //     // }
            //     // for (c, buttons) in controller_up {
            //     //     for b in buttons {
            //     //         menu.controller_up(self, &c, b).await;
            //     //     }
            //     // }
            //     // for (c, ad) in controller_axis {
            //     //     menu.controller_axis(self, &c, ad).await;
            //     // }


            //     // // check text
            //     // if text.len() > 0 { menu.on_text(text).await }

            //     // // window focus change
            //     // if let Some(has_focus) = window_focus_changed {
            //     //     menu.on_focus_change(has_focus, self).await;
            //     // }
                
            //     // menu.update(self).await;
            // }

            // GameState::Spectating(manager) => {   
            //     let actions = manager.update().await;
            //     self.handle_menu_actions(actions).await;

            //     if mouse_moved {manager.mouse_move(mouse_pos, self).await}
            //     for btn in mouse_down {manager.mouse_down(mouse_pos, btn, mods, self).await}
            //     for btn in mouse_up {manager.mouse_up(mouse_pos, btn, mods, self).await}
            //     if scroll_delta != 0.0 {manager.mouse_scroll(scroll_delta, self).await}

            //     for k in keys_down.iter() {manager.key_down(*k, mods, self).await}
            //     for k in keys_up.iter() {manager.key_up(*k, mods, self).await}
            // }

            GameState::None => {
                // might be transitioning
                if self.transition.is_some() && elapsed - self.transition_timer > TRANSITION_TIME / 2.0 {

                    let trans = self.transition.take();
                    self.queue_state_change(trans.unwrap());
                    self.transition_timer = elapsed;
                }
            }

            _ => {}
        }
        
        // update game mode
        match &self.queued_state {
            // queued mode didnt change, set the unlocked's mode to the updated mode
            GameState::None => self.current_state = current_state,
            GameState::Closing => {
                self.settings.save().await;
                self.current_state = GameState::Closing;
                GameWindow::send_event(Game2WindowEvent::CloseGame);
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

                        manager.start().await;
                        let m = manager.metadata.clone();
                        let start_time = manager.start_time;

                        self.set_background_beatmap(&m).await;
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
                    GameState::SetMenu(_) => {
                        if let GameState::SetMenu(menu) = &self.current_state {
                            if menu.get_name() == "pause" {
                                self.handle_menu_actions(vec![SongMenuAction::Play.into()]).await;
                                // if let Some(song) = AudioManager::get_song().await {
                                //     song.play(false);
                                // }
                            }
                        }

                        OnlineManager::set_action(SetAction::Idle, None);
                    }
                    GameState::Closing => {
                        // send logoff
                        OnlineManager::set_action(SetAction::Closing, None);
                    }
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

                    if let GameState::SetMenu(menu) = &mut self.current_state {
                        menu.on_change(true).await;
                    }
                }
            }
        }

        // update the notification manager
        NOTIFICATION_MANAGER.write().await.update(self).await;

        if let Some(mut manager) = OnlineManager::try_get_mut() {
            for host_id in std::mem::take(&mut manager.spectator_info.spectate_pending) {
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
        self.render_queue_sender.write(render_queue.take());
        NEW_RENDER_DATA_AVAILABLE.store(true, Ordering::Release);
        
        self.fps_display.increment();

        // let elapsed = timer.elapsed().as_secs_f32() * 1000.0;
        // if elapsed > 1000.0/144.0 {warn!("render took a while: {elapsed}")}
    }
    
    async fn handle_previous_menu(&mut self, current_menu: &str) -> Option<Box<dyn AsyncMenu>> {
        let in_multi = self.multiplayer_manager.is_some();
        let in_spec = self.spectator_manager.is_some();

        if in_multi { return Some(Box::new(LobbyMenu::new().await)) }
        if in_spec { return Some(Box::new(SpectatorMenu::new())) }

        match current_menu {
            // score menu with no multi or spec is the beatmap select menu
            "score" => return Some(Box::new(BeatmapSelectMenu::new().await)),

            // beatmap menu with no multi or spec is the main menu
            "beatmap_select" => return Some(Box::new(MainMenu::new().await)),

            _ => { 
                error!("unhandled previous menu request for menu {current_menu}")
            }
        }
        None
    }

    pub async fn handle_menu_actions(&mut self, actions: Vec<MenuAction>) {
        for action in actions {
            match action {
                MenuAction::None => continue,
                
                // menu actions
                MenuAction::Menu(MenuMenuAction::SetMenu(menu)) => self.queue_state_change(GameState::SetMenu(menu)),
                MenuAction::Menu(MenuMenuAction::SetMenuCustom(id)) => self.handle_custom_menu(id).await,

                MenuAction::Menu(MenuMenuAction::PreviousMenu(current_menu)) => if let Some(menu) = self.handle_previous_menu(current_menu).await {
                    self.queue_state_change(GameState::SetMenu(menu))
                }
                
                MenuAction::Menu(MenuMenuAction::AddDialog(dialog, allow_duplicates)) => self.add_dialog(dialog, allow_duplicates),
                MenuAction::Menu(MenuMenuAction::AddDialogCustom(dialog, _allow_duplicates)) => {
                    match &*dialog {
                        "settings" => self.add_dialog(Box::new(SettingsMenu::new().await), false),
                        other => warn!("unknown dialog '{other}'")
                    }
                }
                
                // beatmap actions
                MenuAction::Beatmap(BeatmapMenuAction::PlayMap(map, mode)) => {
                    match manager_from_playmode(mode, &map).await {
                        Ok(mut manager) => {
                            let mods = ModManager::get();
                            manager.apply_mods(mods.deref().clone()).await;
                            self.queue_state_change(GameState::Ingame(Box::new(manager)))
                        }
                        Err(e) => NotificationManager::add_error_notification("Error loading beatmap", e).await
                    }
                }
                MenuAction::Beatmap(BeatmapMenuAction::PlaySelected) => {
                    let Some(map) = CurrentBeatmapHelper::new().0.clone() else { continue };
                    let mode = CurrentPlaymodeHelper::new().0.clone();
                    
                    match manager_from_playmode(mode, &map).await {
                        Ok(mut manager) => {
                            let mods = ModManager::get();
                            manager.apply_mods(mods.deref().clone()).await;
                            self.queue_state_change(GameState::Ingame(Box::new(manager)))
                        }
                        Err(e) => NotificationManager::add_error_notification("Error loading beatmap", e).await
                    }
                }

                MenuAction::Beatmap(BeatmapMenuAction::Set(beatmap, use_preview_time)) => {
                    BEATMAP_MANAGER.write().await.set_current_beatmap(self, &beatmap, use_preview_time).await;
                    // warn!("setting beatmap: {}", beatmap.version_string());
                }
                MenuAction::Beatmap(BeatmapMenuAction::SetFromHash(hash, use_preview_time)) => {
                    let mut manager = BEATMAP_MANAGER.write().await;
                    if let Some(beatmap) = manager.get_by_hash(&hash) {
                        manager.set_current_beatmap(self, &beatmap, use_preview_time).await;
                    }
                }
                
                MenuAction::Beatmap(BeatmapMenuAction::Random(use_preview)) => {
                    let mut manager = BEATMAP_MANAGER.write().await;
                    let Some(random) = manager.random_beatmap() else { continue };
                    manager.set_current_beatmap(self, &random, use_preview).await;
                }
                MenuAction::Beatmap(BeatmapMenuAction::Remove) => {
                    BEATMAP_MANAGER.write().await.remove_current_beatmap(self).await;
                    // warn!("removeing beatmap");
                }

                MenuAction::Beatmap(BeatmapMenuAction::Delete(hash)) => {
                    BEATMAP_MANAGER.write().await.delete_beatmap(hash, self, PostDelete::Next).await;
                }
                MenuAction::Beatmap(BeatmapMenuAction::DeleteCurrent(post_delete)) => {
                    let Some(map) = CurrentBeatmapHelper::new().0.clone() else { continue };
                    let mut manager = BEATMAP_MANAGER.write().await;

                    manager.delete_beatmap(map.beatmap_hash, self, post_delete).await;
                }
                MenuAction::Beatmap(BeatmapMenuAction::Next) => {
                    BEATMAP_MANAGER.write().await.next_beatmap(self).await;
                }
                MenuAction::Beatmap(BeatmapMenuAction::Previous(if_none)) => {
                    let mut manager = BEATMAP_MANAGER.write().await;
                    if manager.previous_beatmap(self).await { continue }
                    
                    // no previous map availble, handle accordingly
                    match if_none {
                        MapActionIfNone::ContinueCurrent => continue,
                        MapActionIfNone::Random(use_preview) => {
                            let Some(random) = manager.random_beatmap() else { continue };
                            manager.set_current_beatmap(self, &random, use_preview).await;
                        }
                    }
                }

                
                // game actions
                MenuAction::Game(GameMenuAction::ResumeMap(manager)) => {
                    self.queue_state_change(GameState::Ingame(manager));
                }
                MenuAction::Game(GameMenuAction::StartGame(mut manager)) => {
                    manager.start().await;
                    self.queue_state_change(GameState::Ingame(manager));
                }
                MenuAction::Game(GameMenuAction::WatchReplay(replay)) => {
                    let Some((map, mode)) = replay.score_data.as_ref().map(|s|(s.beatmap_hash, s.playmode.clone())) else {
                        NotificationManager::add_text_notification("Replay has no score data", 5000.0, Color::RED).await;
                        return;
                    };

                    let Some(beatmap) = BEATMAP_MANAGER.read().await.get_by_hash(&map) else {
                        NotificationManager::add_text_notification("You don't have that map!", 5000.0, Color::RED).await;
                        return;
                    };
                    
                    match manager_from_playmode(mode, &beatmap).await {
                        Ok(mut manager) => {
                            manager.set_replay(*replay);
                            self.queue_state_change(GameState::Ingame(Box::new(manager)))
                        }
                        Err(e) => NotificationManager::add_error_notification("Error loading beatmap", e).await
                    }
                }
                MenuAction::Game(GameMenuAction::SetValue(key, value)) => {
                    self.shunting_yard_values.set(key, value);
                }

                // song actions
                MenuAction::Song(song_action) => {
                    match song_action {
                        // needs to be before trying to get the audio because audio might be none when this is run
                        SongMenuAction::Set(action) => {
                            if let Err(e) = self.song_manager.handle_song_set_action(action) {
                                error!("Error handling SongMenuSetAction: {e:?}");
                            }
                        }

                        other => {
                            let Some(audio) = self.song_manager.instance() else { 
                                continue 
                            }; 

                            match other {
                                SongMenuAction::Play => audio.play(false),
                                SongMenuAction::Restart => audio.play(true),
                                SongMenuAction::Pause => audio.pause(),
                                SongMenuAction::Stop => audio.stop(),
                                SongMenuAction::Toggle if audio.is_playing() => audio.pause(),
                                SongMenuAction::Toggle => audio.play(false),
                                SongMenuAction::SeekBy(seek) => audio.set_position(audio.get_position() + seek),
                                SongMenuAction::SetPosition(pos) => audio.set_position(pos),
                                SongMenuAction::SetRate(rate) => audio.set_rate(rate),
                                SongMenuAction::SetVolume(vol) => audio.set_volume(vol),
                                // handled above
                                SongMenuAction::Set(_) => {}
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
                        self.shunting_yard_values.set_multiple([
                            ("song.exists", true),
                            ("song.playing", audio.is_playing()),
                            ("song.paused", audio.is_paused()),
                            ("song.stopped", audio.is_stopped()),
                        ].into_iter());
                    } else {
                        self.shunting_yard_values.set_multiple([
                            ("song.exists", false),
                            ("song.playing", false),
                            ("song.paused", false),
                            ("song.stopped", false),
                        ].into_iter());
                    }
                }

                // multiplayer actions
                MenuAction::MultiplayerAction(MultiplayerManagerAction::QuitMulti) => {
                    tokio::spawn(OnlineManager::leave_lobby());
                    self.multiplayer_manager = None;
                    // TODO: check if ingame, and if yes, dont change state
                    self.queue_state_change(GameState::SetMenu(Box::new(MainMenu::new().await)));
                }
                MenuAction::MultiplayerAction(MultiplayerManagerAction::JoinMulti) => {
                    self.multiplayer_manager = Some(Box::new(MultiplayerManager::new()));
                    self.queue_state_change(GameState::SetMenu(Box::new(LobbyMenu::new().await)));
                }


                MenuAction::PerformOperation(op) => self.ui_manager.add_operation(op),
                

                MenuAction::Quit => {
                    self.queue_state_change(GameState::Closing);
                    break;
                }
            }
        }
    }

    async fn handle_custom_menu(&mut self, id: String) {
        let menu = self.custom_menus.iter().rev().find(|cm|cm.id == id);
        if let Some(menu) = menu {
            self.queue_state_change(GameState::SetMenu(Box::new(menu.build().await)))
        } else {
            match &*id {
                "none" => {}
                "beatmap_select_menu" => self.queue_state_change(GameState::SetMenu(Box::new(BeatmapSelectMenu::new().await))),
                _ => {
                    error!("custom menu not found! {id}");
                    error!("going to main menu instead");
                    self.queue_state_change(GameState::SetMenu(Box::new(MainMenu::new().await)))
                }
            }
        }
    }

    pub fn queue_state_change(&mut self, state:GameState) { 
        match state {
            GameState::SetMenu(menu) => {
                self.queued_state = GameState::InMenu(MenuType::from_menu(&menu));
                debug!("Changing menu to: {} ({:?})", menu.get_name(), menu.get_custom_name());
                self.ui_manager.set_menu(menu);
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
    pub async fn set_background_beatmap(&mut self, beatmap:&BeatmapMeta) {
        let filename = beatmap.image_filename.clone();
        let f = load_image(filename, false, Vector2::ONE);
        self.background_loader = Some(AsyncLoader::new(f));
    }
    /// shortcut for removing the game's background texture
    pub async fn remove_background_beatmap(&mut self) {
        // let filename = beatmap.image_filename.clone();
        // let f = load_image(filename, false, Vector2::ONE);
        // self.background_loader = Some(AsyncLoader::new(f));
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
                            let mut beatmap_manager = BEATMAP_MANAGER.write().await;
                            let Some(last) = beatmap_manager.check_folder(path, HandleDatabase::YesAndReturnNewMaps).await.and_then(|l|l.last().cloned()) else { warn!("didnt get any beatmaps from beatmap file drop"); return };
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
                                beatmap_manager.set_current_beatmap(self, &last, use_preview_time).await;
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

        let mut manager = BEATMAP_MANAGER.write().await;

        let Some(map) = manager.get_by_hash(&score.beatmap_hash) else {
            NotificationManager::add_text_notification("You don't have this beatmap!", 5_000.0, Color::RED).await;
            return;
        };

        manager.set_current_beatmap(self, &map, true).await;

        // move to a score menu with this as the score
        let score = IngameScore::new(score.clone(), false, false);
        let mut menu = ScoreMenu::new(&score, map, false);
        menu.replay = Some(replay);
        self.queued_state = GameState::SetMenu(Box::new(menu));
    }


    pub async fn ingame_complete(&mut self, manager: &mut Box<IngameManager>) {
        trace!("beatmap complete");
        manager.on_complete();
        manager.score.time = chrono::Utc::now().timestamp() as u64;

        if manager.failed {
            trace!("player failed");
            if !manager.get_mode().is_multi() {
                let manager2 = std::mem::take(manager);
                self.queue_state_change(GameState::SetMenu(Box::new(PauseMenu::new(manager2, true).await)));
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

                // submit score
                let submit = ScoreSubmitHelper::new(replay.clone(), &self.settings);
                submit.clone().submit();
                score_submit = Some(submit);
            }

            match manager.get_mode() {
                // go back to beatmap select
                GameplayMode::Replaying {..} => {
                    let menu = BeatmapSelectMenu::new().await; 
                    self.queue_state_change(GameState::SetMenu(Box::new(menu)));
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


    async fn await_screenshot(b: tokio::sync::oneshot::Receiver<(Vec<u8>, u32, u32)>, mods: KeyModifiers) -> TatakuResult {
        let Ok((data, width, height)) = b.await else { return Ok(()) };

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

        let mut writer = encoder.write_header().map_err(|e|TatakuError::String(format!("{e}")))?;
        writer.write_image_data(data.as_slice()).map_err(|e|TatakuError::String(format!("{e}")))?;

        // notify user
        let full_path = std::env::current_dir().unwrap().join(path).to_string_lossy().to_string();
        NotificationManager::add_notification(Notification::new(
            format!("Screenshot saved to {full_path}"), 
            Color::BLUE, 
            5000.0, 
            NotificationOnClick::File(full_path.clone())
        )).await;

        // if shift is pressed, upload to server, and get link
        if mods.shift {
            NotificationManager::add_text_notification("Uploading screenshot...", 5000.0, Color::YELLOW).await;

            let settings = SettingsHelper::new();
            let url = format!("{}/screenshots?username={}&password={}", settings.score_url, settings.username, settings.password);

            let data = match Io::read_file_async(full_path).await {
                Err(e) => { NotificationManager::add_error_notification("Error loading screenshot to send to server", TatakuError::String(e.to_string())).await; return Ok(())},
                Ok(data) => data,
            };

            let r = match reqwest::Client::new().post(url).body(data).send().await {
                Err(e) => { NotificationManager::add_error_notification("Error sending screenshot request", TatakuError::String(e.to_string())).await; return Ok(())},
                Ok(r) => r,
            };
            let b = match r.bytes().await {
                Err(e) => { NotificationManager::add_error_notification("Error reading screenshot response", TatakuError::String(e.to_string())).await; return Ok(())},
                Ok(b) => b, 
            };
            let s = match String::from_utf8(b.to_vec()) {
                Err(e) => { NotificationManager::add_error_notification("Error parsing screenshot response", TatakuError::String(e.to_string())).await; return Ok(())},
                Ok(s) => s,
            };
            let id = match s.parse::<i64>() {
                Err(e) => { NotificationManager::add_error_notification("Error parsing screenshot id", TatakuError::String(e.to_string())).await; return Ok(())},
                Ok(id) => id,
            };

            // copy to clipboard
            let url = format!("{}/screenshots/{id}", settings.score_url);
            if let Err(e) = GameWindow::set_clipboard(url.clone()) {
                warn!("Error copying to clipboard: {e}");
                NotificationManager::add_notification(Notification::new(
                    format!("Screenshot uploaded {url}"), 
                    Color::BLUE, 
                    5000.0, 
                    NotificationOnClick::Url(url)
                )).await;
            } else {
                NotificationManager::add_notification(Notification::new(
                    format!("Screenshot uploaded {url}\nLink copied to clipboard"), 
                    Color::BLUE, 
                    5000.0, 
                    NotificationOnClick::Url(url)
                )).await;
            }
        }

        Ok(())
    }

}


#[derive(Default)]
pub enum GameState {
    #[default]
    None, // use this as the inital game mode, but be sure to change it after
    Closing,
    Ingame(Box<IngameManager>),
    /// need to transition to the provided menu
    SetMenu(Box<dyn AsyncMenu>),
    /// Currently in a menu (this doesnt actually work currently, but it doesnt really matter)
    InMenu(MenuType),

    // Spectating(Box<SpectatorManager>),
}
impl GameState {
    /// spec_check means if we're spectator, check the inner game
    fn is_ingame(&self, spec_check: bool, multi_check: bool) -> bool {
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
