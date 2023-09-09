use crate::prelude::*;
const BANNER_WPADDING:f32 = 5.0;

/// how long of a buffer should we have? (ms)
const SPECTATOR_BUFFER_OK_DURATION:f32 = 500.0;

pub struct SpectatorManager {
    pub frames: Vec<SpectatorFrame>, 
    pub state: SpectatorState, 
    pub game_manager: Option<IngameManager>,
    pub host_id: u32,
    pub host_username: String,
    score_menu: Option<ScoreMenu>,
    window_size: WindowSizeHelper,

    /// what time we have data for
    /// ie, up to what time we can show gameplay
    pub good_until: f32,
    pub map_length: f32,

    /// what is the current map's hash? 
    /// if this is Some and game_manager is None, we dont have the map
    pub current_map: Option<(Md5Hash, String, String, u16)>,

    /// list of id,username for other spectators
    pub spectator_cache: HashMap<u32, String>,

    /// list
    buffered_score_frames: Vec<(f32, Score)>,

    new_map_check: LatestBeatmapHelper
}
impl SpectatorManager {
    pub async fn new(host_id: u32, host_username: String) -> Self {
        Self {
            frames: Vec::new(),
            state: SpectatorState::None,
            host_id,
            host_username,
            game_manager: None,
            good_until: 0.0,
            map_length: 0.0,
            spectator_cache: HashMap::new(),
            score_menu: None,
            buffered_score_frames: Vec::new(),
            current_map: None,
            window_size: WindowSizeHelper::new(),
            new_map_check: LatestBeatmapHelper::new(),
        }
    }

    async fn start_game(&mut self, game:&mut Game, beatmap_hash:Md5Hash, mode:String, mods_str:String, current_time:f32, speed: u16) {
        trace!("Started watching host play a map");
        self.current_map = Some((beatmap_hash, mode.clone(), mods_str.clone(), speed));

        self.good_until = 0.0;
        self.map_length = 0.0;
        self.buffered_score_frames.clear();

        let mut mods = ModManager::new().with_speed(speed);
        mods.mods = Score::mods_from_string(mods_str);

        // find the map
        let mut beatmap_manager = BEATMAP_MANAGER.write().await;
        match beatmap_manager.get_by_hash(&beatmap_hash) {
            Some(map) => {
                beatmap_manager.set_current_beatmap(game, &map, false).await;
                match manager_from_playmode(mode.clone(), &map).await {
                    Ok(mut manager) => {
                        // remove score menu
                        self.score_menu = None;

                        // set manager things
                        manager.apply_mods(mods).await;
                        manager.replaying = true;
                        manager.replay.score_data = Some(Score::new(map.beatmap_hash, self.host_username.clone(), mode.clone()));
                        manager.on_start = Box::new(move |manager| {
                            trace!("Jumping to time {current_time}");
                            manager.jump_to_time(current_time.max(0.0), current_time > 0.0);
                        });
                        manager.start().await;
                        
                        // set our game manager
                        self.map_length = manager.end_time;
                        self.game_manager = Some(manager);
                        self.state = SpectatorState::Watching;
                    }
                    Err(e) => NotificationManager::add_error_notification("Error loading spec beatmap", e).await
                }
            }
            
            // user doesnt have beatmap
            None => NotificationManager::add_text_notification("You do not have the map!", 2000.0, Color::RED).await
        }
    }

    pub fn stop(&mut self) {
        OnlineManager::stop_spectating(self.host_id);
    }

    pub async fn update(&mut self, game: &mut Game) {
        // check for window size updates
        if self.window_size.update() {
            let window_size = (*self.window_size).clone();

            if let Some(menu) = &mut self.score_menu {
                menu.window_size_changed(window_size.clone()).await;
            }

            if let Some(game) = &mut self.game_manager {
                game.window_size_changed(window_size.clone()).await;
            }
        }

        // (try to) read pending data from the online manager
        if let Some(mut online_manager) = OnlineManager::try_get_mut() {
            self.frames.extend(online_manager.get_pending_spec_frames(self.host_id));
        }

        if let Some(menu) = &self.score_menu {
            if menu.should_close {
                self.score_menu = None
            }
        }

        // handle all new maps
        if self.new_map_check.update() {
            let new_map = self.new_map_check.0.clone();
            info!("got new map: {new_map:?}");
            
            let current_time = self.good_until;
            if let (true, Some((current_map, mode, mods, speed))) = (self.game_manager.is_none(), self.current_map.clone()) {
                info!("good state to start map");
                if &new_map.beatmap_hash == &current_map {
                    info!("starting map");
                    self.start_game(game, current_map, mode, mods, current_time, speed).await;
                } else {
                    info!("starting map");
                    // if this wasnt the map we wanted, check to see if the map we wanted was added anyways
                    // because it might have loaded a group of maps, and the one we wanted was loaded before the last map added
                    let has_map = BEATMAP_MANAGER.read().await.get_by_hash(&current_map).is_some();
                    if has_map {
                        self.start_game(game, current_map, mode, mods, current_time, speed).await;
                    }
                }
            }
        }


        // check all incoming frames
        for SpectatorFrame { time, action } in std::mem::take(&mut self.frames) {
            self.good_until = self.good_until.max(time as f32);

            // debug!("Packet: {action:?}");
            match action {
                SpectatorAction::Play { beatmap_hash, mode, mods, speed, map_game, map_link:_} => {
                    info!("got play: {beatmap_hash}, {mode}, {mods}");
                    let beatmap_hash = beatmap_hash.try_into().unwrap();

                    if BEATMAP_MANAGER.read().await.get_by_hash(&beatmap_hash).is_none() {
                        // we dont have the map, try downloading it

                        match map_game {
                            MapGame::Osu => {
                                // need to query the osu api to get the set id for this hashmap
                                match OsuApi::get_beatmap_by_hash(&beatmap_hash).await {
                                    Ok(Some(map_info)) => {
                                        // we have a thing! lets download it
                                        let settings = Settings::get();
                                        let username = &settings.osu_username;
                                        let password = &settings.osu_password;

                                        if !username.is_empty() && !password.is_empty() {
                                            let url = format!("https://osu.ppy.sh/d/{}.osz?u={username}&h={password}", map_info.beatmapset_id);
                                            
                                            let path = format!("downloads/{}.osz", map_info.beatmapset_id);
                                            perform_download(url, path)
                                        } else {
                                            warn!("not downloading map, osu user or password missing")
                                        }
                                    },
                                    Ok(None) => warn!("not downloading map, map not found"),
                                    Err(e) => warn!("not downloading map, {e}"),
                                }
                            }
                            MapGame::Quaver => {
                                // dont know how to download these yet
                            }

                            _ => {
                                // hmm
                            }
                        }
                    }

                    self.start_game(game, beatmap_hash, mode, mods, 0.0, speed).await;
                }

                SpectatorAction::Pause => {
                    trace!("Pause");
                    self.state = SpectatorState::Paused;
                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.pause();
                    }
                }
                SpectatorAction::UnPause => {
                    trace!("Unpause");
                    self.state = SpectatorState::Watching;
                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.start().await;
                    }
                }
                SpectatorAction::Buffer => {/*nothing to handle here*/},
                SpectatorAction::SpectatingOther { .. } => {
                    NotificationManager::add_text_notification("Host speccing someone", 2000.0, Color::BLUE).await;
                }
                SpectatorAction::ReplayAction { action } => {
                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.replay.frames.push(ReplayFrame::new(time, action))
                    }
                }
                SpectatorAction::ScoreSync { score } => {
                    // received score update
                    trace!("Got score update");
                    self.buffered_score_frames.push((time as f32, score));
                    // we should buffer these, and check the time. 
                    // if the time is at the score time, we should update our score, 
                    // as this score is probably more accurate, or at the very least will update new spectators
                }

                SpectatorAction::ChangingMap => {
                    trace!("Host changing maps");
                    self.state = SpectatorState::MapChanging;

                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.pause()
                    }
                }

                SpectatorAction::TimeJump { time } => {
                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.jump_to_time(time, true);
                    }
                }

                SpectatorAction::Unknown => {
                    // uh oh
                }
            }
        }
        
        // check our current state
        match &self.state {
            SpectatorState::None => {
                // in this case, the user should really be allowed to browse menus etc in the mean time. we might have to meme this
                if let Some(menu) = self.score_menu.as_mut() {
                    menu.update(game).await;

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
                        trace!("No longer buffering");
                        manager.start().await;
                    } else {
                        // trace!("Buffering");
                    }
                }
            }
            SpectatorState::Watching => {
                // currently watching someone
                if let Some(manager) = self.game_manager.as_mut() {
                    manager.update().await;

                    let manager_time = manager.time();
                    self.buffered_score_frames.retain(|(time, score)| {
                        if manager_time <= *time {
                            let mut other_score = score.clone();
                            other_score.hit_timings = manager.score.hit_timings.clone();
                            manager.score = IngameScore::new(other_score, true, false);
                            false
                        } else {
                            true
                        }
                    });
                    
                    let buffer_duration = (manager.time() + SPECTATOR_BUFFER_OK_DURATION * 2.0).clamp(0.0, self.map_length);
                    if self.good_until < buffer_duration {
                        self.state = SpectatorState::Buffering;
                        trace!("Starting buffer");
                        manager.pause();
                    }

                    if manager.completed || manager.time() >= self.map_length {
                        manager.on_complete();

                        // if we have a score frame we havent dealt with yet, its most likely the score frame sent once the map has ended
                        if self.buffered_score_frames.len() > 0 {
                            manager.score = IngameScore::new(self.buffered_score_frames.last().unwrap().clone().1, true, false);
                        }
                        let mut score_menu = ScoreMenu::new(&manager.score, manager.metadata.clone(), false);
                        score_menu.dont_do_menu = true;
                        self.score_menu = Some(score_menu);

                        self.state = SpectatorState::None;
                        self.current_map = None;
                        self.game_manager = None;
                    }
                }
            }
            SpectatorState::Paused => {},
            SpectatorState::MapChanging => {},
        }
    }

    pub async fn draw(&mut self, list: &mut RenderableCollection) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.draw(list).await
        }

        // draw score menu
        if let Some(menu) = self.score_menu.as_mut() {
            menu.draw(list).await
        }
        
        // draw spectator banner
        match &self.state {
            SpectatorState::None => {
                if self.score_menu.is_none() {
                    draw_banner("Waiting for Host", self.window_size.0, list);
                }
            }
            SpectatorState::Watching => {}
            SpectatorState::Buffering => draw_banner("Buffering", self.window_size.0, list),
            SpectatorState::Paused => draw_banner("Host Paused", self.window_size.0, list),
            SpectatorState::MapChanging => draw_banner("Host Changing Map", self.window_size.0, list),
        }
    }

    pub async fn mouse_scroll(&mut self, delta: f32, game:&mut Game) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.mouse_scroll(delta).await
        }
        
        // update score menu
        if let Some(menu) = self.score_menu.as_mut() {
            menu.on_scroll(delta, game).await
        }
    }
    pub async fn mouse_move(&mut self, pos:Vector2, game:&mut Game) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.mouse_move(pos).await
        }
        
        // update score menu
        if let Some(menu) = self.score_menu.as_mut() {
            menu.on_mouse_move(pos, game).await
        }
    }
    pub async fn mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.mouse_down(button).await;
        }

        // update score menu
        if let Some(menu) = self.score_menu.as_mut() {
            menu.on_click(pos, button, mods, game).await
        }
    }
    pub async fn mouse_up(&mut self, _pos:Vector2, button:MouseButton, _mods:KeyModifiers, _game:&mut Game) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.mouse_up(button).await
        }
    }

    pub async fn key_down(&mut self, key:Key, mods:KeyModifiers, game:&mut Game) {
        // check if we need to close something
        if key == Key::Escape {
            // if the score menu is open, close it and leave.
            if self.score_menu.is_some() {
                self.score_menu = None;
                return;
            }

            // let menu = game.menus.get("main").unwrap().clone();
            game.queue_state_change(GameState::InMenu(Box::new(MainMenu::new().await)));
            // resume song if paused

            if let Some(song) = AudioManager::get_song().await {
                if song.is_paused() {
                    song.play(false);
                }
            }
        }


        // update score menu
        if let Some(manager) = self.game_manager.as_mut() {
            manager.key_down(key, mods).await
        }

        // update score menu
        if let Some(menu) = self.score_menu.as_mut() {
            menu.on_key_press(key, game, mods).await;
        }
    }
    pub async fn key_up(&mut self, key:Key, _mods:KeyModifiers, game:&mut Game) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.key_up(key).await
        }

        // update score menu
        if let Some(menu) = self.score_menu.as_mut() {
            menu.on_key_release(key, game).await;
        }
    }
}

fn draw_banner(text:&str, window_size: Vector2, list: &mut RenderableCollection) {
    let mut offset_text = Text::new(
        Vector2::ZERO, // centered anyways
        32.0,
        text.to_owned(),
        Color::BLACK,
        Font::Main
    );
    
    let text_width = offset_text.measure_text().x + BANNER_WPADDING;
    // center
    let rect = Bounds::new(
        Vector2::new((window_size.x - text_width) / 2.0, window_size.y * 1.0/3.0), 
        Vector2::new(text_width + BANNER_WPADDING, 64.0)
    );
    offset_text.center_text(&rect);
    // add
    list.push(visibility_bg(rect.pos, rect.size));
    list.push(offset_text);
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
