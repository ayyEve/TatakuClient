use crate::prelude::*;
use crate::databases::save_replay;

pub trait SendSyncTest: Send + Sync {}
impl SendSyncTest for Game {}

/// how long do transitions between gamemodes last?
const TRANSITION_TIME:u64 = 500;

pub struct Game {
    // engine things
    render_queue: Vec<Box<dyn Renderable>>,
    // pub window: AppWindow,
    // pub graphics: GlGraphics,
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
    // input_update_display: FpsDisplay,

    // transition
    transition: Option<GameState>,
    transition_last: Option<GameState>,
    transition_timer: u64,

    // misc
    pub game_start: Instant,
    pub background_image: Option<Image>,
    // register_timings: (f32,f32,f32),

    // #[cfg(feature="bass_audio")]
    // #[allow(dead_code)]
    // /// needed to prevent bass from deinitializing
    // bass: bass_rs::Bass,

    game_event_receiver: MultiBomb<GameEvent>,
    render_queue_sender: TripleBufferSender<TatakuRenderEvent>,

}
impl Game {
    pub async fn new(render_queue_sender: TripleBufferSender<TatakuRenderEvent>, game_event_receiver: MultiBomb<GameEvent>,) -> Game {
        let input_manager = InputManager::new();

        let mut g = Game {
            // engine
            input_manager,
            volume_controller:VolumeControl::new(),
            render_queue: Vec::new(),
            dialogs: Vec::new(),
            background_image: None,
            wallpapers: Vec::new(),

            menus: HashMap::new(),
            current_state: GameState::None,
            queued_state: GameState::None,

            // fps
            fps_display: FpsDisplay::new("fps", 0),
            update_display: FpsDisplay::new("updates/s", 1),
            // input_update_display: FpsDisplay::new("inputs/s", 2),

            // transition
            transition: None,
            transition_last: None,
            transition_timer: 0,

            // cursor

            // misc
            game_start: Instant::now(),
            // register_timings: (0.0,0.0,0.0),
            game_event_receiver,
            render_queue_sender,
        };
        // game_init_benchmark.log("game created", true);

        CursorManager::init();
        g.init().await;
        // game_init_benchmark.log("game initialized", true);

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

        // beatmap manager loop
        BeatmapManager::download_check_loop();
        
        let mut loading_menu = LoadingMenu::new();
        loading_menu.load().await;

        // region == menu setup ==
        let mut menu_init_benchmark = BenchmarkHelper::new("Game::init");
        // main menu
        let main_menu = Arc::new(Mutex::new(MainMenu::new().await));
        self.menus.insert("main", main_menu.clone());
        menu_init_benchmark.log("main menu created", true);

        // setup beatmap select menu
        let beatmap_menu = Arc::new(Mutex::new(BeatmapSelectMenu::new().await));
        self.menus.insert("beatmap", beatmap_menu.clone());
        menu_init_benchmark.log("beatmap menu created", true);

        // check git updates
        self.add_dialog(Box::new(ChangelogDialog::new().await));

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
            Err(e) => {
                NotificationManager::add_error_notification("Error loading wallpaper", e).await
            }
        }

        self.queue_state_change(GameState::InMenu(Arc::new(Mutex::new(loading_menu))));
    }
    pub async fn game_loop(mut self) {
        let mut update_timer = Instant::now();
        let mut draw_timer = Instant::now();
        let mut last_draw_offset = 0.0;

        let window_size:[f64;2] = Settings::window_size().into();
        let args = RenderArgs {
            ext_dt: 0.0,
            window_size,
            draw_size: [window_size[0] as u32, window_size[1] as u32],
        };

        let settings = get_settings!().clone();
        let render_rate = 1.0 / settings.fps_target as f64;
        let update_target = 1.0 / settings.update_target as f64;

        loop {
            while let Some(e) = self.game_event_receiver.exploded() {
                match e {
                    GameEvent::WindowEvent(e) => self.input_manager.handle_events(e),
                    GameEvent::ControllerEvent(e, name) => self.input_manager.handle_controller_events(e, name),
                    GameEvent::WindowClosed => { 
                        self.close_game(); 
                        return
                    }
                }
            }

            let update_now = Instant::now();
            let update_elapsed = update_now.duration_since(update_timer).as_secs_f64();
            if update_elapsed >= update_target {
                update_timer = update_now;
                self.update(update_elapsed).await;
            }

            if let GameState::Closing = &self.current_state {
                self.close_game();
                return;
            }

            const RENDER_DAMPENING_FACTOR:f64 = 0.9;
            let now = Instant::now();
            let elapsed = now.duration_since(draw_timer).as_secs_f64();
            if elapsed + last_draw_offset >= render_rate {
                draw_timer = now;
                last_draw_offset = (elapsed - render_rate).clamp(-5.0, 5.0) * RENDER_DAMPENING_FACTOR;
                self.render(args).await;
            }

        }

    }

    pub fn close_game(&mut self) {
        warn!("stopping game");
    }

    async fn update(&mut self, _delta:f64) {
        // let timer = Instant::now();
        let elapsed = self.game_start.elapsed().as_millis() as u64;
        self.update_display.increment();
        let mut current_state = std::mem::take(&mut self.current_state);

        // read input events
        let mouse_pos = self.input_manager.mouse_pos;
        let mut mouse_down = self.input_manager.get_mouse_down();
        let mut mouse_up = self.input_manager.get_mouse_up();
        let mouse_moved = self.input_manager.get_mouse_moved();
        let mut scroll_delta = self.input_manager.get_scroll_delta();

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

        let settings_clone = get_settings!().clone();
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
        if scroll_delta != 0.0 && self.volume_controller.on_mouse_wheel(scroll_delta, mods).await {scroll_delta = 0.0}
        self.volume_controller.on_key_press(&mut keys_down, mods).await;
        
        // check user panel
        if keys_down.contains(&settings_clone.key_user_panel) {
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

                // pause button, or focus lost, only if not replaying
                if let Some(got_focus) = window_focus_changed {
                    if settings_clone.pause_on_focus_lost {
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
                        trace!("beatmap complete");
                        manager.on_complete();

                        if manager.failed {
                            trace!("player failed");
                            let manager2 = std::mem::take(manager);
                            self.queue_state_change(GameState::InMenu(Arc::new(Mutex::new(PauseMenu::new(manager2, true)))));
                            
                        } else {
                            let score = &manager.score;
                            let replay = &manager.replay;

                            if manager.should_save_score() {
                                // save score
                                Database::save_score(&score).await;
                                match save_replay(&replay, &score) {
                                    Ok(_)=> trace!("replay saved ok"),
                                    Err(e) => NotificationManager::add_error_notification("error saving replay", e).await,
                                }

                                // submit score
                                #[cfg(feature = "online_scores")] {
                                    self.threading.spawn(async move {
                                        //TODO: do this async
                                        trace!("submitting score");
                                        let mut writer = SerializationWriter::new();
                                        writer.write(score.clone());
                                        writer.write(replay.clone());
                                        let data = writer.data();
                                        
                                        let c = reqwest::Client::new();
                                        let res = c.post("http://localhost:8000/score_submit")
                                            .body(data)
                                            .send().await;

                                        match res {
                                            Ok(_isgood) => {
                                                //TODO: do something with the response?
                                                trace!("score submitted successfully");
                                            },
                                            Err(e) => error!("error submitting score: {}", e),
                                        }
                                    });
                                }

                            }

                            // used to indicate user stopped watching a replay
                            if manager.replaying && !manager.started {
                                // go back to beatmap select
                                let menu = self.menus.get("beatmap").unwrap();
                                let menu = menu.clone();
                                self.queue_state_change(GameState::InMenu(menu));
                            } else {
                                // show score menu
                                let menu = ScoreMenu::new(&score, manager.metadata.clone());
                                self.queue_state_change(GameState::InMenu(Arc::new(Mutex::new(menu))));
                            }
                        }

                    }
                }
            }
            
            GameState::InMenu(ref menu) => {
                let mut menu = menu.lock().await;

                // menu input events

                // clicks
                for b in mouse_down { 
                    // game.start_map() can happen here, which needs &mut self
                    menu.on_click(mouse_pos, b, mods, self).await;
                }
                for b in mouse_up { 
                    // game.start_map() can happen here, which needs &mut self
                    menu.on_click_release(mouse_pos, b, self).await;
                }

                // mouse move
                if mouse_moved {menu.on_mouse_move(mouse_pos, self).await}
                // mouse scroll
                if scroll_delta.abs() > 0.0 {menu.on_scroll(scroll_delta, self).await}


                // TODO: this is temp
                if keys_up.contains(&Key::M) && mods.ctrl {self.add_dialog(Box::new(ModDialog::new().await))}
                // TODO: this too
                if keys_up.contains(&Key::S) && mods.ctrl {self.add_dialog(Box::new(SkinSelect::new().await))}

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
                if text.len() > 0 {menu.on_text(text).await}

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
                get_settings!().save().await;
                // self.window.set_should_close(true);
                self.current_state = GameState::Closing;
                
                if let Err(e) = WINDOW_EVENT_QUEUE.get().unwrap().send(RenderSideEvent::CloseGame) {
                    panic!("no: {}", e)
                }
            }

            _ => {
                // if the mode is being changed, clear all shapes, even ones with a lifetime
                self.clear_render_queue(true);

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
                        let text = format!("Playing {}-{}[{}]", m.artist, m.title, m.version);
                        OnlineManager::set_action(UserAction::Ingame, text, m.mode);
                    },
                    GameState::InMenu(_) => {
                        if let GameState::InMenu(menu) = &self.current_state {
                            if menu.lock().await.get_name() == "pause" {
                                if let Some(song) = Audio::get_song().await {
                                    #[cfg(feature="bass_audio")]
                                    song.play(false).unwrap();
                                    #[cfg(feature="neb_audio")]
                                    song.play();
                                }
                            }
                        }

                        OnlineManager::set_action(UserAction::Idle, String::new(), String::new());
                    },
                    GameState::Closing => {
                        // send logoff
                        OnlineManager::set_action(UserAction::Leaving, String::new(), String::new());
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

    async fn render(&mut self, args: RenderArgs) {
        // let timer = Instant::now();
        let settings = get_settings!();
        let elapsed = self.game_start.elapsed().as_millis() as u64;

        // draw background image here
        if let Some(img) = &self.background_image {
            self.render_queue.push(Box::new(img.clone()));
        }

        // should we draw the background dim?
        // if not, the other thing will handle it
        let mut draw_bg_dim = true;

        // mode
        match &mut self.current_state {
            GameState::Ingame(manager) => manager.draw(args, &mut self.render_queue).await,
            GameState::InMenu(menu) => {
                let mut lock = menu.lock().await;
                self.render_queue.extend(lock.draw(args).await);
                if lock.get_name() == "main_menu" {
                    draw_bg_dim = false;
                }
            },
            GameState::Spectating(manager) => manager.draw(args, &mut self.render_queue).await,
            _ => {}
        }

        if draw_bg_dim {
            self.render_queue.push(Box::new(Rectangle::new(
                Color::BLACK.alpha(settings.background_dim),
                f64::MAX - 1.0,
                Vector2::zero(),
                Settings::window_size(),
                None
            )));
        }

        // transition
        if self.transition_timer > 0 && elapsed - self.transition_timer < TRANSITION_TIME {
            // probably transitioning

            // draw fade in rect
            let diff = elapsed as f64 - self.transition_timer as f64;

            let mut alpha = diff / (TRANSITION_TIME as f64 / 2.0);
            if self.transition.is_none() {alpha = 1.0 - diff / TRANSITION_TIME as f64}

            self.render_queue.push(Box::new(Rectangle::new(
                [0.0, 0.0, 0.0, alpha as f32].into(),
                -f64::MAX,
                Vector2::zero(),
                Settings::window_size(),
                None
            )));

            // draw old mode
            match (&self.current_state, &self.transition_last) {
                (GameState::None, Some(GameState::InMenu(menu))) => self.render_queue.extend(menu.lock().await.draw(args).await),
                _ => {}
            }
        }

        // draw any dialogs
        let mut dialog_list = std::mem::take(&mut self.dialogs);
        let mut current_depth = -50_000_000.0;
        const DIALOG_DEPTH_DIFF:f64 = 50.0;
        for d in dialog_list.iter_mut() { //.rev() {
            d.draw(&args, &current_depth, &mut self.render_queue).await;
            current_depth += DIALOG_DEPTH_DIFF;
        }
        self.dialogs = dialog_list;

        // volume control
        self.render_queue.extend(self.volume_controller.draw(args).await);

        // draw fps's
        self.fps_display.draw(&mut self.render_queue);
        self.update_display.draw(&mut self.render_queue);
        // self.input_update_display.draw(&mut self.render_queue);

        // draw the notification manager
        NOTIFICATION_MANAGER.lock().await.draw(&mut self.render_queue);

        // draw cursor
        // let mouse_pressed = self.input_manager.mouse_buttons.len() > 0 
        //     || self.input_manager.key_down(settings.standard_settings.left_key)
        //     || self.input_manager.key_down(settings.standard_settings.right_key);
        // self.cursor_manager.draw(&mut self.render_queue);

        // sort the queue here (so it only needs to be sorted once per frame, instead of every time a shape is added)
        self.render_queue.sort_by(|a, b| b.get_depth().partial_cmp(&a.get_depth()).unwrap());

        // toss the items to the window to render
        let data = std::mem::take(&mut self.render_queue);
        self.render_queue_sender.write(TatakuRenderEvent::Draw(data));
        
        
        // trace!("clearing");
        // self.clear_render_queue(false);
        self.fps_display.increment();

        // let elapsed = timer.elapsed().as_secs_f32() * 1000.0;
        // if elapsed > 1000.0/144.0 {warn!("render took a while: {elapsed}")}
    }

    pub fn clear_render_queue(&mut self, _remove_all:bool) {
        // if remove_all {return self.render_queue.clear()}

        // let elapsed = self.game_start.elapsed().as_millis() as u64;
        // only return items who's lifetime has expired
        // self.render_queue.retain(|e| {
        //     let lifetime = e.get_lifetime();
        //     lifetime > 0 && elapsed - e.get_spawn_time() < lifetime
        // });
        self.render_queue.clear();
    }
    
    pub fn queue_state_change(&mut self, state:GameState) {self.queued_state = state}

    /// shortcut for setting the game's background texture to a beatmap's image
    pub async fn set_background_beatmap(&mut self, beatmap:&BeatmapMeta) {
        // let mut helper = BenchmarkHelper::new("loaad image");

        self.background_image = load_image(&beatmap.image_filename, false).await;

        if self.background_image.is_none() && self.wallpapers.len() > 0 {
            self.background_image = Some(self.wallpapers[0].clone());
        }

        if let Some(bg) = self.background_image.as_mut() {
            bg.origin = Vector2::zero();
            
            // resize to maintain aspect ratio
            let window_size = Settings::window_size();
            let image_size = bg.tex_size();
            let ratio = image_size.y / image_size.x;
            if image_size.x > image_size.y {
                // use width as base
                bg.set_size(Vector2::new(
                    window_size.x,
                    window_size.x * ratio,
                ));
            } else {
                // use height as base
                bg.set_size(Vector2::new(
                    window_size.y * ratio,
                    window_size.y,
                ));
            }
            bg.initial_pos = (window_size - bg.size()) / 2.0;
            bg.current_pos = bg.initial_pos;
        }
    
    }

    pub fn add_dialog(&mut self, dialog: Box<dyn Dialog<Self>>) {
        self.dialogs.push(dialog)
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


#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum SpectatorState {
    None, // Default
    Buffering, // waiting for data
    Watching, // host playing
    Paused, // host paused
    MapChanging, // host is changing map
}


#[derive(Clone)]
pub enum GameEvent {
    WindowClosed,
    WindowEvent(piston::Event),
    /// controller event, controller name
    ControllerEvent(piston::Event, String)
}
