use glfw_window::GlfwWindow as AppWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{Window, input::*, event_loop::*, window::WindowSettings};

use crate::prelude::*;
use crate::databases::save_replay;


/// background color
const GFX_CLEAR_COLOR:Color = Color::BLACK;
/// how long do transitions between gamemodes last?
const TRANSITION_TIME:u64 = 500;

pub struct Game {
    // engine things
    render_queue: Vec<Box<dyn Renderable>>,
    pub window: AppWindow,
    pub graphics: GlGraphics,
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

    // cursor
    cursor_manager: CursorManager,

    // misc
    pub game_start: Instant,
    pub background_image: Option<Image>,
    // register_timings: (f32,f32,f32),

    #[cfg(feature="bass_audio")]
    #[allow(dead_code)]
    /// needed to prevent bass from deinitializing
    bass: bass_rs::Bass,
}
impl Game {
    pub fn new() -> Game {
        let mut game_init_benchmark = BenchmarkHelper::new("Game::new");
        let window_size = Settings::window_size();

        let opengl = OpenGL::V3_2;
        let mut window: AppWindow = WindowSettings::new("Tataku!", [window_size.x, window_size.y])
            .graphics_api(opengl)
            .resizable(false)
            // .fullscreen(true) // this doesnt work?
            // .samples(32) // not sure if this actually works or not
            .build()
            .expect("Error creating window");
        window.window.set_cursor_mode(glfw::CursorMode::Hidden);
        game_init_benchmark.log("window created", true);


        #[cfg(feature="bass_audio")] 
        let bass = {
            #[cfg(target_os = "windows")]
            let window_ptr = window.window.get_win32_window();
            #[cfg(target_os = "linux")]
            let window_ptr = window.window.get_x11_window();
            #[cfg(target_os = "macos")]
            let window_ptr = window.window.get_cocoa_window();

            // initialize bass
            bass_rs::Bass::init_default_with_ptr(window_ptr).expect("Error initializing bass")
        };

        // set window icon
        match image::open("resources/icon-small.png") {
            Ok(img) => {
                window.window.set_icon(vec![img.into_rgba8()]);
                game_init_benchmark.log("window icon set", true);
            }
            Err(e) => {
                game_init_benchmark.log(&format!("error setting window icon: {}", e), true);
            }
        }

        let graphics = GlGraphics::new(opengl);
        game_init_benchmark.log("graphics created", true);

        let input_manager = InputManager::new();
        game_init_benchmark.log("input manager created", true);

        let mut g = Game {
            // engine
            window,
            graphics,
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
            cursor_manager: CursorManager::new(),

            // misc
            game_start: Instant::now(),
            // register_timings: (0.0,0.0,0.0),
            #[cfg(feature="bass_audio")] 
            bass,
        };
        game_init_benchmark.log("game created", true);

        g.init();
        game_init_benchmark.log("game initialized", true);

        g
    }

    pub fn init(&mut self) {
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
        loading_menu.load();

        // region == menu setup ==
        let mut menu_init_benchmark = BenchmarkHelper::new("Game::init");
        // main menu
        let main_menu = Arc::new(Mutex::new(MainMenu::new()));
        self.menus.insert("main", main_menu.clone());
        menu_init_benchmark.log("main menu created", true);

        // setup beatmap select menu
        let beatmap_menu = Arc::new(Mutex::new(BeatmapSelectMenu::new()));
        self.menus.insert("beatmap", beatmap_menu.clone());
        menu_init_benchmark.log("beatmap menu created", true);

        // check git updates
        self.add_dialog(Box::new(ChangelogDialog::new()));

        // load background images
        match std::fs::read_dir("resources/wallpapers") {
            Ok(list) => {
                for wall_file in list {
                    if let Ok(file) = wall_file {
                        if let Some(wallpaper) = load_image(file.path().to_str().unwrap(), false) {
                            self.wallpapers.push(wallpaper)
                        }
                    }
                }
            }
            Err(e) => {
                NotificationManager::add_error_notification("Error loading wallpaper", e)
            }
        }

        self.queue_state_change(GameState::InMenu(Arc::new(Mutex::new(loading_menu))));
    }
    pub fn game_loop(mut self) {
        let mut events = Events::new(EventSettings::new());
        // events.set_ups_reset(0);

        {
            // input and rendering thread times
            let settings = get_settings!();
            events.set_max_fps(settings.fps_target);
            events.set_ups(settings.update_target);
        }

        while let Some(e) = events.next(&mut self.window) {
            self.input_manager.handle_events(e.clone(), &mut self.window);
            if let Some(args) = e.update_args() {self.update(args.dt*1000.0)}
            if let Some(args) = e.render_args() {self.render(args)}
            // if let Some(Button::Keyboard(_)) = e.press_args() {self.input_update_display.increment()}


            if let Event::Input(Input::FileDrag(FileDrag::Drop(d)), _) = e {
                debug!("got file: {:?}", d);
                let path = d.as_path();
                let filename = d.file_name();

                if let Some(ext) = d.extension() {
                    let ext = ext.to_str().unwrap();
                    match *&ext {
                        "osz" | "qp" => {
                            if let Err(e) = std::fs::copy(path, format!("{}/{}", DOWNLOADS_DIR, filename.unwrap().to_str().unwrap())) {
                                error!("Error copying file: {}", e);
                                NotificationManager::add_error_notification(
                                    "Error copying file", 
                                    e
                                );
                            } else {
                                NotificationManager::add_text_notification(
                                    "Set file added, it will be loaded soon...", 
                                    2_000.0, 
                                    Color::BLUE
                                );
                            }
                        }

                        _ => {
                            NotificationManager::add_text_notification(
                                &format!("What is this?"), 
                                3_000.0, 
                                Color::RED
                            );
                        }
                    }
                }
            }
            // e.resize(|args| debug!("Resized '{}, {}'", args.window_size[0], args.window_size[1]));
        }
    }

    fn update(&mut self, _delta:f64) {
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
                controller_pause = true
            }
        }

        let settings_clone = get_settings!().clone();
        // if keys.len() > 0 {
        //     self.register_timings = self.input_manager.get_register_delay();
        //     debug!("register times: min:{}, max: {}, avg:{}", self.register_timings.0,self.register_timings.1,self.register_timings.2);
        // }

        if mouse_down.len() > 0 {
            // check notifs
            if NOTIFICATION_MANAGER.lock().on_click(mouse_pos, self) {
                mouse_down.clear();
            }
        }

        // check for volume change
        if mouse_moved {self.volume_controller.on_mouse_move(mouse_pos)}
        if scroll_delta != 0.0 && self.volume_controller.on_mouse_wheel(scroll_delta, mods) {scroll_delta = 0.0}
        self.volume_controller.on_key_press(&mut keys_down, mods);
        
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
        let mut dialog_list = std::mem::take(&mut self.dialogs);
        for d in dialog_list.iter_mut().rev() {
            // kb events
            keys_down.retain(|k| !d.on_key_press(k, &mods, self));
            keys_up.retain(|k| !d.on_key_release(k,  &mods, self));
            if !text.is_empty() && d.on_text(&text) {text = String::new()}

            // mouse events
            if mouse_moved {d.on_mouse_move(&mouse_pos, self)}
            if d.get_bounds().contains(mouse_pos) {
                mouse_down.retain(|button| !d.on_mouse_down(&mouse_pos, &button, &mods, self));
                mouse_up.retain(|button| !d.on_mouse_up(&mouse_pos, &button, &mods, self));
                if scroll_delta != 0.0 && d.on_mouse_scroll(&scroll_delta, self) {scroll_delta = 0.0}

                mouse_down.clear();
                mouse_up.clear();
            }
            d.update(self)
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
        self.cursor_manager.update(elapsed as f64, &mut self.window);


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
                    if mouse_moved {manager.mouse_move(mouse_pos)}
                    for btn in mouse_down {manager.mouse_down(btn)}
                    for btn in mouse_up {manager.mouse_up(btn)}
                    if scroll_delta != 0.0 {manager.mouse_scroll(scroll_delta)}

                    // kb
                    for k in keys_down.iter() {manager.key_down(*k, mods)}
                    for k in keys_up.iter() {manager.key_up(*k)}
                    if text.len() > 0 {
                        manager.on_text(&text, &mods)
                    }

                    // controller
                    for (c, b) in controller_down {
                        manager.controller_press(&c, b);
                    }
                    for (c, b) in controller_up {
                        manager.controller_release(&c, b);
                    }
                    for (c, b) in controller_axis {
                        manager.controller_axis(&c, b);
                    }


                    // update, then check if complete
                    manager.update();
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
                                Database::save_score(&score);
                                match save_replay(&replay, &score) {
                                    Ok(_)=> trace!("replay saved ok"),
                                    Err(e) => NotificationManager::add_error_notification("error saving replay", e),
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
                let mut menu = menu.lock();

                // menu input events

                // clicks
                for b in mouse_down { 
                    // game.start_map() can happen here, which needs &mut self
                    menu.on_click(mouse_pos, b, mods, self);
                }
                for b in mouse_up { 
                    // game.start_map() can happen here, which needs &mut self
                    menu.on_click_release(mouse_pos, b, self);
                }

                // mouse move
                if mouse_moved {menu.on_mouse_move(mouse_pos, self)}
                // mouse scroll
                if scroll_delta.abs() > 0.0 {menu.on_scroll(scroll_delta, self)}


                // TODO: this is temp
                if keys_up.contains(&Key::M) && mods.ctrl {self.add_dialog(Box::new(ModDialog::new()))}
                // TODO: this too
                if keys_up.contains(&Key::S) && mods.ctrl {self.add_dialog(Box::new(SkinSelect::new()))}

                // check keys down
                for key in keys_down {menu.on_key_press(key, self, mods)}
                // check keys up
                for key in keys_up {menu.on_key_release(key, self)}


                // controller
                for (c, b) in controller_down {
                    menu.controller_down(self, &c, b);
                }
                for (c, b) in controller_up {
                    menu.controller_up(self, &c, b);
                }
                for (c, ad) in controller_axis {
                    menu.controller_axis(self, &c, ad);
                }


                // check text
                if text.len() > 0 {menu.on_text(text)}

                // window focus change
                if let Some(has_focus) = window_focus_changed {
                    menu.on_focus_change(has_focus, self);
                }

                menu.update(self);
            }

            GameState::Spectating(manager) => {   
                manager.update(self);

                if mouse_moved {manager.mouse_move(mouse_pos, self)}
                for btn in mouse_down {manager.mouse_down(mouse_pos, btn, mods, self)}
                for btn in mouse_up {manager.mouse_up(mouse_pos, btn, mods, self)}
                if scroll_delta != 0.0 {manager.mouse_scroll(scroll_delta, self)}

                for k in keys_down.iter() {manager.key_down(*k, mods, self)}
                for k in keys_up.iter() {manager.key_up(*k, mods, self)}
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
                get_settings!().save();
                self.window.set_should_close(true);
            }

            _ => {
                // if the mode is being changed, clear all shapes, even ones with a lifetime
                self.clear_render_queue(true);

                // if the old state is a menu, tell it we're changing
                if let GameState::InMenu(menu) = &current_state {
                    menu.lock().on_change(false)
                }

                // let cloned_mode = self.queued_mode.clone();
                // self.threading.spawn(async move {
                //     online_manager.lock().await.discord.change_status(cloned_mode);
                //     OnlineManager::set_action(online_manager, UserAction::Leaving, String::new()).await;
                // });

                match &mut self.queued_state {
                    GameState::Ingame(manager) => {
                        let m = {
                            manager.start();
                            manager.metadata.clone()
                        };

                        self.set_background_beatmap(&m);
                        let text = format!("Playing {}-{}[{}]", m.artist, m.title, m.version);
                        OnlineManager::set_action(UserAction::Ingame, text, m.mode);
                    },
                    GameState::InMenu(_) => {
                        if let GameState::InMenu(menu) = &self.current_state {
                            if menu.lock().get_name() == "pause" {
                                if let Some(song) = Audio::get_song() {
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
                    GameState::InMenu(menu) if menu.lock().get_name() == "pause" => do_transition = false,
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
                        menu.lock().on_change(true);
                    }
                }
            }
        }

        // update the notification manager
        NOTIFICATION_MANAGER.lock().update();


        if let Ok(manager) = &mut ONLINE_MANAGER.try_write() {
            manager.do_game_things(self);
        }
        
        // if timer.elapsed().as_secs_f32() * 1000.0 > 1.0 {
        //     debug!("update took a while: {}", timer.elapsed().as_secs_f32() * 1000.0);
        // }
    }

    fn render(&mut self, args: RenderArgs) {
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
            GameState::Ingame(manager) => manager.draw(args, &mut self.render_queue),
            GameState::InMenu(menu) => {
                let mut lock = menu.lock();
                self.render_queue.extend(lock.draw(args));
                if lock.get_name() == "main_menu" {
                    draw_bg_dim = false;
                }
            },
            GameState::Spectating(manager) => manager.draw(args, &mut self.render_queue),
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
                (GameState::None, Some(GameState::InMenu(menu))) => self.render_queue.extend(menu.lock().draw(args)),
                _ => {}
            }
        }

        // draw any dialogs
        let mut dialog_list = std::mem::take(&mut self.dialogs);
        let mut current_depth = -50_000_000.0;
        const DIALOG_DEPTH_DIFF:f64 = 50.0;
        for d in dialog_list.iter_mut() { //.rev() {
            d.draw(&args, &current_depth, &mut self.render_queue);
            current_depth += DIALOG_DEPTH_DIFF;
        }
        self.dialogs = dialog_list;

        // volume control
        self.render_queue.extend(self.volume_controller.draw(args));

        // draw fps's
        self.fps_display.draw(&mut self.render_queue);
        self.update_display.draw(&mut self.render_queue);
        // self.input_update_display.draw(&mut self.render_queue);

        // draw the notification manager
        NOTIFICATION_MANAGER.lock().draw(&mut self.render_queue);

        // draw cursor
        // let mouse_pressed = self.input_manager.mouse_buttons.len() > 0 
        //     || self.input_manager.key_down(settings.standard_settings.left_key)
        //     || self.input_manager.key_down(settings.standard_settings.right_key);
        self.cursor_manager.draw(&mut self.render_queue);

        // sort the queue here (so it only needs to be sorted once per frame, instead of every time a shape is added)
        self.render_queue.sort_by(|a, b| b.get_depth().partial_cmp(&a.get_depth()).unwrap());


        // TODO: use this for snipping
        // // actually draw everything now
        // let mut orig_c = self.graphics.draw_begin(args.viewport());

        // graphics::clear(GFX_CLEAR_COLOR.into(), &mut self.graphics);
        // for i in self.render_queue.iter_mut() {
        //     let mut drawstate_changed = false;
        //     let c = if let Some(ic) = i.get_context() {
        //         drawstate_changed = true;
        //         // debug!("ic: {:?}", ic);
        //         self.graphics.draw_end();
        //         self.graphics.draw_begin(args.viewport());
        //         self.graphics.use_draw_state(&ic.draw_state);
        //         ic
        //     } else {
        //         orig_c
        //     };
            
        //     // self.graphics.use_draw_state(&c.draw_state);
        //     if i.get_spawn_time() == 0 {i.set_spawn_time(elapsed)}
        //     i.draw(&mut self.graphics, c);

        //     if drawstate_changed {
        //         self.graphics.draw_end();
        //         orig_c = self.graphics.draw_begin(args.viewport());
        //         self.graphics.use_draw_state(&orig_c.draw_state);
        //     }
        // }
        // self.graphics.draw_end();

        // TODO: dont use this for snipping
        let queue = self.render_queue.as_mut_slice();
        self.graphics.draw(args.viewport(), |c, g| {
            graphics::clear(GFX_CLEAR_COLOR.into(), g);
            for i in queue.as_mut() {
                i.draw(g, c);
            }
        });
        
        self.clear_render_queue(false);
        self.fps_display.increment();


        // if timer.elapsed().as_secs_f32() * 1000.0 > 1.0 {
        //     debug!("render took a while: {}", timer.elapsed().as_secs_f32() * 1000.0);
        // }
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
    pub fn set_background_beatmap(&mut self, beatmap:&BeatmapMeta) {
        // let mut helper = BenchmarkHelper::new("loaad image");

        self.background_image = load_image(&beatmap.image_filename, false);

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
        


        // let settings = opengl_graphics::TextureSettings::new();
        // // helper.log("settings made", true);
        // let buf: Vec<u8> = match std::fs::read(&beatmap.image_filename) {
        //     Ok(buf) => buf,
        //     Err(_) => {
        //         self.background_image = None;
        //         return;
        //     }
        // };

        // // let buf = file.unwrap();
        // // helper.log("file read", true);

        // let img = image::load_from_memory(&buf).unwrap();
        // // helper.log("image created", true);
        // let img = img.into_rgba8();
        // // helper.log("format converted", true);
        
        // let tex = opengl_graphics::Texture::from_image(&img, &settings);
        // // helper.log("texture made", true);

        // self.background_image = Some(Image::new(Vector2::zero(), f64::MAX, tex, window_size()));
        // // helper.log("background set", true);

        // // match opengl_graphics::Texture::from_path(beatmap.image_filename.clone(), &settings) {
        // //     Ok(tex) => self.background_image = Some(Image::new(Vector2::zero(), f64::MAX, tex, window_size())),
        // //     Err(e) => {
        // //         error!("Error loading beatmap texture: {}", e);
        // //         self.background_image = None; //TODO!: use a known good background image
        // //     },
        // // }
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
