use crate::prelude::*;

#[derive(Debug)]
pub struct MultiplayerManager {
    /// list of actions to send back to the game object
    actions: ActionQueue,
    
    /// lobby data
    pub lobby: CurrentLobbyInfo,

    /// what is the current beatmap we have selected?
    current_beatmap: SyValueHelper<Md5Hash>,
    
    /// what playmode is selected by the host?
    selected_mode: Option<String>,

    /// what mods are currently enabled?
    current_mods: SyValueHelper<ModManager>,

    /// helper to get new beatmaps
    new_beatmap_helper: SyValueHelper<Md5Hash>,

    /// async beatmap loader
    beatmap_loader: Option<AsyncLoader<TatakuResult<GameplayManager>>>,

    /// have we sent that we've loaded the beatmap?
    load_complete_sent: bool,
}

impl MultiplayerManager {
    pub fn new(lobby: CurrentLobbyInfo) -> Self {
        Self {
            actions: ActionQueue::new(),
            lobby,
            current_beatmap: SyValueHelper::new("map.hash"),
            selected_mode: None,
            current_mods: SyValueHelper::new("global.mods"),

            new_beatmap_helper: SyValueHelper::new("global.new_map_hash"),

            beatmap_loader: None,
            load_complete_sent: false,
        }
    }


    pub async fn update(
        &mut self,
        manager: Option<&mut Box<GameplayManager>>,
        values: &mut ValueCollection
    ) -> Vec<TatakuAction> {
        let previous_map = self.current_beatmap.clone();
        if let Ok(Some(_)) = self.current_beatmap.update(values) {

            // if we're the host, the map update was us selecting a map
            // so, update the lobby with the selected map
            if self.is_host() {
                // if let Ok(hash) = self.current_beatmap.deref().try_into() {
                //     if !self.current_beatmap_is_selected() {
                //         self.actions.push(MultiplayerAction::SetBeatmap { hash, mode: self.selected_mode.clone() });
                //     }
                // } else if let Ok(old_hash) = previous_map.deref().try_into() {
                //     // if nothing was selected, make sure we revert back to the previous beatmap
                //     self.actions.push(BeatmapAction::SetFromHash(old_hash, true, false));
                    tokio::spawn(OnlineManager::update_lobby_state(LobbyUserState::NotReady));
                // }
            } else {
                // otherwise, the lobby map was updated
                if let Some(hash) = self.current_beatmap.as_ref() {
                    // if the map is valid, select it
                    self.actions.push(BeatmapAction::SetFromHash(*hash, SetBeatmapOptions::new().restart_song(true)));
                    tokio::spawn(OnlineManager::update_lobby_state(LobbyUserState::NotReady));
                } else {
                    // otherwise, remove the current map
                    self.actions.push(BeatmapAction::Remove);
                    tokio::spawn(OnlineManager::update_lobby_state(LobbyUserState::NoMap));
                }
            }

            // // only update the map if a map was selected (result is none if back/esc was pressed)
            // if let Some(map) = &self.current_beatmap.0 {
            //     // dont set the beatmap here, wait for the server reply
            //     let mode = self.selected_mode.clone().unwrap_or_default();
            //     tokio::spawn(OnlineManager::update_lobby_beatmap(map.clone(), mode));
            // } else if let Some(beatmap) = &previous_map.0 {
            //     // if nothing was selected, make sure we revert back to the previous beatmap
            //     self.actions.push(BeatmapAction::Set(beatmap.clone(), true, false));
            // }
        }
        
        // if we're loading the beatmap, check if its done
        if let Some(loader) = &self.beatmap_loader {
            if !self.load_complete_sent && loader.is_complete() {
                self.load_complete_sent = true;
                tokio::spawn(OnlineManager::lobby_load_complete());
            }
        }

        // if our mods changed, let the lobby know
        if let Ok(Some(mods)) = self.current_mods.update(values) {
            let speed = mods.speed;
            let mods = mods.mods.clone();
            tokio::spawn(OnlineManager::lobby_update_mods(mods, speed.as_u16()));
        }
    
        // check if a new beatmap was added
        if let Some(Some(new_hash)) = self.new_beatmap_helper.update(values).ok().filter(|_| manager.is_none()) {

            // if the map that was just added is the lobby's map, set it as our current map
            if let Some(beatmap) = &self.lobby.current_beatmap {

                if new_hash == &beatmap.hash {
                    self.actions.push(BeatmapAction::SetFromHash(beatmap.hash, SetBeatmapOptions::new().restart_song(true)));
                }

                // let beatmap_manager = BEATMAP_MANAGER.read().await;
                // if let Some(beatmap) = beatmap_manager.get_by_hash(&beatmap.hash) {
                //     // self.selected_beatmap = Some(beatmap);
                    
                //     // tokio::spawn(OnlineManager::update_lobby_state(LobbyUserState::NotReady));
                // }
            }
        }

        self.actions.take()
    }

    pub fn update_values(&self, values: &mut ValueCollection) {
        debug!("multi manager value update");
        // TODO: optimize this

        // let lobby_info:TatakuValue = (&self.lobby).into();
        // let mut map = lobby_info.to_map();
        // map.set_value("has_beatmap", TatakuVariable::new_game(self.current_beatmap_is_selected()));

        // values.set("lobby", TatakuVariable::new_game(map));

        values.lobby = Some(self.lobby.clone());
    }

    
    fn current_beatmap_is_selected(&self) -> bool {
        let Some(current_hash) = self.current_beatmap.as_ref() else { return false };
        let Some(selected) = &self.lobby.current_beatmap else { return false };
        &selected.hash == current_hash
    }

    pub async fn handle_packet(
        &mut self, 
        values: &mut ValueCollection, 
        packet: &MultiplayerPacket,
        mut manager: Option<&mut Box<GameplayManager>>
    ) -> TatakuResult {
        match packet {

            MultiplayerPacket::Server_LobbyUserJoined { lobby_id, user_id } => {
                if &self.lobby.info.id != lobby_id { return Ok(()) }
                self.lobby.info.players.push(LobbyUser { user_id: *user_id, ..Default::default() });

                let Some(user) = OnlineManager::get().await.users.get(&user_id).cloned() else { 
                    NotificationManager::add_text_notification(format!("user with id {} joined the match", user_id), 3000.0, Color::PURPLE).await;
                    self.update_values(values);
                    return Ok(())
                };

                let user = user.lock().await;
                self.lobby.player_usernames.insert(*user_id, user.username.clone());
                
                NotificationManager::add_text_notification(format!("{} joined the match", user.username), 3000.0, Color::PURPLE).await;
            }
            MultiplayerPacket::Server_LobbyUserLeft { lobby_id, user_id } => {
                if &self.lobby.id != lobby_id { return Ok(()); }

                self.lobby.players.retain(|u| &u.user_id != user_id);

                // find the slot that had this user and set it to empty (server will update its proper status next update)
                self.lobby.slots.values_mut().find(|s| **s == LobbySlot::Filled{user: *user_id}).ok_do_mut(|s|**s = LobbySlot::Empty);
                
                if user_id != &self.lobby.our_user_id {
                    let username = self.lobby.player_usernames.remove(&user_id).unwrap_or_default();
                    NotificationManager::add_text_notification(format!("{username} left the match"), 3000.0, Color::PURPLE).await;
                }
            }

            MultiplayerPacket::Server_LobbySlotChange { slot, new_status } => {
                self.lobby.info.slots.get_mut(&slot).ok_do_mut(|s| **s = *new_status);
            }

            MultiplayerPacket::Server_LobbyUserState { user_id, new_state } => {
                self.lobby.info.players
                    .iter_mut()
                    .find(|u| &u.user_id == user_id)
                    .ok_do_mut(|u| u.state = *new_state);
            }


            MultiplayerPacket::Server_LobbyStart => {
                self.lobby.play_pending = true;

                // if the server wants us to load the map and we arent already doing that, do it
                if self.beatmap_loader.is_none() {
                    
                    // only load map if we have it selected
                    if self.current_beatmap_is_selected() {
                        let Some(mode) = self.selected_mode.clone() else { return Ok(()) };
                        let Some(map) = &values.beatmap_manager.current_beatmap else { return Ok(()) };
                        
                        // let Ok(hash) = values.try_get("map.hash") else { return Ok(()) };
                        // let Ok(path) = values.get_string("map.path") else { return Ok(()) };
                        // let Ok(mods) = values.try_get("global.mods") else { return Ok(()) };
                        let mods = values.mods.clone();
                        let f = manager_from_playmode_path_hash(
                            mode, 
                            map.file_path.clone(), 
                            map.beatmap_hash, 
                            mods
                        );
                        self.beatmap_loader = Some(AsyncLoader::new(f));
                    } else {
                        warn!("not loading map: current != selected");
                    }

                    // if let Some((map, mode)) = self.selected_beatmap.as_ref().zip(self.selected_mode.clone()) {
                    //     let map = map.clone();
                    //     let mode = mode;
                    //     let f = async move {manager_from_playmode(mode, &map).await};
                    //     self.beatmap_loader = Some(AsyncLoader::new(f));
                    // }
                }
            }
            MultiplayerPacket::Server_LobbyBeginRound => {
                self.lobby.should_play = true;

                // if we're not playing yet
                if manager.is_none() {
                    if let Some(loader) = &self.beatmap_loader {
                        if let Some(Ok(mut manager)) = loader.check().await {
                            manager.set_mode(GameplayMode::multi());
                            self.actions.push(GameAction::StartGame(Box::new(manager)));
                            tokio::spawn(OnlineManager::update_lobby_state(LobbyUserState::InGame));
                        }
                    }
                    self.beatmap_loader = None;
                    self.load_complete_sent = false;

                    self.lobby.play_pending = false;
                    self.lobby.should_play = false;
                }
            }

            MultiplayerPacket::Server_LobbyMapChange { lobby_id, new_map } => {
                if &self.lobby.id != lobby_id { return Ok(()) };
                self.lobby.info.current_beatmap = Some(new_map.clone());

                if let Some(beatmap) = &self.lobby.current_beatmap {
                    // self.selected_beatmap = BEATMAP_MANAGER.read().await.get_by_hash(&beatmap.hash);
                    // self.selected_mode = Some(beatmap.mode.clone());
                    // GlobalValueManager::update(Arc::new(CurrentPlaymode(beatmap.mode.clone())));

                    self.selected_mode = Some(beatmap.mode.clone());
                    // update the playmode
                    self.actions.push(BeatmapAction::SetPlaymode(beatmap.mode.clone()));
                    
                    // the beatmap change handler in Self::update will handle the rest
                    self.actions.push(BeatmapAction::SetFromHash(beatmap.hash, SetBeatmapOptions::new().restart_song(true)));


                    // if let Some(beatmap) = &self.selected_beatmap {
                        // self.actions.push(BeatmapAction::SetFromHash(beatmap.hash, true, false));
                    // } 
                    // else {
                    //     self.actions.push(BeatmapMenuAction::Remove);
                    // }

                    // let new_state = match self.selected_beatmap {
                    //     Some(_) => LobbyUserState::NotReady,
                    //     None => LobbyUserState::NoMap,
                    // };
                    // tokio::spawn(OnlineManager::update_lobby_state(new_state));
                } else {
                    self.actions.push(BeatmapAction::Remove);
                    tokio::spawn(OnlineManager::update_lobby_state(LobbyUserState::NoMap));
                }
            }

            //TODO: implement no free mods
            MultiplayerPacket::Server_LobbyModsChanged { free_mods:_, mods:_, speed:_ } => {
                // let mut current_lobby = CurrentLobbyInfo::get_mut();
                // let Some(lobby) = &mut *current_lobby else { continue };
                // lobby.free_mods = free_mods
                // lobby.mods = mods;
                // lobby.speed = speed;
            }

            MultiplayerPacket::Server_LobbyUserModsChanged { user_id, mods, speed } => {
                self.lobby.players.iter_mut().find(|u| &u.user_id == user_id).ok_do_mut(|u| {
                    u.mods = mods.clone();
                    u.speed = *speed;
                });
            }

            MultiplayerPacket::Server_LobbyPlayerMapComplete { user_id, score } => {
                self.lobby.player_scores.insert(*user_id, score.clone());
            }

            MultiplayerPacket::Server_LobbyRoundComplete => {
                info!("lobby round completed");
            }

            MultiplayerPacket::Server_LobbyScoreUpdate { user_id, score } => {
                self.lobby.player_scores.insert(*user_id, score.clone());

                // update the manager
                if let Some(manager) = &mut manager {
                    manager.score_list = self.lobby.player_scores.iter()
                        .filter(|(u,_)| u != &&self.lobby.our_user_id) // make sure we dont re-add our own score in
                        .map(|(_,s)| IngameScore::new(s.clone(), false, false)).collect();
                    manager.score_list.sort_by(|a, b| b.score.score.cmp(&a.score.score));
                }
            }
            
            MultiplayerPacket::Server_LobbyStateChange { lobby_id, new_state } => {
                if lobby_id != &self.lobby.id { return Ok(()) }
                self.lobby.info.state = *new_state;
            }

            MultiplayerPacket::Server_LobbyChangeHost { new_host } => {
                let was_host = self.is_host();
                self.lobby.host = *new_host;

                // if we just became the host, show a notif
                if self.is_host() && !was_host {
                    NotificationManager::add_text_notification("You are now the host!", 3000.0, Color::PURPLE_AMETHYST).await;
                }
            }

            _ => {}
        }

        self.update_values(values);
        
        Ok(())
    }

    pub async fn handle_lobby_action(&mut self, action: LobbyAction) {
        match action {
            LobbyAction::Ready => {
                tokio::spawn(OnlineManager::update_lobby_state(LobbyUserState::Ready));
            }
            LobbyAction::Unready => {
                tokio::spawn(OnlineManager::update_lobby_state(LobbyUserState::NotReady));
            }

            LobbyAction::OpenMapLink => {
                let Some(beatmap) = &self.lobby.current_beatmap else { return };
                let hash = beatmap.hash;

                tokio::spawn(async move {
                    let settings = Settings::get();
                    let req = reqwest::get(format!("{}/api/get_beatmap_url?hash={hash}", settings.score_url)).await;
                    match req {
                        Err(e) => NotificationManager::add_error_notification("Error with beatmap url request", e.to_string()).await,
                        Ok(resp) => {
                            #[allow(unused)] #[derive(Deserialize)]
                            struct Resp { error: Option<String>, url: Option<String> }
                            
                            let Ok(body) = resp.text().await else { NotificationManager::add_text_notification("shit", 3000.0, Color::RED).await; return; };
                            info!("url resp: {body}");

                            match serde_json::from_str(&body) {
                                Ok(Resp {url: Some(url), ..}) => open_link(url),
                                _ => error!("some shit broke i dont care")
                            }
                        }
                    }
                });
            }

            // slot actions

            LobbyAction::SlotAction(LobbySlotAction::ShowProfile(slot)) => {
                let Some(LobbySlot::Filled { user: _ }) = self.lobby.slots.get(&slot) else { return };
                // TODO: set values for user_id and slot_id, then open dialog
                // self.actions.push()
            }
            
            LobbyAction::SlotAction(LobbySlotAction::MoveTo(slot)) => {
                let Some(LobbySlot::Empty) = self.lobby.slots.get(&slot) else { return };
                
                tokio::spawn(OnlineManager::move_lobby_slot(slot));
            }

            LobbyAction::SlotAction(LobbySlotAction::TransferHost(slot)) => {
                if !self.is_host() { return }
                let Some(LobbySlot::Filled { user }) = self.lobby.slots.get(&slot) else { return };
                tokio::spawn(OnlineManager::lobby_change_host(*user));
            }

            LobbyAction::SlotAction(LobbySlotAction::Lock(slot)) 
            | LobbyAction::SlotAction(LobbySlotAction::Kick(slot))
            => {
                if !self.is_host() { return }
                tokio::spawn(OnlineManager::update_lobby_slot(slot, LobbySlot::Locked));
            }
            LobbyAction::SlotAction(LobbySlotAction::Unlock(slot)) => {
                if !self.is_host() { return }
                tokio::spawn(OnlineManager::update_lobby_slot(slot, LobbySlot::Empty));
            }

            _ => {}
        }
    }
    
    pub fn is_host(&self) -> bool {
        self.lobby.is_host()
    }
}

