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
    window_size: WindowSizeHelper,

    /// what time we have data for
    /// ie, up to what time we can show gameplay
    pub good_until: f32,
    pub map_length: f32,

    /// what is the current map's hash? 
    /// if this is Some and game_manager is None, we dont have the map
    pub current_map: Option<(String, PlayMode, String, u16)>,

    /// list of id,username for specs
    pub spectator_cache: HashMap<u32, String>,

    /// list
    buffered_score_frames: Vec<(f32, Score)>,

    new_map_check: MultiBomb<Arc<BeatmapMeta>>
}
impl SpectatorManager {
    pub async fn new() -> Self {
        Self {
            frames: Vec::new(),
            state: SpectatorState::None,
            game_manager: None,
            good_until: 0.0,
            map_length: 0.0,
            spectator_cache: HashMap::new(),
            score_menu: None,
            buffered_score_frames: Vec::new(),
            current_map: None,
            new_map_check: BEATMAP_MANAGER.read().await.new_map_added.1.clone(),
            window_size: WindowSizeHelper::new(),
        }
    }

    pub async fn update(&mut self, game: &mut Game) {
        self.window_size.update();

        // (try to) read pending data from the online manager
        if let Ok(mut online_manager) = ONLINE_MANAGER.try_write() {
            self.frames.extend(online_manager.get_pending_spec_frames());
        }

        if let Some(menu) = &self.score_menu {
            if menu.should_close {
                self.score_menu = None
            }
        }

        // handle all new maps
        while let Some(new_map) = self.new_map_check.exploded() {
            info!("got new map: {new_map:?}");
            
            let current_time = self.good_until;
            if let (true, Some((current_map, mode, mods, speed))) = (self.game_manager.is_none(), self.current_map.clone()) {
                info!("good state to start map");
                if &new_map.beatmap_hash == &current_map {
                    info!("starting map");
                    self.start_game(game, current_map, mode, mods, current_time, speed).await;
                }
            }
        }


        // check all incoming frames
        for (time, frame) in std::mem::take(&mut self.frames) {
            self.good_until = self.good_until.max(time as f32);

            trace!("Packet: {:?}", frame);
            match frame {
                SpectatorFrameData::Play { beatmap_hash, mode, mods, speed} => {
                    info!("got play: {beatmap_hash}, {mode}, {mods}");
                    self.start_game(game, beatmap_hash, mode, mods, 0.0, speed).await;
                }

                SpectatorFrameData::MapInfo { beatmap_hash, game, download_link: _ } => {
                    let beatmap_manager = BEATMAP_MANAGER.read().await;
                    if beatmap_manager.get_by_hash(&beatmap_hash).is_none() {
                        // we dont have the map, try downloading it

                        match &*game {
                            "osu" => {
                                // need to query the osu api to get the set id for this hashmap
                                match OsuApi::get_beatmap_by_hash(&beatmap_hash).await {
                                    Ok(Some(map_info)) => {
                                        // we have a thing! lets download it
                                        let settings = get_settings!();
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
                            },
                            "quaver" => {
                                // dont know how to download these yet
                            },

                            _ => {
                                // hmm
                            }
                        }
                    }
                }

                SpectatorFrameData::Pause => {
                    trace!("Pause");
                    self.state = SpectatorState::Paused;
                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.pause();
                    }
                }
                SpectatorFrameData::UnPause => {
                    trace!("Unpause");
                    self.state = SpectatorState::Watching;
                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.start().await;
                    }
                }
                SpectatorFrameData::Buffer => {/*nothing to handle here*/},
                SpectatorFrameData::SpectatingOther { .. } => {
                    NotificationManager::add_text_notification("Host speccing someone", 2000.0, Color::BLUE).await;
                }
                SpectatorFrameData::ReplayFrame { frame } => {
                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.replay.frames.push((time as f32, frame))
                    }
                }
                SpectatorFrameData::ScoreSync { score } => {
                    // received score update
                    trace!("Got score update");
                    self.buffered_score_frames.push((time as f32, score));
                    // we should buffer these, and check the time. 
                    // if the time is at the score time, we should update our score, 
                    // as this score is probably more accurate, or at the very least will update new spectators
                }

                SpectatorFrameData::ChangingMap => {
                    trace!("Host changing maps");
                    self.state = SpectatorState::MapChanging;

                    if let Some(manager) = self.game_manager.as_mut() {
                        manager.pause()
                    }
                }

                SpectatorFrameData::PlayingResponse { user_id, beatmap_hash, mode, mods, current_time, speed } => {
                    warn!("got playing response: {user_id}, {beatmap_hash}, {mode}, {mods}, {current_time}");

                    let self_id = ONLINE_MANAGER.read().await.user_id;

                    if user_id == self_id {
                        self.start_game(game, beatmap_hash, mode, mods, current_time, speed).await
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
                        let mut score_menu = ScoreMenu::new(&manager.score, manager.metadata.clone());
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

    async fn start_game(&mut self, game:&mut Game, beatmap_hash:String, mode:PlayMode, mods_str:String, current_time:f32, speed: u16) {
        self.current_map = Some((beatmap_hash.clone(), mode.clone(), mods_str.clone(), speed));

        self.good_until = 0.0;
        self.map_length = 0.0;
        self.buffered_score_frames.clear();
        // user started playing a map
        trace!("Host started playing map");

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
                        manager.on_start = Box::new(move |manager| {
                            trace!("Jumping to time {}", current_time);
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

    pub async fn draw(&mut self, args: RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.draw(args, list).await
        }

        // draw score menu
        if let Some(menu) = self.score_menu.as_mut() {
            list.extend(menu.draw(args).await)
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

    pub async fn mouse_scroll(&mut self, delta: f64, game:&mut Game) {
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

    pub async fn key_down(&mut self, key:piston::Key, mods:KeyModifiers, game:&mut Game) {
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
    pub async fn key_up(&mut self, key:piston::Key, _mods:KeyModifiers, game:&mut Game) {
        if let Some(manager) = self.game_manager.as_mut() {
            manager.key_up(key).await
        }

        // update score menu
        if let Some(menu) = self.score_menu.as_mut() {
            menu.on_key_release(key, game).await;
        }
    }
}

// when the manager is dropped, tell the server we stopped spectating
impl Drop for SpectatorManager {
    fn drop(&mut self) {
        OnlineManager::stop_spectating();
    }
}


fn draw_banner(text:&str, window_size: Vector2, list: &mut Vec<Box<dyn Renderable>>) {
    let font = get_font();

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
    list.push(visibility_bg(rect.current_pos, rect.size, BANNER_DEPTH + 10.0));
    list.push(Box::new(offset_text));
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