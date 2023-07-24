use crate::prelude::*;
use chrono::{ Datelike, Timelike };

/// how long do transitions between gamemodes last?
const TRANSITION_TIME:f32 = 500.0;

pub struct Game {
    // engine things
    input_manager: InputManager,
    volume_controller: VolumeControl,
    pub current_state: GameState,
    queued_state: GameState,
    game_event_receiver: tokio::sync::mpsc::Receiver<Window2GameEvent>,
    render_queue_sender: TripleBufferSender<RenderData>,

    pub dialogs: Vec<Box<dyn Dialog<Self>>>,

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

    background_loader: Option<AsyncLoader<Option<Image>>>
}
impl Game {
    pub async fn new(render_queue_sender: TripleBufferSender<RenderData>, game_event_receiver: tokio::sync::mpsc::Receiver<Window2GameEvent>) -> Game {
        GlobalValueManager::update(Arc::new(CurrentBeatmap::default()));
        GlobalValueManager::update(Arc::new(CurrentPlaymode("osu".to_owned())));

        let mut g = Game {
            // engine
            input_manager: InputManager::new(),
            volume_controller: VolumeControl::new().await,
            dialogs: Vec::new(),
            background_image: None,
            wallpapers: Vec::new(),
            settings: SettingsHelper::new(),
            window_size: WindowSizeHelper::new(),

            // menus: HashMap::new(),
            current_state: GameState::None,
            queued_state: GameState::None,

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
            background_loader: None
        };

        g.init().await;

        g
    }

    pub async fn init(&mut self) {
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

        self.queue_state_change(GameState::InMenu(Box::new(loading_menu)));
    }
    
    pub async fn game_loop(mut self) {
        let mut update_timer = Instant::now();
        let mut draw_timer = Instant::now();
        let mut last_draw_offset = 0.0;

        let game_start = std::time::Instant::now();


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
                    // save the settings
                    self.settings.save().await;
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
                    (true, false) => ONLINE_MANAGER.write().await.discord = None,
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
                    GameState::Spectating(sm) => if let Some(igm) = &mut sm.game_manager { 
                        if skin_changed { igm.reload_skin().await; }
                        igm.force_update_settings().await;
                    },
                    _ => {}
                }
            }


            // update our instant's time
            set_time(game_start.elapsed());
            
            let now = Instant::now();

            let update_elapsed = now.duration_since(update_timer).as_secs_f64();
            if update_elapsed >= update_target {
                update_timer = now;
                self.update(update_elapsed).await;
            }

            if let GameState::Closing = &self.current_state {
                self.close_game();
                return;
            }

            const DRAW_DAMPENING_FACTOR:f64 = 0.9;
            let elapsed = now.duration_since(draw_timer).as_secs_f64();
            if elapsed + last_draw_offset >= render_rate {
                draw_timer = now;
                last_draw_offset = (elapsed - render_rate).clamp(-5.0, 5.0) * DRAW_DAMPENING_FACTOR;
                self.draw().await;
            }

        }

    }

    pub fn close_game(&mut self) {
        warn!("stopping game");
    }

    async fn update(&mut self, _delta:f64) {
        let elapsed = self.game_start.as_millis();
        // update the cursor
        self.cursor_manager.update(elapsed, self.input_manager.mouse_pos).await;

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
            let window_size = WindowSize::get();
            for i in self.dialogs.iter_mut() {
                i.window_size_changed(window_size.clone()).await;
            }
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
        let mut mouse_up = self.input_manager.get_mouse_up();
        let mouse_moved = self.input_manager.get_mouse_moved();
        // TODO: do we want this here or only in menus?
        let mut scroll_delta = self.input_manager.get_scroll_delta() * self.settings.scroll_sensitivity;

        let mut keys_down = self.input_manager.get_keys_down();
        let mut keys_up = self.input_manager.get_keys_up();
        let mods = self.input_manager.get_key_mods();
        let mut text = self.input_manager.get_text();
        let window_focus_changed = self.input_manager.get_changed_focus();

        let controller_down = self.input_manager.get_controller_down();
        let controller_up = self.input_manager.get_controller_up();
        let controller_axis = self.input_manager.get_controller_axis();
        
        
        // update cursor
        if mouse_down.contains(&MouseButton::Left) {
            CursorManager::left_pressed(true, false)
        } else if mouse_up.contains(&MouseButton::Left) {
            CursorManager::left_pressed(false, false)
        }
        if mouse_down.contains(&MouseButton::Right) {
            CursorManager::right_pressed(true, false)
        } else if mouse_up.contains(&MouseButton::Right) {
            CursorManager::right_pressed(false, false)
        }

        let mut controller_pause = false;
        for (_c, b) in controller_down.iter() {
            if b.contains(&ControllerButton::Start) {
                controller_pause = true;
                break;
            }
        }

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
        if mouse_moved {self.volume_controller.on_mouse_move(mouse_pos)}
        if scroll_delta != 0.0 && self.volume_controller.on_mouse_wheel(scroll_delta / (self.settings.scroll_sensitivity * 1.5), mods).await {scroll_delta = 0.0}
        self.volume_controller.on_key_press(&mut keys_down, mods).await;
        
        // check user panel
        if keys_down.contains(&self.settings.key_user_panel) {
            let mut user_panel_exists = false;
            let mut chat_exists = false;
            for i in self.dialogs.iter() {
                if i.name() == "UserPanel" {
                    user_panel_exists = true;
                }
                if i.name() == "Chat" {
                    chat_exists = true;
                }
                // if both exist, no need to continue looping
                if user_panel_exists && chat_exists {break}
            }

            if !user_panel_exists {
                // close existing chat window
                if chat_exists {
                    self.dialogs.retain(|d|d.name() != "Chat");
                }
                
                self.add_dialog(Box::new(UserPanel::new()), false);
            } else {
                self.dialogs.retain(|d|d.name() != "UserPanel");
            }

            // if let Some(chat) = Chat::new() {
            //     self.add_dialog(Box::new(chat));
            // }
            // trace!("Show user list: {}", self.show_user_list);
        }

        // screenshot
        if keys_down.contains(&Key::F12) {
            let (f, b) = Bomb::new();
            GameWindow::send_event(Game2WindowEvent::TakeScreenshot(f));

            tokio::spawn(async move {
                macro_rules! check {
                    ($e:expr) => {
                        match $e {
                            Ok(e) => e,
                            Err(e) => {
                                NotificationManager::add_error_notification("Error saving screenshot", e).await;
                                break;
                            }
                        }
                    };
                }

                loop {
                    if let Some((data, width, height)) = b.exploded() {
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

                        check!(std::fs::create_dir_all(path.parent().unwrap()));
                        let file = check!(std::fs::File::create(path));

                        // save as png
                        let w = &mut std::io::BufWriter::new(file);
                        let mut encoder = png::Encoder::new(w, *width, *height);
                        encoder.set_color(png::ColorType::Rgba);

                        let mut writer = check!(encoder.write_header().map_err(|e|TatakuError::String(format!("{e}"))));
                        check!(writer.write_image_data(data.as_slice()).map_err(|e|TatakuError::String(format!("{e}"))));

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
                            if let Some((s, err)) = Self::upload_screenshot(full_path).await {
                                NotificationManager::add_error_notification(s, err).await
                            }
                        }

                        break;
                    }
                }
            
            });

        }

        // if keys_down.contains(&Key::D1) && mods.ctrl {
        //     GlobalValueManager::update(Arc::new(CurrentTheme(tataku_theme())))
        // }
        // if keys_down.contains(&Key::D2) && mods.ctrl {
        //     GlobalValueManager::update(Arc::new(CurrentTheme(osu_theme())))
        // }

        if keys_down.contains(&Key::O) && mods.ctrl {
            let allow_ingame = self.settings.common_game_settings.allow_ingame_settings;
            let is_ingame = self.current_state.is_ingame(false, true);

            // im sure theres a way to do this in one statement (without the ||) but i'm tired so too bad
            if !is_ingame || (is_ingame && allow_ingame) {
                self.add_dialog(Box::new(SettingsMenu::new().await), false);
            }
        }

        if keys_down.contains(&Key::PageDown) && mods.ctrl {
            // self.add_dialog(Box::new(DraggableDialog::new(Vector2::ZERO, Box::new(TestDialog::new()))), false);
            if ONLINE_MANAGER.read().await.logged_in {
                self.queue_state_change(GameState::InMenu(Box::new(LobbySelect::new().await)));
            } else {
                NotificationManager::add_text_notification("You must be logged in to play multiplayer!", 1000.0, Color::RED).await;
            }
        }
        if keys_down.contains(&Key::PageUp) && mods.ctrl {
            self.add_dialog(Box::new(DraggableDialog::new(Vector2::ZERO, Box::new(StupidDialog::new().await))), true);
        }

        // update any dialogs
        use crate::async_retain;

        let mut dialog_list = std::mem::take(&mut self.dialogs);

        // if escape was pressed, force close the most recent dialog
        if let Some(dialog) = dialog_list.last_mut() {
            if keys_down.contains(&Key::Escape) {
                dialog.force_close().await;
                keys_down.remove_item(Key::Escape);
            }
        }

        for d in dialog_list.iter_mut().rev() {
            if d.should_close() { continue }

            // kb events
            async_retain!(keys_down, k, !d.on_key_press(*k, &mods, self).await);
            async_retain!(keys_up, k, !d.on_key_release(*k, &mods, self).await);

            // TODO: this
            // async_retain!(controller_down, k, !d.on_controller_press(&k.0, &k.1).await);
            // async_retain!(controller_up, k, !d.on_controller_release(&k.0, &k.1).await);

            for (c, b) in controller_axis.iter() {
                d.on_controller_axis(c, b).await;
            }

            if !text.is_empty() && d.on_text(&text).await {text = String::new()}

            // mouse events
            if mouse_moved {d.on_mouse_move(mouse_pos, self).await}
            if d.get_bounds().contains(mouse_pos) {
                async_retain!(mouse_down, button, !d.on_mouse_down(mouse_pos, *button, &mods, self).await);
                async_retain!(mouse_up, button, !d.on_mouse_up(mouse_pos, *button, &mods, self).await);
                if scroll_delta != 0.0 && d.on_mouse_scroll(scroll_delta, self).await {scroll_delta = 0.0}

                mouse_down.clear();
                mouse_up.clear();
            }
            d.update(self).await
        }
        // remove any dialogs which should be closed
        dialog_list.retain(|d|!d.should_close());
        // add any new dialogs to the end of the list
        dialog_list.extend(std::mem::take(&mut self.dialogs));
        self.dialogs = dialog_list;



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
                    let manager2 = std::mem::take(manager);
                    let menu = PauseMenu::new(manager2, false).await;
                    self.queue_state_change(GameState::InMenu(Box::new(menu)));
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
                    manager.update().await;
                    if manager.completed {
                        self.ingame_complete(manager).await;
                    }
                }
            }
            
            GameState::InMenu(menu) => {

                // menu input events
                if window_size_updated {
                    menu.window_size_changed((*self.window_size).clone()).await;
                }

                // clicks
                for b in mouse_down { 
                    menu.on_click(mouse_pos, b, mods, self).await;
                }
                for b in mouse_up { 
                    menu.on_click_release(mouse_pos, b, self).await;
                }

                // mouse move
                if mouse_moved {menu.on_mouse_move(mouse_pos, self).await}
                // mouse scroll
                if scroll_delta.abs() > 0.0 {menu.on_scroll(scroll_delta, self).await}


                // TODO: this is temp
                if keys_up.contains(&Key::S) && mods.ctrl { self.add_dialog(Box::new(SkinSelect::new().await), false) }
                // TODO: this too
                if keys_up.contains(&Key::G) && mods.ctrl { self.add_dialog(Box::new(GameImportDialog::new().await), false) }

                // check keys down
                for key in keys_down {menu.on_key_press(key, self, mods).await}
                // check keys up
                for key in keys_up {menu.on_key_release(key, self).await}


                // controller
                for (c, buttons) in controller_down {
                    for b in buttons {
                        menu.controller_down(self, &c, b).await;
                    }
                }
                for (c, buttons) in controller_up {
                    for b in buttons {
                        menu.controller_up(self, &c, b).await;
                    }
                }
                for (c, ad) in controller_axis {
                    menu.controller_axis(self, &c, ad).await;
                }


                // check text
                if text.len() > 0 { menu.on_text(text).await }

                // window focus change
                if let Some(has_focus) = window_focus_changed {
                    menu.on_focus_change(has_focus, self).await;
                }

                menu.update(self).await;
            }

            GameState::Spectating(manager) => {   
                manager.update(self).await;

                if mouse_moved {manager.mouse_move(mouse_pos, self).await}
                for btn in mouse_down {manager.mouse_down(mouse_pos, btn, mods, self).await}
                for btn in mouse_up {manager.mouse_up(mouse_pos, btn, mods, self).await}
                if scroll_delta != 0.0 {manager.mouse_scroll(scroll_delta, self).await}

                for k in keys_down.iter() {manager.key_down(*k, mods, self).await}
                for k in keys_up.iter() {manager.key_up(*k, mods, self).await}
            }

            GameState::None => {
                // might be transitioning
                if self.transition.is_some() && elapsed - self.transition_timer > TRANSITION_TIME / 2.0 {

                    let trans = std::mem::take(&mut self.transition);
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
                for i in self.dialogs.iter_mut() {
                    i.force_close().await;
                }
                self.dialogs.clear();

                // if the old state is a menu, tell it we're changing
                if let GameState::InMenu(menu) = &mut current_state {
                    menu.on_change(false).await
                }

                // let cloned_mode = self.queued_mode.clone();
                // self.threading.spawn(async move {
                //     online_manager.lock().await.discord.change_status(cloned_mode);
                //     OnlineManager::set_action(online_manager, UserAction::Leaving, String::new()).await;
                // });

                match &mut self.queued_state {
                    GameState::Ingame(manager) => {
                        manager.start().await;
                        let m = manager.metadata.clone();
                        let start_time = manager.start_time;

                        self.set_background_beatmap(&m).await;
                        let action = SetAction::Playing { 
                            artist: m.artist.clone(),
                            title: m.title.clone(),
                            version: m.version.clone(),
                            creator: m.creator.clone(),
                            multiplayer_lobby_name: None,
                            start_time
                        };

                        OnlineManager::set_action(action, Some(m.mode.clone()));
                    }
                    GameState::InMenu(_) => {
                        if let GameState::InMenu(menu) = &self.current_state {
                            if menu.get_name() == "pause" {
                                if let Some(song) = AudioManager::get_song().await {
                                    song.play(false);
                                }
                            }
                        }

                        OnlineManager::set_action(SetAction::Idle, None);
                    }
                    GameState::Closing => {
                        // send logoff
                        OnlineManager::set_action(SetAction::Closing, None);
                    }
                    GameState::Spectating(manager) => {
                        if let Some(gm) = &manager.game_manager {
                            let m = gm.beatmap.get_beatmap_meta();
                            let action = SetAction::Spectating { 
                                artist: m.artist.clone(),
                                title: m.title.clone(),
                                version: m.version.clone(),
                                creator: m.creator.clone(),
                                player: String::new()
                            };

                            OnlineManager::set_action(action, None);
                        } else {
                            OnlineManager::set_action(SetAction::Idle, None);
                        }
                    }
                    _ => {}
                }

                let mut do_transition = true;
                match &current_state {
                    GameState::None => do_transition = false,
                    GameState::InMenu(menu) if menu.get_name() == "pause" => do_transition = false,
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

                    if let GameState::InMenu(menu) = &mut self.current_state {
                        menu.on_change(true).await;
                    }
                }
            }
        }

        // update the notification manager
        NOTIFICATION_MANAGER.write().await.update().await;


        if let Ok(manager) = &mut ONLINE_MANAGER.try_write() {
            manager.do_game_things(self).await;
        }
        
        // let elapsed = timer.elapsed().as_secs_f32() * 1000.0;
        // if elapsed > 1.0 {warn!("update took a while: {elapsed}");}
    }

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
        match &mut self.current_state {
            GameState::Ingame(manager) => manager.draw(&mut render_queue).await,
            GameState::InMenu(menu) => menu.draw(&mut render_queue).await,
            GameState::Spectating(manager) => manager.draw(&mut render_queue).await,
            _ => {}
        }

        // transition
        if self.transition_timer > 0.0 && elapsed - self.transition_timer < TRANSITION_TIME {
            // probably transitioning

            // draw old mode
            match (&self.current_state, &mut self.transition_last) {
                (GameState::None, Some(GameState::InMenu(menu))) => menu.draw(&mut render_queue).await,
                _ => {}
            }
            
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

        // draw any dialogs
        let mut dialog_list = std::mem::take(&mut self.dialogs);
        for d in dialog_list.iter_mut() { //.rev() {
            d.draw(Vector2::ZERO, &mut render_queue).await;
        }
        self.dialogs = dialog_list;

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

        // sort the queue here (so it only needs to be sorted once per frame, instead of every time a shape is added)
        let render_queue = render_queue.take();
        // render_queue.sort_by(|a, b| b.get_depth().partial_cmp(&a.get_depth()).unwrap());

        // toss the items to the window to render
        self.render_queue_sender.write(render_queue);
        NEW_RENDER_DATA_AVAILABLE.store(true, Ordering::Release);
        
        self.fps_display.increment();

        // let elapsed = timer.elapsed().as_secs_f32() * 1000.0;
        // if elapsed > 1000.0/144.0 {warn!("render took a while: {elapsed}")}
    }
    
    pub fn queue_state_change(&mut self, state:GameState) { self.queued_state = state; }

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
        if let Some(bg) = &mut self.background_image {
            bg.fit_to_bg_size(self.window_size.0, false);
        }
    }

    pub fn add_dialog(&mut self, dialog: Box<dyn Dialog<Self>>, allow_duplicates: bool) {
        if !allow_duplicates {
            // check if said dialog already exists, if so, dont add it
            let name = dialog.name();
            if let Some(_) = self.dialogs.iter().find(|n|n.name() == name) { return }
        }

        self.dialogs.push(dialog)
    }

    pub async fn handle_file_drop(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        let filename = path.file_name();

        if let Some(ext) = path.extension() {
            let ext = ext.to_str().unwrap();
            match *&ext {
                // osu | quaver | ptyping zipped set file
                "osz" | "qp" | "ptm" => {
                    if let Err(e) = std::fs::copy(path, format!("{}/{}", DOWNLOADS_DIR, filename.unwrap().to_str().unwrap())) {
                        error!("Error copying file: {}", e);
                        NotificationManager::add_error_notification(
                            "Error copying file", 
                            e
                        ).await;
                    } else {
                        NotificationManager::add_text_notification(
                            "Set file added, it will be loaded soon...", 
                            2_000.0, 
                            Color::BLUE
                        ).await;
                    }
                }

                // tataku | osu replay
                "ttkr" | "osr" => {
                    match read_other_game_replay(path).await {
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
        self.queued_state = GameState::InMenu(Box::new(menu));
    }


    pub async fn ingame_complete(&mut self, manager: &mut Box<IngameManager>) {
        trace!("beatmap complete");
        manager.on_complete();
        manager.score.time = chrono::Utc::now().timestamp() as u64;

        if manager.failed {
            trace!("player failed");
            if !manager.multiplayer {
                let manager2 = std::mem::take(manager);
                self.queue_state_change(GameState::InMenu(Box::new(PauseMenu::new(manager2, true).await)));
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

            if !manager.multiplayer {
                // used to indicate user stopped watching a replay
                if manager.replaying && !manager.started {
                    // go back to beatmap select
                    // let menu = self.menus.get("beatmap").unwrap();
                    let menu = BeatmapSelectMenu::new().await; 
                    self.queue_state_change(GameState::InMenu(Box::new(menu)));
                } else {
                    // show score menu
                    let mut menu = ScoreMenu::new(&score, manager.metadata.clone(), true);
                    menu.replay = Some(replay.clone());
                    menu.score_submit = score_submit;
                    self.queue_state_change(GameState::InMenu(Box::new(menu)));
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


    async fn upload_screenshot(full_path: String) -> Option<(&'static str, TatakuError)> {
        NotificationManager::add_text_notification("Uploading screenshot...", 5000.0, Color::YELLOW).await;

        let settings = SettingsHelper::new();
        let url = format!("{}/screenshots?username={}&password={}", settings.score_url, settings.username, settings.password);

        let data = match Io::read_file_async(full_path).await {
            Err(e) => return Some(("Error loading screenshot to send to server", TatakuError::String(e.to_string()))),
            Ok(data) => data,
        };

        let r = match reqwest::Client::new().post(url).body(data).send().await {
            Err(e) => return Some(("Error sending screenshot request", TatakuError::String(e.to_string()))),
            Ok(r) => r,
        };
        let b = match r.bytes().await {
            Err(e) => return Some(("Error reading screenshot response", TatakuError::String(e.to_string()))),
            Ok(b) => b, 
        };
        let s = match String::from_utf8(b.to_vec()) {
            Err(e) => return Some(("Error parsing screenshot response", TatakuError::String(e.to_string()))),
            Ok(s) => s,
        };
        let id = match s.parse::<i64>() {
            Err(e) => return Some(("Error parsing screenshot id", TatakuError::String(e.to_string()))),
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

        None
    }
}


#[derive(Default)]
pub enum GameState {
    #[default]
    None, // use this as the inital game mode, but be sure to change it after
    Closing,
    Ingame(Box<IngameManager>),
    InMenu(Box<dyn ControllerInputMenu<Game>>),

    Spectating(SpectatorManager), // frames awaiting replay, state, beatmap
    // Multiplaying(MultiplayerState), // wink wink nudge nudge (dont hold your breath)
}
impl GameState {
    /// spec_check means if we're spectator, check the inner game
    fn is_ingame(&self, spec_check: bool, _multi_check: bool) -> bool {
        match self {
            Self::Ingame(_) => true,
            Self::Spectating(s) if spec_check => s.game_manager.is_some(),
            
            _ => false
        }
    }
}

// pub enum MultiplayerState {
//     InLobby(Box<LobbyMenu>),
//     Ingame(Box<IngameManager>),
//     BeatmapSelect(Box<BeatmapSelectMenu>),
// }