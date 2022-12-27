use std::path::PathBuf;

use chrono::{ Datelike, Timelike };

use crate::prelude::*;

/// how long do transitions between gamemodes last?
const TRANSITION_TIME:u64 = 500;

#[macro_export]
macro_rules! err_text_notif {
    ($str: expr) => {
        NotificationManager::add_text_notification(
            $str, 
            5_000.0, 
            Color::RED
        ).await;
    }
}

pub struct Game {
    // engine things
    pub input_manager: InputManager,
    pub volume_controller: VolumeControl,
    
    pub menus: HashMap<&'static str, Arc<Mutex<dyn ControllerInputMenu<Game>>>>,
    pub current_state: GameState,
    pub queued_state: GameState,

    pub dialogs: Vec<Box<dyn Dialog<Self>>>,
    pub wallpapers: Vec<Image>,

    // fps
    fps_display: FpsDisplay,
    update_display: FpsDisplay,
    render_display: AsyncFpsDisplay,
    input_display: AsyncFpsDisplay,

    // transition
    transition: Option<GameState>,
    transition_last: Option<GameState>,
    transition_timer: u64,

    // misc
    pub game_start: Instant,
    pub background_image: Option<Image>,
    // register_timings: (f32,f32,f32),

    game_event_receiver: tokio::sync::mpsc::Receiver<GameEvent>,
    render_queue_sender: TripleBufferSender<TatakuRenderEvent>,

    settings: SettingsHelper,
    window_size: WindowSizeHelper,
    cursor_manager: CursorManager,
    last_skin: String,

    background_loader: Option<AsyncLoader<Option<Image>>>
}
impl Game {
    pub async fn new(render_queue_sender: TripleBufferSender<TatakuRenderEvent>, game_event_receiver: tokio::sync::mpsc::Receiver<GameEvent>) -> Game {
        GlobalObjectManager::update(Arc::new(CurrentBeatmap::default()));
        GlobalObjectManager::update(Arc::new(CurrentPlaymode("osu".to_owned())));


        let mut g = Game {
            // engine
            input_manager: InputManager::new(),
            volume_controller: VolumeControl::new().await,
            dialogs: Vec::new(),
            background_image: None,
            wallpapers: Vec::new(),
            settings: SettingsHelper::new(),
            window_size: WindowSizeHelper::new(),

            menus: HashMap::new(),
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
            transition_timer: 0,

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
        // online loop
        tokio::spawn(async move {
            loop {
                OnlineManager::start().await;
                tokio::time::sleep(Duration::from_millis(1_000)).await;
            }
        });

        // make sure we have a value in the mod manager global store
        GlobalObjectManager::update(Arc::new(ModManager::new()));

        // set the current leaderboard filter
        // this is here so it happens before anything else
        let settings = SettingsHelper::new();
        SCORE_HELPER.write().await.current_method = settings.last_score_retreival_method;
        self.last_skin = settings.current_skin.clone();

        // setup double tap protection
        self.input_manager.set_double_tap_protection(settings.enable_double_tap_protection.then(|| settings.double_tap_protection_duration));

        // beatmap manager loop
        BeatmapManager::download_check_loop();

        // init diff manager
        init_diffs().await;


        // init integrations
        if settings.lastfm_enabled {
            LastFmIntegration::check().await;
        }

        
        // region == menu setup ==

        let mut loading_menu = LoadingMenu::new().await;
        loading_menu.load().await;

        // main menu
        let main_menu = Arc::new(Mutex::new(MainMenu::new().await));
        self.menus.insert("main", main_menu.clone());
        trace!("main menu created");

        // setup beatmap select menu
        let beatmap_menu = Arc::new(Mutex::new(BeatmapSelectMenu::new().await));
        self.menus.insert("beatmap", beatmap_menu.clone());
        trace!("beatmap menu created");

        // // check git updates
        // self.add_dialog(Box::new(ChangelogDialog::new().await));

        // load background images
        match std::fs::read_dir("resources/wallpapers") {
            Ok(list) => {
                for wall_file in list {
                    if let Ok(file) = wall_file {
                        if let Some(wallpaper) = load_image(file.path().to_str().unwrap(), false).await {
                            self.wallpapers.push(wallpaper)
                        }
                    }
                }
            }
            Err(_e) => {
                // NotificationManager::add_error_notification("Error loading wallpaper", e).await
            }
        }

        self.queue_state_change(GameState::InMenu(Arc::new(Mutex::new(loading_menu))));
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
                    GameEvent::WindowEvent(e) => self.input_manager.handle_events(e),
                    GameEvent::ControllerEvent(e, name) => self.input_manager.handle_controller_events(e, name),
                    
                    GameEvent::DragAndDrop(path) => self.handle_file_drop(path).await,
                    GameEvent::WindowClosed => { 
                        self.close_game(); 
                        return
                    }
                }
            }

            
            // update our settings
            let last_master_vol = self.settings.master_vol;
            let last_music_vol = self.settings.music_vol;
            let last_effect_vol = self.settings.effect_vol;
            
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

                if self.settings.current_skin != self.last_skin {
                    SkinManager::change_skin(self.settings.current_skin.clone(), false).await;
                    self.last_skin = self.settings.current_skin.clone();
                }

                // update doubletap protection
                self.input_manager.set_double_tap_protection(self.settings.enable_double_tap_protection.then(|| self.settings.double_tap_protection_duration));
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

                let window_size:[f64;2] = self.window_size.0.into();
                let args = RenderArgs {
                    ext_dt: 0.0,
                    window_size,
                    draw_size: [window_size[0] as u32, window_size[1] as u32],
                };
                self.draw(args).await;
            }

        }

    }

    pub fn close_game(&mut self) {
        warn!("stopping game");
    }

    async fn update(&mut self, _delta:f64) {
        let elapsed = self.game_start.elapsed().as_millis() as u64;
        // update the cursor
        self.cursor_manager.update(elapsed as f64).await;

        // check bg loaded
        if let Some(loader) = self.background_loader.clone() {
            if let Some(image) = loader.check().await {
                self.background_loader = None;
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
        self.fps_display.update();
        self.update_display.update();
        self.render_display.update();
        self.input_display.update();

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

        let mut controller_pause = false;
        for (c, b) in controller_down.iter() {
            if Some(crate::prelude::ControllerButton::Start) == c.map_button(*b) {
                controller_pause = true;
                break;
            }
        }

        // if keys.len() > 0 {
        //     self.register_timings = self.input_manager.get_register_delay();
        //     debug!("register times: min:{}, max: {}, avg:{}", self.register_timings.0,self.register_timings.1,self.register_timings.2);
        // }

        if mouse_down.len() > 0 {
            // check notifs
            if NOTIFICATION_MANAGER.lock().await.on_click(mouse_pos, self) {
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
                
                self.add_dialog(Box::new(UserPanel::new()));
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
            WINDOW_EVENT_QUEUE.get().unwrap().send(WindowEvent::TakeScreenshot(f)).unwrap();

            tokio::spawn(async move {

                loop {
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
                        encoder.set_color(png::ColorType::Rgb);

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
                            NotificationManager::add_text_notification("uploading screenshot", 5000.0, Color::YELLOW).await;

                            let settings = SettingsHelper::new();
                            let url = format!("{}/screenshots?username={}&password={}", settings.score_url, settings.username, settings.password);

                            let mut err:Option<(&str, TatakuError)> = None;
                            match std::fs::read(full_path) {
                                Err(e) => err = Some(("Error loading screenshot to send to server", TatakuError::String(e.to_string()))),
                                Ok(data) => match reqwest::Client::new().post(url).body(data).send().await {
                                    Err(e) => err = Some(("Error sending screenshot request", TatakuError::String(e.to_string()))),
                                    Ok(r) => match r.bytes().await {
                                        Err(e) => err = Some(("Error reading screenshot response", TatakuError::String(e.to_string()))),
                                        Ok(b) => match String::from_utf8(b.to_vec()) {
                                            Err(e) => err = Some(("Error parsing screenshot response", TatakuError::String(e.to_string()))),
                                            Ok(s) => match s.parse::<i64>() {
                                                Err(e) => err = Some(("Error parsing screenshot id", TatakuError::String(e.to_string()))),
                                                Ok(id) => {
                                                    let url = format!("{}/screenshots/{id}", settings.score_url);
                                                    // copy to clipboard
                                                    if let Err(e) = WINDOW_EVENT_QUEUE.get().unwrap().send(WindowEvent::SetClipboard(url.clone())) {
                                                        println!("error copying to clipboard: {e}");
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
                                            }
                                        }
                                    }
                                }
                            }

                            if let Some((s, err)) = err {
                                NotificationManager::add_error_notification(s, err).await
                            }
                        }

                        break;
                    }
                }
            
            });

        }

        // update any dialogs
        use crate::async_retain;

        let mut dialog_list = std::mem::take(&mut self.dialogs);
        for d in dialog_list.iter_mut().rev() {
            // kb events

            async_retain!(keys_down, k, !d.on_key_press(k, &mods, self).await);
            async_retain!(keys_up, k, !d.on_key_release(k,  &mods, self).await);

            if !text.is_empty() && d.on_text(&text).await {text = String::new()}

            // mouse events
            if mouse_moved {d.on_mouse_move(&mouse_pos, self).await}
            if d.get_bounds().contains(mouse_pos) {
                async_retain!(mouse_down, button, !d.on_mouse_down(&mouse_pos, &button, &mods, self).await);
                async_retain!(mouse_up, button, !d.on_mouse_up(&mouse_pos, &button, &mods, self).await);
                if scroll_delta != 0.0 && d.on_mouse_scroll(&scroll_delta, self).await {scroll_delta = 0.0}

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

        
        // update cursor
        if mouse_moved {CursorManager::set_pos(mouse_pos, false)}
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
                    let menu = PauseMenu::new(manager2, false);
                    self.queue_state_change(GameState::InMenu(Arc::new(Mutex::new(menu))));
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
                    for (c, b) in controller_down {
                        manager.controller_press(&c, b).await;
                    }
                    for (c, b) in controller_up {
                        manager.controller_release(&c, b).await;
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
            
            GameState::InMenu(ref menu) => {
                let mut menu = menu.lock().await;

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
                if keys_up.contains(&Key::S) && mods.ctrl { self.add_dialog(Box::new(SkinSelect::new().await)) }
                // TODO: this too
                if keys_up.contains(&Key::G) && mods.ctrl { self.add_dialog(Box::new(GameImportDialog::new().await)) }

                // check keys down
                for key in keys_down {menu.on_key_press(key, self, mods).await}
                // check keys up
                for key in keys_up {menu.on_key_release(key, self).await}


                // controller
                for (c, b) in controller_down {
                    menu.controller_down(self, &c, b).await;
                }
                for (c, b) in controller_up {
                    menu.controller_up(self, &c, b).await;
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
                if self.transition.is_some() && elapsed - self.transition_timer > TRANSITION_TIME / 2 {

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
                
                if let Err(e) = WINDOW_EVENT_QUEUE.get().unwrap().send(WindowEvent::CloseGame) {
                    panic!("no: {}", e)
                }
            }

            _ => {
                // if the old state is a menu, tell it we're changing
                if let GameState::InMenu(menu) = &current_state {
                    menu.lock().await.on_change(false).await
                }

                // let cloned_mode = self.queued_mode.clone();
                // self.threading.spawn(async move {
                //     online_manager.lock().await.discord.change_status(cloned_mode);
                //     OnlineManager::set_action(online_manager, UserAction::Leaving, String::new()).await;
                // });

                match &mut self.queued_state {
                    GameState::Ingame(manager) => {
                        let m = {
                            manager.start().await;
                            manager.metadata.clone()
                        };

                        self.set_background_beatmap(&m).await;
                        let action = SetAction::Playing { 
                            artist: m.artist.clone(),
                            title: m.title.clone(),
                            version: m.version.clone(),
                            creator: m.creator.clone(),
                            multiplayer_lobby_name: None
                        };

                        OnlineManager::set_action(action, Some(m.mode.clone()));
                    },
                    GameState::InMenu(_) => {
                        if let GameState::InMenu(menu) = &self.current_state {
                            if menu.lock().await.get_name() == "pause" {
                                if let Some(song) = AudioManager::get_song().await {
                                    song.play(false);
                                }
                            }
                        }

                        OnlineManager::set_action(SetAction::Idle, None);
                    },
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
                    GameState::InMenu(menu) if menu.lock().await.get_name() == "pause" => do_transition = false,
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

                    if let GameState::InMenu(menu) = &self.current_state {
                        menu.lock().await.on_change(true).await;
                    }
                }
            }
        }

        // update the notification manager
        NOTIFICATION_MANAGER.lock().await.update().await;


        if let Ok(manager) = &mut ONLINE_MANAGER.try_write() {
            manager.do_game_things(self).await;
        }
        
        // let elapsed = timer.elapsed().as_secs_f32() * 1000.0;
        // if elapsed > 1.0 {warn!("update took a while: {elapsed}");}

    }

    async fn draw(&mut self, args: RenderArgs) {
        // let timer = Instant::now();
        let elapsed = self.game_start.elapsed().as_millis() as u64;

        let mut render_queue = RenderableCollection::new();

        self.cursor_manager.draw(&mut render_queue).await;

        // draw background image here
        if let Some(img) = &self.background_image {
            render_queue.push(img.clone());
        }

        // should we draw the background dim?
        // if not, the other thing will handle it
        let mut draw_bg_dim = true;

        // mode
        match &mut self.current_state {
            GameState::Ingame(manager) => manager.draw(args, &mut render_queue).await,
            GameState::InMenu(menu) => {
                let mut lock = menu.lock().await;
                lock.draw(args, &mut render_queue).await;
                if lock.get_name() == "main_menu" {
                    draw_bg_dim = false;
                }
            },
            GameState::Spectating(manager) => manager.draw(args, &mut render_queue).await,
            _ => {}
        }

        if draw_bg_dim {
            render_queue.push(Rectangle::new(
                Color::BLACK.alpha(self.settings.background_dim),
                f64::MAX - 1.0,
                Vector2::zero(),
                self.window_size.0,
                None
            ));
        }

        // transition
        if self.transition_timer > 0 && elapsed - self.transition_timer < TRANSITION_TIME {
            // probably transitioning

            // draw fade in rect
            let diff = elapsed as f64 - self.transition_timer as f64;

            let mut alpha = diff / (TRANSITION_TIME as f64 / 2.0);
            if self.transition.is_none() {alpha = 1.0 - diff / TRANSITION_TIME as f64}

            render_queue.push(Rectangle::new(
                [0.0, 0.0, 0.0, alpha as f32].into(),
                -f64::MAX,
                Vector2::zero(),
                self.window_size.0,
                None
            ));

            // draw old mode
            match (&self.current_state, &self.transition_last) {
                (GameState::None, Some(GameState::InMenu(menu))) => menu.lock().await.draw(args, &mut render_queue).await,
                _ => {}
            }
        }

        // draw any dialogs
        let mut dialog_list = std::mem::take(&mut self.dialogs);
        let mut current_depth = -50_000_000.0;
        const DIALOG_DEPTH_DIFF:f64 = 50.0;
        for d in dialog_list.iter_mut() { //.rev() {
            d.draw(&args, &current_depth, &mut render_queue).await;
            current_depth += DIALOG_DEPTH_DIFF;
        }
        self.dialogs = dialog_list;

        // volume control
        self.volume_controller.draw(args, &mut render_queue).await;

        // draw fps's
        self.fps_display.draw(&mut render_queue);
        self.update_display.draw(&mut render_queue);
        self.render_display.draw(&mut render_queue);
        self.input_display.draw(&mut render_queue);
        // self.input_update_display.draw(&mut self.render_queue);

        // draw the notification manager
        NOTIFICATION_MANAGER.lock().await.draw(&mut render_queue);

        // draw cursor
        // let mouse_pressed = self.input_manager.mouse_buttons.len() > 0 
        //     || self.input_manager.key_down(settings.standard_settings.left_key)
        //     || self.input_manager.key_down(settings.standard_settings.right_key);
        // self.cursor_manager.draw(&mut self.render_queue);

        // sort the queue here (so it only needs to be sorted once per frame, instead of every time a shape is added)
        let mut render_queue = render_queue.take();
        render_queue.sort_by(|a, b| b.get_depth().partial_cmp(&a.get_depth()).unwrap());

        // toss the items to the window to render
        self.render_queue_sender.write(TatakuRenderEvent::Draw(render_queue));
        NEW_RENDER_DATA_AVAILABLE.store(true, Ordering::Release);
        
        self.fps_display.increment();

        // let elapsed = timer.elapsed().as_secs_f32() * 1000.0;
        // if elapsed > 1000.0/144.0 {warn!("render took a while: {elapsed}")}
    }
    
    pub fn queue_state_change(&mut self, state:GameState) {self.queued_state = state}

    /// shortcut for setting the game's background texture to a beatmap's image
    pub async fn set_background_beatmap(&mut self, beatmap:&BeatmapMeta) {
        // let mut helper = BenchmarkHelper::new("loaad image");
        let filename = beatmap.image_filename.clone();
        let f = async move {load_image(filename, false).await};
        self.background_loader = Some(AsyncLoader::new(f));

        // self.background_image = load_image(&beatmap.image_filename, false).await;

        // if self.background_image.is_none() && self.wallpapers.len() > 0 {
        //     self.background_image = Some(self.wallpapers[0].clone());
        // }

        // self.resize_bg();
    }

    fn resize_bg(&mut self) {
        if let Some(bg) = self.background_image.as_mut() {
            bg.origin = Vector2::zero();
            
            // resize to maintain aspect ratio
            let image_size = bg.tex_size();
            let ratio = image_size.y / image_size.x;
            if image_size.x > image_size.y {
                // use width as base
                bg.set_size(Vector2::new(
                    self.window_size.x,
                    self.window_size.x * ratio,
                ));
            } else {
                // use height as base
                bg.set_size(Vector2::new(
                    self.window_size.y * ratio,
                    self.window_size.y,
                ));
            }
            bg.pos = (self.window_size.0 - bg.size()) / 2.0;
        }
    }

    pub fn add_dialog(&mut self, dialog: Box<dyn Dialog<Self>>) {
        self.dialogs.push(dialog)
    }

    pub async fn handle_file_drop(&mut self, path: PathBuf) {
        let path = path.as_path();
        let filename = path.file_name();

        if let Some(ext) = path.extension() {
            let ext = ext.to_str().unwrap();
            match *&ext {
                "osz" | "qp" => {
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
        if let Some(score) = &replay.score_data {
            let mut manager = BEATMAP_MANAGER.write().await;

            if let Some(map) = manager.get_by_hash(&score.beatmap_hash) {
                manager.set_current_beatmap(self, &map, true).await;

                // move to a score menu with this as the score
                let score = IngameScore::new(score.clone(), false, false);
                let mut menu = ScoreMenu::new(&score, map, false);
                menu.replay = Some(replay);
                self.queued_state = GameState::InMenu(Arc::new(Mutex::new(menu)));
            } else {
                err_text_notif!("You don't have this beatmap!");
            }
        } else {
            err_text_notif!("Replay does not contain score data (too old?)");
        }
    }


    pub async fn ingame_complete(&mut self, manager: &mut IngameManager) {
        trace!("beatmap complete");
        manager.on_complete();
        manager.score.time = chrono::Utc::now().timestamp() as u64;

        if manager.failed {
            trace!("player failed");
            let manager2 = std::mem::take(manager);
            self.queue_state_change(GameState::InMenu(Arc::new(Mutex::new(PauseMenu::new(manager2, true)))));
            
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

            // used to indicate user stopped watching a replay
            if manager.replaying && !manager.started {
                // go back to beatmap select
                let menu = self.menus.get("beatmap").unwrap();
                let menu = menu.clone();
                self.queue_state_change(GameState::InMenu(menu));
            } else {
                // show score menu
                let mut menu = ScoreMenu::new(&score, manager.metadata.clone(), true);
                menu.replay = Some(replay.clone());
                menu.score_submit = score_submit;
                self.queue_state_change(GameState::InMenu(Arc::new(Mutex::new(menu))));
            }


        }
    }

}


pub enum GameState {
    None, // use this as the inital game mode, but be sure to change it after
    Closing,
    Ingame(IngameManager),
    InMenu(Arc<Mutex<dyn ControllerInputMenu<Game>>>),

    #[allow(dead_code)]
    Spectating(SpectatorManager), // frames awaiting replay, state, beatmap
    // Multiplaying(MultiplayerState), // wink wink nudge nudge (dont hold your breath)
}
impl Default for GameState {
    fn default() -> Self {
        GameState::None
    }
}

#[derive(Clone)]
pub enum GameEvent {
    WindowClosed,
    WindowEvent(piston::Event),
    DragAndDrop(PathBuf),
    /// controller event, controller name
    ControllerEvent(piston::Event, String)
}
