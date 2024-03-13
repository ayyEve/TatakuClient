use crate::prelude::*;


pub struct MultiplayerManager {
    /// list of actions to send back to the game object
    actions: ActionQueue,
    
    /// lobby data
    lobby: CurrentLobbyDataHelper,

    /// what map is selected by the host?
    selected_beatmap: Option<Arc<BeatmapMeta>>,

    /// what is the current beatmap we have selected?
    current_beatmap: CurrentBeatmapHelper,
    
    /// what playmode is selected by the host?
    selected_mode: Option<String>,

    /// what mods are currently enabled?
    current_mods: ModManagerHelper,

    /// helper to get new beatmaps
    latest_beatmap_helper: LatestBeatmapHelper,

    /// async beatmap loader
    beatmap_loader: Option<AsyncLoader<TatakuResult<IngameManager>>>,

    /// have we sent that we've loaded the beatmap?
    load_complete_sent: bool,
}

impl MultiplayerManager {
    pub fn new() -> Self {
        Self {
            actions: ActionQueue::new(),
            lobby: CurrentLobbyDataHelper::new(),
            selected_beatmap: None,
            current_beatmap: CurrentBeatmapHelper::new(),
            selected_mode: None,
            current_mods: ModManagerHelper::new(),
            latest_beatmap_helper: LatestBeatmapHelper::new(),
            beatmap_loader: None,
            load_complete_sent: false,
        }
    }


    pub async fn update(
        &mut self,
        mut manager: Option<&mut Box<IngameManager>>,
    ) -> Vec<MenuAction> {
        let old_info = self.lobby.clone();
        let old_info = old_info.as_ref();

        if self.lobby.update() {
            // get the lobby info
            let Some(info) = &**self.lobby else {
                return vec![MenuAction::MultiplayerAction(MultiplayerManagerAction::QuitMulti)]
            };

            // update the manager
            if let Some(manager) = &mut manager {
                manager.score_list = info.player_scores.iter()
                    .filter(|(u,_)|u != &&info.our_user_id) // make sure we dont re-add our own score in
                    .map(|(_,s)| IngameScore::new(s.clone(), false, false)).collect();
                manager.score_list.sort_by(|a, b|b.score.score.cmp(&a.score.score));
            }
            
            // update beatmap
            if old_info.as_ref().map(|i|&i.current_beatmap) != Some(&info.current_beatmap) {
                if let Some(beatmap) = &info.current_beatmap {
                    self.selected_beatmap = BEATMAP_MANAGER.read().await.get_by_hash(&beatmap.hash);
                    self.selected_mode = Some(beatmap.mode.clone());
                    GlobalValueManager::update(Arc::new(CurrentPlaymode(beatmap.mode.clone())));

                    if let Some(beatmap) = &self.selected_beatmap {
                        self.actions.push(BeatmapMenuAction::Set(beatmap.clone(), true, false));
                    } else {
                        self.actions.push(BeatmapMenuAction::Remove);
                    }

                    let new_state = match self.selected_beatmap {
                        Some(_) => LobbyUserState::NotReady,
                        None => LobbyUserState::NoMap,
                    };
                    tokio::spawn(OnlineManager::update_lobby_state(new_state));
                } else {
                    self.actions.push(BeatmapMenuAction::Remove);
                    tokio::spawn(OnlineManager::update_lobby_state(LobbyUserState::NoMap));
                }
            }

            // if we just became the host, show a notif
            if info.is_host() && (old_info.is_none() || !old_info.as_ref().unwrap().is_host()) {
                NotificationManager::add_text_notification("You are now the host!", 3000.0, Color::PURPLE_AMETHYST).await;
            }

            // if the server wants us to load the map and we arent already doing that, do it
            if info.play_pending && self.beatmap_loader.is_none() {
                if let Some((map, mode)) = self.selected_beatmap.as_ref().zip(self.selected_mode.clone()) {
                    let map = map.clone();
                    let mode = mode;
                    let f = async move {manager_from_playmode(mode, &map).await};
                    self.beatmap_loader = Some(AsyncLoader::new(f));
                }
            }

            // if we're not playing yet
            if info.should_play && manager.is_none() {
                if let Some(loader) = &self.beatmap_loader {
                    if let Some(Ok(mut manager)) = loader.check().await {
                        manager.set_mode(GameplayMode::multi());
                        self.actions.push(GameMenuAction::StartGame(Box::new(manager)));

                        // self.manager = Some(Box::new(manager));
                        tokio::spawn(OnlineManager::update_lobby_state(LobbyUserState::InGame));
                    }
                }
                self.beatmap_loader = None;
                self.load_complete_sent = false;

                if let Some(lobby) = &mut *CurrentLobbyInfo::get_mut() {
                    lobby.play_pending = false;
                    lobby.should_play = false;
                }
            }


        }


        let previous_map = self.current_beatmap.clone();
        if self.current_beatmap.update() && self.is_host() {
            // update the lobby with the selected map

            // only update the map if a map was selected (result is none if back/esc was pressed)
            if let Some(map) = &self.current_beatmap.0 {
                // dont set the beatmap here, wait for the server reply
                let mode = self.selected_mode.clone().unwrap_or_default();
                tokio::spawn(OnlineManager::update_lobby_beatmap(map.clone(), mode));
            } else if let Some(beatmap) = &previous_map.0 {
                // if nothing was selected, make sure we revert back to the previous beatmap
                self.actions.push(BeatmapMenuAction::Set(beatmap.clone(), true, false));
            }
        }
        
        // if we're loading the beatmap, check if its done
        if let Some(loader) = &self.beatmap_loader {
            if !self.load_complete_sent && loader.is_complete() {
                self.load_complete_sent = true;
                tokio::spawn(OnlineManager::lobby_load_complete());
            }
        }

        // if our mods changed, let the lobby know
        if self.current_mods.update() {
            let mods = self.current_mods.mods.clone();
            let speed = self.current_mods.speed;
            tokio::spawn(OnlineManager::lobby_update_mods(mods, speed.as_u16()));
        }
    
        // check if a new beatmap was added
        if self.latest_beatmap_helper.update() && manager.is_none() {
            // if the map that was just added is the lobby's map, set it as our current map
            let lobby_info = self.lobby_info();
            if let Some(beatmap) = &lobby_info.current_beatmap {

                let beatmap_manager = BEATMAP_MANAGER.read().await;
                if let Some(beatmap) = beatmap_manager.get_by_hash(&beatmap.hash) {
                    self.actions.push(BeatmapMenuAction::Set(beatmap.clone(), true, false));
                    self.selected_beatmap = Some(beatmap);
                    
                    tokio::spawn(OnlineManager::update_lobby_state(LobbyUserState::NotReady));
                }
            }
        }



        self.actions.take()
    }


    fn is_host(&self) -> bool {
        if self.lobby.is_none() { return false; }
        let info = self.lobby_info();
        info.host == info.our_user_id
    }

    
    fn lobby_info(&self) -> &CurrentLobbyInfo {
        if let Some(info) = &**self.lobby {
            info
        } else {
            panic!("tried getting current lobby info when not in lobby")
        }
    }
}

#[derive(Debug)]
pub enum MultiplayerManagerAction {
    QuitMulti,
    JoinMulti,
}