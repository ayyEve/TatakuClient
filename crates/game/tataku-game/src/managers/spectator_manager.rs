use crate::prelude::*;

/// Manager for when we're spectating another user
pub struct SpectatorManager {
    frames: VecDeque<SpectatorFrame>,
    state: SpectatorState,
    pub host_id: u32,
    pub host_username: ArcStr,

    /// what is the current map's hash?
    /// if this is Some and game_manager is None, we dont have the map
    host_map: Option<HostMap>,

    /// list of id,username for other spectators
    pub spectator_cache: HashMap<u32, ArcStr>,
    new_map: ValueChangeHelper<Md5Hash>,
    // own_beatmap: ValueChangeHelper<Arc<BeatmapMeta>>,

    infos: GamemodeInfos,
}
impl SpectatorManager {
    pub fn new(
        host_id: u32, 
        host_username: impl Into<ArcStr>,
        infos: GamemodeInfos,
    ) -> Self {
        Self {
            infos,

            frames: VecDeque::new(),
            state: SpectatorState::None,
            host_id,
            host_username: host_username.into(),
            spectator_cache: HashMap::new(),
            host_map: None,

            // own_beatmap: ValueChangeHelper::new("map.hash"),
            new_map: ValueChangeHelper::new("global.new_map_hash"),
        }
    }

    pub fn add_frame(&mut self, frame: SpectatorFrame) {
        self.frames.push_back(frame);
    }

    fn start_game(
        &mut self, 
        values: &ValueCollection, 
        current_time: f32,
        actions: &mut ActionQueue,
    ) -> Option<Box<GameplayManager>> {
        trace!("Trying to watch host play a map");
        let HostMap { 
            map_hash, 
            playmode, 
            mods 
        } = self.host_map.clone()?;

        // see if our current map is the host's map
        let map = values.beatmap_manager.current_beatmap.as_ref()?;
        let map_path = map.file_path.clone();
        let hash = map.beatmap_hash;
        if hash != map_hash { return None }

        match manager_from_playmode_path_hash(
            &self.infos, 
            &playmode, 
            &map_path, 
            hash, 
            mods.clone(), 
            &values.settings
        ) {
            Ok(mut manager) => {
                // set manager things
                manager.handle_action(
                    GameplayAction::ApplyMods(mods), 
                    &values.settings
                );

                manager.set_mode(GameplayMode::Spectator(Box::new(SpectatorGameplayInfo { 
                    host_id: self.host_id,
                    host_username: self.host_username.clone(),
                    pending_frames: self.frames.take(),
                    spectators: self.spectator_cache.clone()
                })).into());

                // manager.replay.score_data = Some(Score::new(map.beatmap_hash, self.host_username.clone(), mode.clone()));
                manager.on_start = Some(Box::new(move |manager| {
                    trace!("Jumping to time {current_time}");
                    manager.jump_to_time(
                        current_time.max(0.0), 
                        current_time > 0.0
                    );
                }));

                return Some(Box::new(manager));
            }

            Err(e) => actions.push(
                Notification::new_error(
                    "Error loading spec beatmap", 
                    e
                )
            ),
        }

        None
    }

    fn check_new_maps(
        &mut self,
        manager: Option<&mut Box<GameplayManager>>,
        values: &mut ValueCollection,
        actions: &mut ActionQueue,
    ) -> Option<Box<GameplayManager>> { 
        // only continue if we received a map update
        let Ok(Some(_)) = self.new_map.update(values) else { return None };

        // and only continue if we arent playing anything right now
        if manager.is_some() { return None }

        let host_map = self.host_map.as_ref()?;
        if values.beatmap_manager.beatmaps_by_hash.contains_key(&host_map.map_hash) {
            actions.push(BeatmapAction::SetFromHash(
                host_map.map_hash, 
                SetBeatmapOptions::default().restart_song(true)
            ));

            let current_time = (self.frames.iter().fold(
                0.0, 
                |t, f| f.time.max(t)) - 2000.0
            ).max(0.0);
            
            return self.start_game(values, current_time, actions);
        }

        None
    }

    pub fn update(
        &mut self,
        manager: Option<&mut Box<GameplayManager>>,
        values: &mut ValueCollection,
        actions: &mut ActionQueue,
    ) -> Option<Box<GameplayManager>> {
        // handle new maps
        if let Some(manager) = self.check_new_maps(manager, values, actions) {
            return Some(manager)
        }

        // check all incoming frames
        while let Some(SpectatorFrame { 
            time: _, 
            action 
        }) = self.frames.pop_front() {
            println!("Handling spec frame: {action:?}");

            // debug!("Packet: {action:?}");
            match action {
                SpectatorAction::Play { 
                    beatmap_hash, 
                    mode, 
                    mods, 
                    speed, 
                    map_game, 
                    map_link: _
                } => {
                    info!("got play: {beatmap_hash}, {mode}, {mods:?}");
                    self.host_map = Some(HostMap::new(
                        beatmap_hash, 
                        mode, 
                        mods, 
                        speed
                    ));

                    if values.beatmap_manager.get_by_hash(&beatmap_hash).is_some() {
                        actions.push(BeatmapAction::SetFromHash(
                            beatmap_hash, 
                            SetBeatmapOptions::default().restart_song(true)
                        ));
                        self.start_game(values, 0.0, actions);
                    } else {
                        let settings = &values.settings;
                        info!("no beatmap, attempting to download");
                        self.download_beatmap(beatmap_hash, &map_game, settings, actions);
                    }

                    break;
                }
                SpectatorAction::SpectatingOther { .. } => {
                    actions.push(
                        Notification::default()
                        .text("Host speccing someone")
                        .duration(2000.0)
                        .color(Color::BLUE)
                    );
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

        None
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


    pub fn key_down(
        &mut self, 
        key: Key, 
        _mods: KeyModifiers,
        actions: &mut ActionQueue,
    ) {
        // check if we need to close something
        if key == Key::Escape {
            #[cfg(feature="graphics")] 
            actions.push(MenuAction::set_menu("main_menu"));
            // resume song if paused
            actions.push(SongAction::Play);
        }
    }


    fn download_beatmap(
        &self, 
        beatmap_hash: Md5Hash, 
        map_game: &MapGame, 
        settings: &Settings,
        actions: &mut ActionQueue,
    ) {
        match map_game {
            MapGame::Osu => {
                // need to query the osu api to get the set id for this hashmap
                match OsuApi::get_beatmap_by_hash(beatmap_hash, settings) {
                    Ok(Some(map_info)) => {
                        // we have a thing! lets download it
                        let creds = &settings.integrations.osu;
                        let username = &creds.username;
                        let password = &creds.password;

                        if !username.is_empty() && !password.is_empty() {
                            let id = map_info.beatmapset_id;

                            let url = format!("https://osu.ppy.sh/d/{id}.osz?u={username}&h={password}");
                            let path = format!("downloads/{id}.osz");

                            let dl = Downloadable::new(
                                path,
                                move || Downloader::download(DownloadOptions::new(url.clone(), 2))
                            );
                            actions.push(TatakuAction::Download(Box::new(dl)));
                        } else {
                            warn!("not downloading map, osu user or password missing");
                            // actions.push(  
                            //     Notification::default()
                            //         .text("Click here to download the beatmap")
                            //         .color(Color::RED)
                            //         .duration(10_000.0)
                            //         .onclick(NotificationOnClick::Url(format!("https://osu.ppy.sh/beatmapsets/{id}")))
                            // );
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
}

#[derive(Clone)]
struct HostMap {
    map_hash: Md5Hash,
    playmode: String,
    mods: ModManager,
}
impl HostMap {
    fn new(
        map_hash: Md5Hash,
        playmode: String,
        mods: Vec<ModDefinition>,
        speed: u16
    ) -> Self {
        Self { 
            map_hash, 
            playmode, 
            mods: ModManager::default()
                .with_speed(speed)
                .with_mods(mods.iter()) 
        }
    }
}
