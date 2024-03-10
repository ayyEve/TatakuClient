use crate::prelude::*;
const BANNER_WPADDING:f32 = 5.0;

/// how long of a buffer should we have? (ms)
pub const SPECTATOR_BUFFER_OK_DURATION:f32 = 500.0;

pub struct SpectatorManager {
    actions: ActionQueue,

    pub frames: VecDeque<SpectatorFrame>, 
    pub state: SpectatorState, 
    pub host_id: u32,
    pub host_username: String,

    /// what is the current map's hash? 
    /// if this is Some and game_manager is None, we dont have the map
    pub current_map: Option<(Md5Hash, String, String, u16)>,

    /// list of id,username for other spectators
    pub spectator_cache: HashMap<u32, String>,

    new_map_check: LatestBeatmapHelper
}
impl SpectatorManager {
    pub async fn new(host_id: u32, host_username: String) -> Self {
        Self {
            actions: ActionQueue::new(),

            frames: VecDeque::new(),
            state: SpectatorState::None,
            host_id,
            host_username,
            spectator_cache: HashMap::new(),
            current_map: None,
            new_map_check: LatestBeatmapHelper::new(),
        }
    }
    pub async fn new_from_manager(manager: &IngameManager) -> Self {
        let GameplayMode::Spectator { 
            frames, 
            host_id, 
            host_username, 
            spectators,
            ..
        } = manager.get_mode() else { panic!("trying to make a spectator manager from an ingame manager which isnt in spectating mode") };

        Self {
            actions: ActionQueue::new(),
            state: SpectatorState::None,

            frames: frames.clone(),
            host_id: *host_id,
            host_username: host_username.clone(),
            spectator_cache: spectators.clone(),
            current_map: None,
            new_map_check: LatestBeatmapHelper::new(),
        }
    }

    async fn start_game(&mut self, beatmap_hash:Md5Hash, mode:String, mods_str:String, current_time:f32, speed: u16) {
        trace!("Started watching host play a map");
        self.current_map = Some((beatmap_hash, mode.clone(), mods_str.clone(), speed));

        let mut mods = ModManager::new().with_speed(speed);
        mods.mods = Score::mods_from_string(mods_str);

        // find the map
        let beatmap_manager = BEATMAP_MANAGER.read().await;
        match beatmap_manager.get_by_hash(&beatmap_hash) {
            Some(map) => {
                // beatmap_manager.set_current_beatmap(game, &map, false).await;
                self.actions.push(BeatmapMenuAction::Set(map.clone(), false));

                match manager_from_playmode(mode.clone(), &map).await {
                    Ok(mut manager) => {
                        // set manager things
                        manager.apply_mods(mods).await;
                        manager.set_mode(GameplayMode::spectator(
                            self.host_id,
                            self.host_username.clone(),
                            self.frames.take(),
                            self.spectator_cache.clone()
                        ));
                        // manager.replay.score_data = Some(Score::new(map.beatmap_hash, self.host_username.clone(), mode.clone()));
                        manager.on_start = Box::new(move |manager| {
                            trace!("Jumping to time {current_time}");
                            manager.jump_to_time(current_time.max(0.0), current_time > 0.0);
                        });
                        
                        self.actions.push(GameMenuAction::StartGame(Box::new(manager)));
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

    pub async fn update(
        &mut self, 
        manager: Option<&mut Box<IngameManager>>,
    ) -> Vec<MenuAction> {
        if manager.is_some() { return self.actions.take() }

        // (try to) read pending data from the online manager
        if let Some(mut online_manager) = OnlineManager::try_get_mut() {
            self.frames.extend(online_manager.get_pending_spec_frames(self.host_id));
        }

        // handle all new maps
        if self.new_map_check.update() {
            let new_map = self.new_map_check.0.clone();
            info!("got new map: {new_map:?}");

            // TODO: !!!
            let current_time = 0.0;
            if let Some((current_map, mode, mods, speed)) = self.current_map.clone() {
                info!("good state to start map");
                if &new_map.beatmap_hash == &current_map {
                    info!("starting map");
                    self.start_game( current_map, mode, mods, current_time, speed).await;
                } else {
                    info!("starting map");
                    // if this wasnt the map we wanted, check to see if the map we wanted was added anyways
                    // because it might have loaded a group of maps, and the one we wanted was loaded before the last map added
                    let has_map = BEATMAP_MANAGER.read().await.get_by_hash(&current_map).is_some();
                    if has_map {
                        self.start_game(current_map, mode, mods, current_time, speed).await;
                    }
                }
            }
        }


        // check all incoming frames
        while let Some(SpectatorFrame { time, action }) = self.frames.pop_front() {

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
                                            perform_download(url, path, Default::default())
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

                    self.start_game(beatmap_hash, mode, mods, 0.0, speed).await;

                    break;
                }
                SpectatorAction::SpectatingOther { .. } => {
                    NotificationManager::add_text_notification("Host speccing someone", 2000.0, Color::BLUE).await;
                }

                SpectatorAction::ChangingMap => {
                    trace!("Host changing maps");
                    self.state = SpectatorState::MapChanging;
                }


                SpectatorAction::Unknown => {
                    // uh oh
                }

                other => warn!("spectator manager got unexpected spec action: {other:?}")
            }
        }

        self.actions.take()
    }

    // pub async fn draw(&mut self, list: &mut RenderableCollection) {
    //     // draw spectator banner
    //     match &self.state {
    //         SpectatorState::None => {
    //             if self.score_menu.is_none() {
    //                 draw_banner("Waiting for Host", self.window_size.0, list);
    //             }
    //         }
    //         SpectatorState::Watching => {}
    //         SpectatorState::Buffering => draw_banner("Buffering", self.window_size.0, list),
    //         SpectatorState::Paused => draw_banner("Host Paused", self.window_size.0, list),
    //         SpectatorState::MapChanging => draw_banner("Host Changing Map", self.window_size.0, list),
    //     }
    // }


    pub async fn key_down(&mut self, key:Key, _mods:KeyModifiers, game:&mut Game) {
        // check if we need to close something
        if key == Key::Escape {

            // let menu = game.menus.get("main").unwrap().clone();
            game.queue_state_change(GameState::SetMenu(Box::new(MainMenu::new().await)));
            // resume song if paused

            if let Some(song) = AudioManager::get_song().await {
                if song.is_paused() {
                    song.play(false);
                }
            }
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

#[derive(Debug)]
pub enum SpectatorManagerAction {
    QuitSpec,
}
