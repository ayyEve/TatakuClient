use std::path::Path;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use tokio::runtime::{Builder, Runtime};
use glfw_window::GlfwWindow as AppWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{Window,input::*, event_loop::*, window::WindowSettings};

use taiko_rs_common::types::UserAction;
use crate::game::BenchmarkHelper;
use crate::gameplay::{Beatmap, Replay, KeyPress};
use crate::databases::{save_all_scores, save_score};
use crate::{HIT_AREA_RADIUS, HIT_POSITION, WINDOW_SIZE, SONGS_DIR, menu::*};
use crate::render::{HalfCircle, Rectangle, Renderable, Text, Color, Border};
use crate::game::{InputManager, Settings, get_font, Vector2, online::USER_ITEM_SIZE, online::OnlineManager};

/// how long should the volume thing be displayed when changed
const VOLUME_CHANGE_DISPLAY_TIME:u64 = 2000;
/// background color
const GFX_CLEAR_COLOR:Color = Color::WHITE;
/// how long do transitions between gamemodes last?
const TRANSITION_TIME:u64 = 500;
/// how long should the drum buttons last for?
const DRUM_LIFETIME_TIME:u64 = 100;

pub struct Game {
    render_queue: Vec<Box<dyn Renderable>>,

    pub window: AppWindow,
    pub graphics: GlGraphics,
    pub input_manager: InputManager,
    pub online_manager: Arc<tokio::sync::Mutex<OnlineManager>>,
    pub threading: Runtime,
    
    pub menus: HashMap<&'static str, Arc<Mutex<Box<dyn Menu>>>>,
    pub game_start: Instant,
    pub current_mode: GameMode,
    pub queued_mode: GameMode,

    // volume change things 
    // maybe move these to another object? not really necessary but might help clean up code a bit maybe
    //NOTE: these cant be changed to shape lifetimes as the user might potentially change the selected volume
    /// 0-2, 0 = master, 1 = effect, 2 = music
    vol_selected_index: u8, 
    ///when the volume was changed, or the selected index changed
    vol_selected_time: u64,

    // fps
    fps_count: u32,
    fps_timer: Instant,
    fps_last: f32,
    update_count: u32,
    update_timer: Instant,
    update_last: f32,

    // transition
    transition: Option<GameMode>,
    transition_last: Option<GameMode>,
    transition_timer: u64,

    // user list
    show_user_list: bool
}
impl Game {
    pub fn new() -> Game {
        let mut game_init_benchmark = BenchmarkHelper::new("game_init");

        let opengl = OpenGL::V3_2;

        let mut window: AppWindow = WindowSettings::new("Taiko", [WINDOW_SIZE.x, WINDOW_SIZE.y])
            .graphics_api(opengl)
            .resizable(false)
            .build()
            .unwrap();
        game_init_benchmark.log("window created", true);

        set_icon(&mut window);
        game_init_benchmark.log("window icon set (jk for now)", true);

        let graphics = GlGraphics::new(opengl);
        game_init_benchmark.log("graphics created", true);

        let input_manager = InputManager::new();
        game_init_benchmark.log("input manager created", true);

        let online_manager = Arc::new(tokio::sync::Mutex::new(OnlineManager::new()));
        game_init_benchmark.log("online manager created", true);

        let threading = Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        game_init_benchmark.log("threading created", true);

        let mut g = Game {
            current_mode: GameMode::None,
            queued_mode: GameMode::None,
            window,
            graphics,
            threading,
            input_manager,
            online_manager,
            render_queue: Vec::new(),
            game_start: Instant::now(),

            menus: HashMap::new(),

            // vol things
            vol_selected_index: 0,
            vol_selected_time: 0,

            // fps
            fps_timer: Instant::now(),
            fps_last: 0.0,
            fps_count: 0,
            update_timer: Instant::now(),
            update_last: 0.0,
            update_count: 0,

            // transition
            transition: None,
            transition_last: None,
            transition_timer: 0,

            show_user_list: false
        };
        game_init_benchmark.log("game created", true);


        //region == menu setup ==
        // main menu
        let main_menu:Box<dyn Menu> = Box::new(MainMenu::new());
        let main_menu = Arc::new(Mutex::new(main_menu));
        g.menus.insert("main", main_menu.clone());

        game_init_benchmark.log("main menu created", true);

        // setup beatmap select menu
        let beatmap_menu:Box<dyn Menu> = Box::new(BeatmapSelectMenu::new());
        let beatmap_menu = Arc::new(Mutex::new(beatmap_menu));
        g.menus.insert("beatmap", beatmap_menu.clone());

        game_init_benchmark.log("beatmap menu created", true);

        // setup settings menu
        let settings_menu:Box<dyn Menu> = Box::new(SettingsMenu::new());
        let settings_menu = Arc::new(Mutex::new(settings_menu));
        g.menus.insert("settings", settings_menu.clone());
        game_init_benchmark.log("settings menu created", true);

        // set current mode to main menu
        g.queue_mode_change(GameMode::InMenu(main_menu.clone()));

        g.init();
        game_init_benchmark.log("game initialized", true);

        g
    }

    pub fn init(&mut self) {
        let clone = self.online_manager.clone();
        self.threading.spawn(async move {
            loop {
                OnlineManager::start(clone.clone()).await;
                tokio::time::sleep(Duration::from_millis(1_000)).await;
            }
        });
    }
    pub fn game_loop(mut self) {
        // input and rendering thread
        let mut events = Events::new(EventSettings::new()); //.lazy(true));

        //TODO: load this from settings
        events.set_max_fps(144);
        events.set_ups(1_000);

        #[cfg(feature = "unlimited_fps")] {
            events.set_max_fps(10_000);
            events.set_ups(10_000);
        }

        while let Some(e) = events.next(&mut self.window) {
            if let Some(args) = e.update_args() {self.update(args.dt*1000.0)}
            if let Some(args) = e.render_args() {self.render(args)}
            self.input_manager.handle_events(e.clone());
            // e.resize(|args| println!("Resized '{}, {}'", args.window_size[0], args.window_size[1]));
        }
    }

    pub fn queue_mode_change(&mut self, mode:GameMode) {self.queued_mode = mode}

    fn update(&mut self, _delta:f64) {
        self.update_count += 1;
        let arc = Arc::new(Mutex::new(self));
        let clone = arc.clone();
        let mut current_mode = clone.lock().unwrap().current_mode.clone().to_owned();
        let elapsed = clone.lock().unwrap().game_start.elapsed().as_millis() as u64;

        //TODO: move these fonctions in input manager, like get_text()
        // check input events
        let mouse_pos = clone.lock().unwrap().input_manager.mouse_pos;
        // clicks
        let mut mouse_buttons = clone.lock().unwrap().input_manager.mouse_buttons.clone();
        clone.lock().unwrap().input_manager.mouse_buttons.clear();
        // mouse move
        let mouse_moved = clone.lock().unwrap().input_manager.mouse_moved.clone();
        clone.lock().unwrap().input_manager.mouse_moved = false;
        // mouse scroll
        let mut scroll_delta = clone.lock().unwrap().input_manager.scroll_delta.clone();
        clone.lock().unwrap().input_manager.scroll_delta = 0.0;
        // keys down 
        let keys = clone.lock().unwrap().input_manager.all_down_once();
        let mods = clone.lock().unwrap().input_manager.get_key_mods();
        let text = clone.lock().unwrap().input_manager.get_text();
        let window_focus_changed = clone.lock().unwrap().input_manager.get_changed_focus();


        
        // users list
        if clone.lock().unwrap().show_user_list {
            if let Ok(om) = clone.lock().unwrap().online_manager.try_lock() {
                for (_, user) in &om.users.clone() {
                    if let Ok(mut u) = user.try_lock() {
                        if mouse_moved {u.on_mouse_move(mouse_pos)}
                        if mouse_buttons.len() > 0 {mouse_buttons.retain(|button| !u.on_click(mouse_pos, button.clone()))}
                    }
                }
            }
        }


        // check for volume change
        let mut volume_changed = false;
        if mods.alt {
            let mut lock = clone.lock().unwrap();
            let mut settings = Settings::get_mut();
            let elapsed = lock.game_start.elapsed().as_millis() as u64;

            let mut delta:f32 = 0.0;
            if keys.contains(&Key::Right) {delta = 0.1;}
            if keys.contains(&Key::Left) {delta = -0.1;}
            if scroll_delta != 0.0 {
                delta = scroll_delta as f32 / 10.0;
                scroll_delta = 0.0;
            }
            
            // check volume changed, if it has, set the selected time to now
            if delta != 0.0 || keys.contains(&Key::Up) || keys.contains(&Key::Down) {
                // reset index back to 0 (master) if the volume hasnt been touched in a while
                if elapsed - lock.vol_selected_time > VOLUME_CHANGE_DISPLAY_TIME + 1000 {
                    lock.vol_selected_index = 0;
                }

                // find out what volume to edit, and edit it
                match lock.vol_selected_index {
                    0 => { // master
                        settings.master_vol += delta;
                        settings.master_vol = settings.master_vol.max(0.0).min(1.0);
                    },
                    1 => { // effect
                        settings.effect_vol += delta;
                        settings.effect_vol = settings.effect_vol.max(0.0).min(1.0);
                    },
                    2 => { // music
                        settings.music_vol += delta;
                        settings.music_vol = settings.music_vol.max(0.0).min(1.0);
                    },

                    _ => println!("lock.vol_selected_index out of bounds somehow")
                }

                volume_changed = true;
                lock.vol_selected_time = elapsed;
            }

            // if the volume thing is viewable, check for index selecting keys
            if elapsed - lock.vol_selected_time < VOLUME_CHANGE_DISPLAY_TIME {
                if keys.contains(&Key::Up) {
                    lock.vol_selected_index = (3+(lock.vol_selected_index as i8 - 1)) as u8 % 3;
                    lock.vol_selected_time = elapsed;
                }
                if keys.contains(&Key::Down) {
                    lock.vol_selected_index = (lock.vol_selected_index + 1) % 3;
                    lock.vol_selected_time = elapsed;
                }
            }
        }
        {
            let mut c = clone.lock().unwrap();
            // check if mouse clicked volume button
            if c.vol_selected_time > 0 && elapsed as f64 - (c.vol_selected_time as f64) < VOLUME_CHANGE_DISPLAY_TIME as f64 {
                if mouse_buttons.contains(&MouseButton::Left) || mouse_moved {
                    let window_size = WINDOW_SIZE;
                    let master_pos = window_size - Vector2::new(300.0, 90.0);
                    let effect_pos = window_size - Vector2::new(300.0, 60.0);
                    let music_pos = window_size - Vector2::new(300.0, 30.0);

                    if mouse_pos.x >= master_pos.x {
                        let mut modified = false;
                        if mouse_pos.y >= music_pos.y {
                            c.vol_selected_index = 2;
                            modified = true;
                        } else if mouse_pos.y >= effect_pos.y {
                            c.vol_selected_index = 1;
                            modified = true;
                        } else if mouse_pos.y >= master_pos.y {
                            c.vol_selected_index = 0;
                            modified = true;
                        }

                        // was just updated
                        if modified {
                            c.vol_selected_time = elapsed;
                            // remove left click from list as it was consumed by this
                            mouse_buttons.retain(|e| e == &MouseButton::Left);
                        }
                    }
                }
            }
        }

        if keys.contains(&Key::F8) {
            let mut c = clone.lock().unwrap();
            c.show_user_list = !c.show_user_list;
            println!("show user list: {}", c.show_user_list);
        }

        match current_mode {
            GameMode::Ingame(ref beatmap) => {
                let settings = Settings::get();
                let og_beatmap = beatmap;
                let mut lock = clone.lock().unwrap();
                let mut beatmap = beatmap.lock().unwrap();
                
                if keys.contains(&settings.left_kat) {
                    beatmap.hit(KeyPress::LeftKat);

                    let mut hit = HalfCircle::new(
                        Color::BLUE,
                        HIT_POSITION,
                        1.0,
                        HIT_AREA_RADIUS,
                        true
                    );
                    hit.set_lifetime(DRUM_LIFETIME_TIME);
                    lock.render_queue.push(Box::new(hit));
                }
                if keys.contains(&settings.left_don) {
                    beatmap.hit(KeyPress::LeftDon);

                    let mut hit = HalfCircle::new(
                        Color::RED,
                        HIT_POSITION,
                        1.0,
                        HIT_AREA_RADIUS,
                        true
                    );
                    hit.set_lifetime(DRUM_LIFETIME_TIME);
                    lock.render_queue.push(Box::new(hit));
                }
                if keys.contains(&settings.right_don) {
                    beatmap.hit(KeyPress::RightDon);

                    let mut hit = HalfCircle::new(
                        Color::RED,
                        HIT_POSITION,
                        1.0,
                        HIT_AREA_RADIUS,
                        false
                    );
                    hit.set_lifetime(DRUM_LIFETIME_TIME);
                    lock.render_queue.push(Box::new(hit));
                }
                if keys.contains(&settings.right_kat) {
                    beatmap.hit(KeyPress::RightKat);

                    let mut hit = HalfCircle::new(
                        Color::BLUE,
                        HIT_POSITION,
                        1.0,
                        HIT_AREA_RADIUS,
                        false
                    );
                    hit.set_lifetime(DRUM_LIFETIME_TIME);
                    lock.render_queue.push(Box::new(hit));
                }
                
                // pause button, or focus lost
                if keys.contains(&Key::Escape) {
                    // pause
                    beatmap.pause();
                    let menu = PauseMenu::new(og_beatmap.clone());
                    lock.queue_mode_change(GameMode::InMenu(Arc::new(Mutex::new(Box::new(menu)))));
                }
                if let Some(e) = window_focus_changed {
                    if !e { // window lost focus, pause
                        // pause
                        beatmap.pause();
                        let menu = PauseMenu::new(og_beatmap.clone());
                        lock.queue_mode_change(GameMode::InMenu(Arc::new(Mutex::new(Box::new(menu)))));
                    }
                }

                // offset adjust
                if keys.contains(&Key::Equals) {beatmap.increment_offset(5)}
                if keys.contains(&Key::Minus) {beatmap.increment_offset(-5)}

                // volume
                if volume_changed {beatmap.song.set_volume(settings.get_music_vol())}

                beatmap.update();

                if beatmap.completed {
                    // save score
                    save_score(beatmap.score.clone());
                    match save_all_scores() {
                        Ok(_) => println!("Scores saved successfully"),
                        Err(e) => println!("Failed to save scores! {}", e),
                    }


                    // submit score
                    #[cfg(feature = "online_scores")] 
                    {
                        let score_clone = beatmap.score.clone();
                        lock.threading.spawn(async move {
                            //TODO: do this async
                            println!("submitting score");
                            let mut writer = crate::game::SerializationWriter::new();
                            writer.write(score_clone);
                            let data = writer.data();
                            
                            let c = reqwest::Client::new();
                            let res = c.post("http://localhost:8000/score_submit")
                                .body(data)
                                .send().await;

                            match res {
                                Ok(_isgood) => {
                                    //TODO: do something with the response?
                                    println!("score submitted successfully");
                                },
                                Err(e) => println!("error submitting score: {}", e),
                            }
                        });
                    }

                    // show score menu
                    let menu = ScoreMenu::new(beatmap.score.clone(), Arc::new(Mutex::new(beatmap.clone())));
                    lock.queue_mode_change(GameMode::InMenu(Arc::new(Mutex::new(Box::new(menu)))));

                    // cleanup memory-hogs in the beatmap object
                    beatmap.cleanup();
                }
            }

            GameMode::Replaying(ref beatmap, ref replay, ref mut frame_index) => {
                let settings = Settings::get();
                let mut lock = clone.lock().unwrap();
                let mut beatmap = beatmap.lock().unwrap();

                {
                    let time = beatmap.time();
                    // read any frames that need to be read
                    loop {
                        if *frame_index as usize >= replay.presses.len() {break}
                        
                        let (press_time, pressed) = replay.presses[*frame_index as usize];
                        if press_time > time {break}

                        beatmap.hit(pressed);
                        match pressed {
                            crate::gameplay::KeyPress::LeftKat => {
                                let mut hit = HalfCircle::new(
                                    Color::BLUE,
                                    HIT_POSITION,
                                    1.0,
                                    HIT_AREA_RADIUS,
                                    true
                                );
                                hit.set_lifetime(DRUM_LIFETIME_TIME);
                                lock.render_queue.push(Box::new(hit));
                            },
                            crate::gameplay::KeyPress::LeftDon => {
                                let mut hit = HalfCircle::new(
                                    Color::RED,
                                    HIT_POSITION,
                                    1.0,
                                    HIT_AREA_RADIUS,
                                    true
                                );
                                hit.set_lifetime(DRUM_LIFETIME_TIME);
                                lock.render_queue.push(Box::new(hit));
                            },
                            crate::gameplay::KeyPress::RightDon => {
                                let mut hit = HalfCircle::new(
                                    Color::RED,
                                    HIT_POSITION,
                                    1.0,
                                    HIT_AREA_RADIUS,
                                    false
                                );
                                hit.set_lifetime(DRUM_LIFETIME_TIME);
                                lock.render_queue.push(Box::new(hit));
                            },
                            crate::gameplay::KeyPress::RightKat => {
                                let mut hit = HalfCircle::new(
                                    Color::BLUE,
                                    HIT_POSITION,
                                    1.0,
                                    HIT_AREA_RADIUS,
                                    false
                                );
                                hit.set_lifetime(DRUM_LIFETIME_TIME);
                                lock.render_queue.push(Box::new(hit));
                            },
                        }

                        *frame_index += 1;
                    }
                }
                
                // pause button, or focus lost
                if keys.contains(&Key::Escape) {
                    beatmap.reset();
                    let menu = lock.menus.get("beatmap").unwrap().clone();
                    lock.queue_mode_change(GameMode::InMenu(menu));
                }

                // offset adjust
                if keys.contains(&Key::Equals) {
                    beatmap.increment_offset(5);
                }
                if keys.contains(&Key::Minus) {
                    beatmap.increment_offset(-5);
                }

                // volume
                if volume_changed {beatmap.song.set_volume(settings.get_music_vol())}

                beatmap.update();

                if beatmap.completed {
                    // show score menu
                    let menu = ScoreMenu::new(beatmap.score.clone(), Arc::new(Mutex::new(beatmap.clone())));
                    lock.queue_mode_change(GameMode::InMenu(Arc::new(Mutex::new(Box::new(menu)))));

                    beatmap.cleanup();
                }
            }
            
            GameMode::InMenu(ref menu) => {
                let mut menu = menu.lock().unwrap();

                // menu input events
                // vol
                if volume_changed {menu.on_volume_change()}

                // clicks
                if mouse_buttons.len() > 0 {
                    for b in mouse_buttons { 
                        // game.start_map() can happen here, which needs &mut self
                        menu.on_click(mouse_pos, b, arc.clone());
                    }
                }
                // mouse move
                if mouse_moved {menu.on_mouse_move(mouse_pos, arc.clone())}
                // mouse scroll
                if scroll_delta.abs() > 0.0 {menu.on_scroll(scroll_delta, arc.clone())}

                //check keys down
                for key in keys {menu.on_key_press(key, arc.clone(), mods)}
                //TODO: check keys up (or remove it, probably not used anywhere)

                // check text
                if text.len() > 0 {menu.on_text(text);}

                // window focus change
                if let Some(has_focus) = window_focus_changed {
                    menu.on_focus_change(has_focus, arc.clone());
                }

                menu.update(arc.clone());
            }

            GameMode::None => {
                // might be transitioning
                let mut lock = clone.lock().unwrap();

                if lock.transition.is_some().clone() && elapsed - lock.transition_timer > TRANSITION_TIME / 2 {
                    let trans = lock.transition.as_ref().unwrap().clone();
                    lock.queue_mode_change(trans);
                    lock.transition = None;
                    lock.transition_timer = elapsed;
                }
            }

            _ => {}
        }
        
        // update game mode
        let mut unlocked = clone.lock().unwrap();
        match &unlocked.queued_mode {
            GameMode::None => {
                unlocked.current_mode = current_mode;
            }
            GameMode::Closing => {
                Settings::get().save();
                unlocked.window.set_should_close(true);
            }

            _ => {
                // if the mode is being changed, clear all shapes, even ones with a lifetime
                unlocked.clear_render_queue(true);

                let online_manager = unlocked.online_manager.clone();
                match &unlocked.queued_mode {
                    GameMode::Ingame(map) => {
                        let m = map.lock().unwrap().metadata.clone();
                        let text = format!("{}-{}[{}]\n{}",m.artist,m.title,m.version,map.lock().unwrap().hash.clone());

                        unlocked.threading.spawn(async move {
                            OnlineManager::set_action(online_manager, UserAction::Ingame, text).await;
                        });
                    },
                    GameMode::InMenu(_) => {
                        unlocked.threading.spawn(async move {
                            OnlineManager::set_action(online_manager, UserAction::Idle, String::new()).await;
                        });
                    },
                    GameMode::Closing => {
                        // send logoff
                        unlocked.threading.spawn(async move {
                            OnlineManager::set_action(online_manager, UserAction::Leaving, String::new()).await;
                        });
                    }
                    _ => {}
                }


                let mut do_transition = true;

                match &unlocked.current_mode {
                    GameMode::None => do_transition = false,
                    GameMode::InMenu(menu) if menu.lock().unwrap().get_name() == "pause" => do_transition = false,
                    _ => {}
                }
                
                // if let GameMode::InMenu(menu) = &unlocked.current_mode {
                //     menu.lock().unwrap().on_change(false);
                // }

                if do_transition {
                    // do a transition
                    let qm = &unlocked.queued_mode;
                    unlocked.transition = Some(qm.clone());
                    unlocked.transition_timer = elapsed;
                    unlocked.transition_last = Some(unlocked.current_mode.clone());
                    unlocked.queued_mode = GameMode::None;
                    unlocked.current_mode = GameMode::None;
                } else {
                    // old mode was none, or was pause menu, transition to new mode
                    let mode = unlocked.queued_mode.clone();

                    unlocked.current_mode = mode.clone();
                    unlocked.queued_mode = GameMode::None;


                    if let GameMode::InMenu(menu) = &unlocked.current_mode {
                        menu.lock().unwrap().on_change(true);
                    }
                }
            }
        }
    }

    fn render(&mut self, args: RenderArgs) {
        let mut renderables: Vec<Box<dyn Renderable>> = Vec::new();
        let window_size = Vector2::new(args.window_size[0], args.window_size[1]);
        let settings = Settings::get();
        let elapsed = self.game_start.elapsed().as_millis() as u64;
        let font = get_font("main");

        // mode
        match &self.current_mode {
            GameMode::Ingame(beatmap) => {
                renderables.extend(beatmap.lock().unwrap().draw(args));
            }
            GameMode::Replaying(beatmap,_,_) => {
                renderables.extend(beatmap.lock().unwrap().draw(args));
            }
            GameMode::InMenu(ref menu) => {
                renderables.extend(menu.lock().unwrap().draw(args));
            }
            _ => {}
        }

        // transition
        if self.transition_timer > 0 && elapsed - self.transition_timer < TRANSITION_TIME {
            // probably transitioning

            // draw fade in rect
            let diff = elapsed as f64 - self.transition_timer as f64;

            let mut alpha = diff / (TRANSITION_TIME as f64 / 2.0);
            if self.transition.is_none() {
                alpha = 1.0 - diff / TRANSITION_TIME as f64;
            }

            renderables.push(Box::new(Rectangle::new(
                [0.0,0.0,0.0, alpha as f32].into(),
                -f64::MAX,
                Vector2::new(0.0, 0.0),
                Vector2::new(WINDOW_SIZE.x as f64, WINDOW_SIZE.y as f64),
                None
            )));

            // draw old mode
            if let GameMode::None = self.current_mode {
                if let Some(old_mode) = &self.transition_last {
                    match old_mode {
                        GameMode::InMenu(menu) => {
                            renderables.extend(menu.lock().unwrap().draw(args));
                        },

                        _ => {}
                    }
                }
            }
        }
        

        // users list
        if self.show_user_list {
            let mut counter = 0;
            
            if let Ok(om) = self.online_manager.try_lock() {
                for (_, user) in &om.users.clone() {
                    if let Ok(mut u) = user.try_lock() {

                        let x = if counter % 2 == 0 {0.0} else {USER_ITEM_SIZE.x};
                        let y = (counter - counter % 2) as f64 * USER_ITEM_SIZE.y;

                        counter += 1;
                        renderables.extend(u.draw(args, Vector2::new(x, y), -100.0));
                    }
                }
            }
        }


        // add the things we just made to the render queue
        self.render_queue.extend(renderables);

        // draw the volume things if needed
        if self.vol_selected_time > 0 && elapsed - self.vol_selected_time < VOLUME_CHANGE_DISPLAY_TIME {
            let b_size = Vector2::new(300.0, 100.0);
            let b = Rectangle::new(
                Color::WHITE,
                -7.0,
                window_size - b_size,
                b_size,
                Some(Border::new(Color::BLACK, 1.2))
            );
            self.render_queue.push(Box::new(b));

            // text 100px wide, bar 190px (10px padding)
            let border_padding = 10.0;
            let border_size = Vector2::new(200.0 - border_padding, 20.0);
            const TEXT_YOFFSET:f64 = -17.0; // bc font measuring broken
            
            // == master bar ==
            // text
            let mut master_text = Text::new(
                Color::BLACK,
                -9.0,
                window_size - Vector2::new(300.0, 90.0+TEXT_YOFFSET),
                20,
                "Master:".to_owned(),
                font.clone(),
            );
            // border
            let master_border = Rectangle::new(
                Color::TRANSPARENT_WHITE,
                -9.0,
                window_size - Vector2::new(border_size.x + border_padding, 90.0),
                border_size,
                Some(Border::new(Color::RED, 1.0))
            );
            // fill
            let master_fill = Rectangle::new(
                Color::BLUE,
                -8.0,
                window_size - Vector2::new(border_size.x + border_padding, 90.0),
                Vector2::new(border_size.x * settings.master_vol as f64, border_size.y),
                None
            );

            // == effects bar ==
            // text
            let mut effect_text = Text::new(
                Color::BLACK,
                -9.0,
                window_size - Vector2::new(300.0, 60.0+TEXT_YOFFSET),
                20,
                "Effects:".to_owned(),
                font.clone()
            );
            // border
            let effect_border = Rectangle::new(
                Color::TRANSPARENT_WHITE,
                -9.0,
                window_size - Vector2::new(border_size.x + border_padding, 60.0),
                border_size,
                Some(Border::new(Color::RED, 1.0))
            );
            // fill
            let effect_fill = Rectangle::new(
                Color::BLUE,
                -8.0,
                window_size - Vector2::new(border_size.x + border_padding, 60.0),
                Vector2::new(border_size.x * settings.effect_vol as f64, border_size.y),
                None
            );

            // == music bar ==
            // text
            let mut music_text = Text::new(
                Color::BLACK,
                -9.0,
                window_size - Vector2::new(300.0, 30.0+TEXT_YOFFSET),
                20,
                "Music:".to_owned(),
                font.clone()
            );
            // border
            let music_border = Rectangle::new(
                Color::TRANSPARENT_WHITE,
                -9.0,
                window_size - Vector2::new(border_size.x + border_padding, 30.0),
                border_size,
                Some(Border::new(Color::RED, 1.0))
            );
            // fill
            let music_fill = Rectangle::new(
                Color::BLUE,
                -8.0,
                window_size - Vector2::new(border_size.x + border_padding, 30.0),
                Vector2::new(border_size.x * settings.music_vol as f64, border_size.y),
                None
            );
            
            // highlight selected index
            match self.vol_selected_index {
                0 => {
                    master_text.color = Color::RED;
                },
                1 => {
                    effect_text.color = Color::RED;
                },
                2 => {
                    music_text.color = Color::RED;
                },
                _ => println!("self.vol_selected_index out of bounds somehow")
            }

            
            self.render_queue.push(Box::new(master_text));
            self.render_queue.push(Box::new(master_border));
            self.render_queue.push(Box::new(master_fill));

            self.render_queue.push(Box::new(effect_text));
            self.render_queue.push(Box::new(effect_border));
            self.render_queue.push(Box::new(effect_fill));

            self.render_queue.push(Box::new(music_text));
            self.render_queue.push(Box::new(music_border));
            self.render_queue.push(Box::new(music_fill));

            // vs
            // let a:[Box<dyn Renderable>; 9] = [
            //     Box::new(master_text),
            //     Box::new(master_border),
            //     Box::new(master_fill),
                
            //     Box::new(effect_text),
            //     Box::new(effect_border),
            //     Box::new(effect_fill),

            //     Box::new(music_text),
            //     Box::new(music_border),
            //     Box::new(music_fill),
            // ];
            // self.render_queue.extend(a);
        }


        // update fps var if needed
        let fps_elapsed = self.fps_timer.elapsed().as_micros() as f64 / 1000.0;
        if fps_elapsed >= 100.0 {
            self.fps_last = (self.fps_count as f64 / fps_elapsed * 1000.0) as f32;
            self.fps_timer = Instant::now();
            self.fps_count = 0;
        }
        // draw fps
        self.render_queue.push(Box::new(Text::new(
            Color::BLACK,
            -1.0,
            Vector2::new(0.0, 10.0),
            12,
            format!("{:.2}fps", self.fps_last),
            font.clone()
        )));
        // draw updates
        let updates_elapsed = self.update_timer.elapsed().as_micros() as f64 / 1000.0;
        if updates_elapsed >= 100.0 {
            self.update_last = (self.update_count as f64 / updates_elapsed * 1000.0) as f32;
            self.update_timer = Instant::now();
            self.update_count = 0;
        }
        // draw fps
        self.render_queue.push(Box::new(Text::new (
            Color::BLACK,
            -1.0,
            Vector2::new(0.0, 30.0),
            12,
            format!("{:.2} updates/s", self.update_last),
            font.clone()
        )));

        // sort the queue here (so it only needs to be sorted once per frame, instead of every time a shape is added)
        self.render_queue.sort_by(|a, b| b.get_depth().partial_cmp(&a.get_depth()).unwrap());

        // slice the queue because loops
        let queue = self.render_queue.as_mut_slice();
        self.graphics.draw(args.viewport(), |c, g| {
                graphics::clear(GFX_CLEAR_COLOR.into(), g);

                for i in queue.as_mut() {
                    if i.get_spawn_time() == 0 {i.set_spawn_time(elapsed);}
                    i.draw(g, c);
                }
            }
        );
        
        self.clear_render_queue(false);
        self.fps_count += 1;
    }

    pub fn clear_render_queue(&mut self, remove_all:bool) {
        if remove_all {
            self.render_queue.clear();
            return;
        }

        let elapsed = self.game_start.elapsed().as_millis() as u64;
        // only return items who's lifetime has expired
        self.render_queue.retain(|e| {
            let lifetime = e.get_lifetime();
            lifetime > 0 && elapsed - e.get_spawn_time() < lifetime
        });
    }


    /// extract all zips from the downloads folder into the songs folder. not a static function as it uses threading
    pub fn extract_all(&self) {
        let runtime = &self.threading;

        // check for new maps
        if let Ok(files) = std::fs::read_dir(crate::DOWNLOADS_DIR) {
            for file in files {
                if let Ok(filename) = file {
                    runtime.spawn(async move {

                        // unzip file into ./Songs
                        let file = std::fs::File::open(filename.path().to_str().unwrap()).unwrap();
                        let mut archive = zip::ZipArchive::new(file).unwrap();
                        
                        for i in 0..archive.len() {
                            let mut file = archive.by_index(i).unwrap();
                            let mut outpath = match file.enclosed_name() {
                                Some(path) => path,
                                None => continue,
                            };

                            let x = outpath.to_str().unwrap();
                            let y = format!("{}/{}/", SONGS_DIR, filename.file_name().to_str().unwrap().trim_end_matches(".osz"));
                            let z = &(y + x);
                            outpath = Path::new(z);

                            if (&*file.name()).ends_with('/') {
                                println!("File {} extracted to \"{}\"", i, outpath.display());
                                std::fs::create_dir_all(&outpath).unwrap();
                            } else {
                                println!("File {} extracted to \"{}\" ({} bytes)", i, outpath.display(), file.size());
                                if let Some(p) = outpath.parent() {
                                    if !p.exists() {std::fs::create_dir_all(&p).unwrap()}
                                }
                                let mut outfile = std::fs::File::create(&outpath).unwrap();
                                std::io::copy(&mut file, &mut outfile).unwrap();
                            }

                            // Get and Set permissions
                            // #[cfg(unix)] {
                            //     use std::os::unix::fs::PermissionsExt;

                            //     if let Some(mode) = file.unix_mode() {
                            //         fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
                            //     }
                            // }
                        }
                    
                        match std::fs::remove_file(filename.path().to_str().unwrap()) {
                            Ok(_) => {},
                            Err(e) => println!("error deleting file: {}", e),
                        }
                    });
                }
            }
        }
    }

}

/// tries to set the app window. does nothing if theres an issue
fn set_icon(_window:&mut AppWindow) {

    // // read file
    // if let Ok(img) =  image::open("") {

    //     // glfw::PixelImage

    //     window.window.set_icon_from_pixels(pain);
    // }
}

#[derive(Clone)]
pub enum GameMode {
    None, // use this as the inital game mode, but me sure to change it after
    Closing,
    Ingame(Arc<Mutex<Beatmap>>),
    InMenu(Arc<Mutex<Box<dyn Menu>>>),

    Replaying(Arc<Mutex<Beatmap>>, Replay, u64)
}
