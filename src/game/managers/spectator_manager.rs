use crate::prelude::*;

const BANNER_DEPTH: f64 = -99000.0;
const BANNER_WPADDING:f64 = 5.0;


/// how long of a buffer should we have? (ms)
const SPECTATOR_BUFFER_OK_DURATION:f32 = 500.0;

pub struct SpectatorManager {
    pub frames: SpectatorFrames, 
    pub state: SpectatorState, 
    pub game_manager: Option<IngameManager>,
    score_menu: Option<ScoreMenu>,

    /// what time we have data for
    /// ie, up to what time we can show gameplay
    pub good_until: f32,
    pub map_length: f32,

    /// list of id,username for specs
    pub spectator_cache: HashMap<u32, String>,

    /// list
    buffered_score_frames: Vec<(f32, Score)>
}
impl SpectatorManager {
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            state: SpectatorState::None,
            game_manager: None,
            good_until: 0.0,
            map_length: 0.0,
            spectator_cache: HashMap::new(),
            score_menu: None,
            buffered_score_frames: Vec::new()
        }
    }

    pub fn update(&mut self, game: &mut Game) {
        // (try to) read pending data from the online manager
        match ONLINE_MANAGER.try_lock() {
            Ok(mut online_manager) => self.frames.extend(online_manager.get_pending_spec_frames()),
            Err(e) => println!("[SpectatorManager::update] failed to lock online manager, {}", e),
        }

        if let Some(menu) = &self.score_menu {
            if menu.should_close {
                self.score_menu = None
            }
        }

        // check all incoming frames
        for (time, frame) in std::mem::take(&mut self.frames) {
            self.good_until = self.good_until.max(time as f32);

            println!("[Spec] packet: {:?}", frame);
            match frame {
                SpectatorFrameData::Play { beatmap_hash, mode, mods } => {
                    self.start_game(game, beatmap_hash, mode, mods, 0.0)
                }

                SpectatorFrameData::Pause => {
                    println!("[Spec] pause");
                    self.state = SpectatorState::Paused;
                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.pause();
                    }
                }
                SpectatorFrameData::UnPause => {
                    println!("[Spec] unpause");
                    self.state = SpectatorState::Watching;
                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.start();
                    }
                }
                SpectatorFrameData::Buffer => {/*nothing to handle here*/},
                SpectatorFrameData::SpectatingOther { .. } => {
                    NotificationManager::add_text_notification("Host speccing someone", 2000.0, Color::BLUE);
                }
                SpectatorFrameData::ReplayFrame { frame } => {
                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.replay.frames.push((time as f32, frame))
                    }
                }
                SpectatorFrameData::ScoreSync { score } => {
                    // received score update
                    println!("[Spec] got score update");
                    self.buffered_score_frames.push((time as f32, score));
                    // we should buffer these, and check the time. 
                    // if the time is at the score time, we should update our score, 
                    // as this score is probably more accurate, or at the very least will update new spectators
                }

                SpectatorFrameData::ChangingMap => {
                    println!("[Spec] host changing maps");
                    self.state = SpectatorState::MapChanging;

                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.pause()
                    }
                }

                SpectatorFrameData::PlayingResponse { user_id, beatmap_hash, mode, mods, current_time } => {
                    let self_id;
                    // cant do .blocking_lock() because it spawns a runtime?
                    loop {
                        match ONLINE_MANAGER.try_lock() {
                            Ok(m) => {
                                self_id = m.user_id;
                                break;
                            }
                            Err(_) => {}
                        }
                    }

                    if user_id == self_id {
                        self.start_game(game, beatmap_hash, mode, mods, current_time)
                    }
                }
                SpectatorFrameData::Unknown => {
                    // uh oh
                },
            }
        }
        
        // check our current state
        match &self.state {
            SpectatorState::None => {
                // in this case, the user should really be allowed to browse menus etc in the mean time. we might have to meme this
                if let Some(menu) = self.score_menu.as_mut() {
                    menu.update(game);

                    if menu.should_close {
                        self.score_menu = None;
                    }
                }
            }
            SpectatorState::Buffering => {
                if let Some(manager) = self.game_manager.as_mut() {
                    // buffer twice as long as we need
                    let buffer_duration = (manager.time() + SPECTATOR_BUFFER_OK_DURATION * 2.0).clamp(0.0, self.map_length);

                    if self.good_until >= buffer_duration {
                        self.state = SpectatorState::Watching;
                        println!("[Spec] no longer buffering");
                        manager.start();
                    } else {
                        // println!("[Spec] buffering");
                    }
                }
            }
            SpectatorState::Watching => {
                // currently watching someone
                if let Some(manager) = self.game_manager.as_mut() {
                    manager.update();

                    let manager_time = manager.time();
                    self.buffered_score_frames.retain(|(time, score)| {
                        if manager_time <= *time {
                            manager.score = score.to_owned();
                            false
                        } else {
                            true
                        }
                    });
                    
                    let buffer_duration = (manager.time() + SPECTATOR_BUFFER_OK_DURATION * 2.0).clamp(0.0, self.map_length);
                    if self.good_until < buffer_duration {
                        self.state = SpectatorState::Buffering;
                        println!("[Spec] starting buffer");
                        manager.pause();
                    }

                    if manager.completed || manager.time() >= self.map_length {
                        // if we have a score frame we havent dealt with yet, its most likely the score frame sent once the map has ended
                        if self.buffered_score_frames.len() > 0 {
                            manager.score = self.buffered_score_frames.last().unwrap().clone().1;
                        }
                        let mut score_menu = ScoreMenu::new(&manager.score, manager.metadata.clone());
                        score_menu.dont_do_menu = true;
                        self.score_menu = Some(score_menu);

                        self.state = SpectatorState::None;
                        self.game_manager = None;
                    }
                }
            }
            SpectatorState::Paused => {},
            SpectatorState::MapChanging => {},
        }
    }

    fn start_game(&mut self, game:&mut Game, beatmap_hash:String, mode:PlayMode, mods:String, current_time:f32) {
        self.good_until = 0.0;
        self.map_length = 0.0;
        self.buffered_score_frames.clear();
        // user started playing a map
        println!("[Spec] Host started playing map");

        let mods:ModManager = serde_json::from_str(&mods).unwrap();
        // find the map
        let mut beatmap_manager = BEATMAP_MANAGER.lock();
        match beatmap_manager.get_by_hash(&beatmap_hash) {
            Some(map) => {
                beatmap_manager.set_current_beatmap(game, &map, false, false);
                match manager_from_playmode(mode, &map) {
                    Ok(manager) => {
                        // remove score menu
                        self.score_menu = None;
                        // set our game manager
                        self.game_manager = Some(manager);

                        // need a mutable reference
                        let m = self.game_manager.as_mut().unwrap();
                        m.apply_mods(mods);
                        m.replaying = true;
                        m.on_start = Box::new(move |manager| {
                            println!("[Spec] jumping to time {}", current_time);
                            manager.jump_to_time(current_time.max(0.0), current_time > 0.0);
                        });
                        m.start();

                        self.map_length = m.end_time;
                        self.state = SpectatorState::Watching;
                    }
                    Err(e) => NotificationManager::add_error_notification("Error loading spec beatmap", e)
                }
            }
            
            // user doesnt have beatmap
            None => NotificationManager::add_text_notification("You do not have the map!", 2000.0, Color::RED)
        }
    }

    pub fn draw(&mut self, args: RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.draw(args, list)
        }

        // draw score menu
        if let Some(menu) = self.score_menu.as_mut() {
            list.extend(menu.draw(args))
        }
        
        // draw spectator banner
        match &self.state {
            SpectatorState::None => {
                if self.score_menu.is_none() {
                    draw_banner("Waiting for Host", list);
                }
            }
            SpectatorState::Watching => {}
            SpectatorState::Buffering => draw_banner("Buffering", list),
            SpectatorState::Paused => draw_banner("Host Paused", list),
            SpectatorState::MapChanging => draw_banner("Host Changing Map", list),
        }
    }

    pub fn mouse_scroll(&mut self, delta: f64, game:&mut Game) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.mouse_scroll(delta)
        }
        
        // update score menu
        if let Some(menu) = self.score_menu.as_mut() {
            menu.on_scroll(delta, game)
        }
    }
    pub fn mouse_move(&mut self, pos:Vector2, game:&mut Game) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.mouse_move(pos)
        }
        
        // update score menu
        if let Some(menu) = self.score_menu.as_mut() {
            menu.on_mouse_move(pos, game)
        }
    }
    pub fn mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.mouse_down(button);
        }

        // update score menu
        if let Some(menu) = self.score_menu.as_mut() {
            menu.on_click(pos, button, mods, game)
        }
    }
    pub fn mouse_up(&mut self, _pos:Vector2, button:MouseButton, _mods:KeyModifiers, _game:&mut Game) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.mouse_up(button)
        }
    }

    pub fn key_down(&mut self, key:piston::Key, mods:KeyModifiers, game:&mut Game) {
        // check if we need to close something
        if key == piston::Key::Escape {
            // if the score menu is open, close it and leave.
            if self.score_menu.is_some() {
                self.score_menu = None;
                return;
            }

            let menu = game.menus.get("main").unwrap().clone();
            game.queue_state_change(GameState::InMenu(menu));
            // resume song if paused

            #[cfg(feature="bass_audio")]
            if let Some(song) = Audio::get_song() {
                if song.get_playback_state() == Ok(PlaybackState::Paused) {
                    let _ = song.play(false);
                }
            }
        }


        // update score menu
        if let Some(manager) = self.game_manager.as_mut() {
            manager.key_down(key, mods)
        }

        // update score menu
        if let Some(menu) = self.score_menu.as_mut() {
            menu.on_key_press(key, game, mods);
        }
    }
    pub fn key_up(&mut self, key:piston::Key, _mods:KeyModifiers, game:&mut Game) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.key_up(key)
        }

        // update score menu
        if let Some(menu) = self.score_menu.as_mut() {
            menu.on_key_release(key, game);
        }
    }
}

// when the manager is dropped, tell the server we stopped spectating
impl Drop for SpectatorManager {
    fn drop(&mut self) {
        OnlineManager::stop_spectating();
    }
}


fn draw_banner(text:&str, list: &mut Vec<Box<dyn Renderable>>) {
    let window_size = Settings::window_size();
    let font = get_font("main");

    let mut offset_text = Text::new(
        Color::BLACK,
        BANNER_DEPTH,
        Vector2::zero(), // centered anyways
        32,
        text.to_owned(),
        font.clone()
    );
    
    let text_width = offset_text.measure_text().x + BANNER_WPADDING;
    // center
    let rect = Rectangle::bounds_only(
        Vector2::new((window_size.x - text_width) / 2.0, window_size.y * 1.0/3.0), 
        Vector2::new( text_width + BANNER_WPADDING, 64.0)
    );
    offset_text.center_text(rect);
    // add
    list.push(visibility_bg(rect.pos, rect.size, BANNER_DEPTH + 10.0));
    list.push(Box::new(offset_text));
}
