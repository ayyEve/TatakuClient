use crate::prelude::*;
// use tokio::sync::mpsc::{Receiver, channel};

pub const SCORE_SEND_TIME:f32 = 1_000.0;
// const CHANNEL_COUNT:usize = 2;

pub struct LobbyMenu {
    actions: ActionQueue,
    slots: Vec<LobbySlotComponent>,

    init_pending: bool,
    lobby_info: CurrentLobbyDataHelper,
    gameplay_preview: GameplayPreview,


    selected_beatmap: Option<Arc<BeatmapMeta>>,
    selected_mode: Option<String>,
    current_mods: ModManagerHelper,
    // latest_beatmap_helper: LatestBeatmapHelper,


    // beatmap_loader: Option<AsyncLoader<TatakuResult<IngameManager>>>,
    // load_complete_sent: bool,
    

    
    // left_scrollable: ScrollableArea,
    // right_scrollable: ScrollableArea,
    // menu: Option<Box<dyn AsyncMenu>>,
    // on_beatmap_select: Option<Receiver<Option<(Arc<BeatmapMeta>, String)>>>,
    // on_score_menu_close: Option<AsyncReceiver<()>>,
    // menu_game: MenuGameHelper,

    // manager: Option<Box<IngameManager>>,
    // since_last_escape: Instant,
    // score_send_timer: Instant,

    // slot_senders: HashMap<u8, (AsyncSender<(LobbySlot, bool)>, AsyncSender<Option<LobbyPlayerInfo>>)>,

}
impl LobbyMenu {
    pub async fn new() -> Self {
        let lobby_info = CurrentLobbyDataHelper::new();

        // let mut slot_senders = HashMap::new();
        let mut slots = Vec::new();
        for slot in 0..16 {
            slots.push(LobbySlotComponent::new(LobbySlot::Empty, false, None));
            // let (state_sender, state_receiver) = async_channel(CHANNEL_COUNT);
            // let (player_sender, player_receiver) = async_channel(CHANNEL_COUNT);
            
            // left_scrollable.add_item(Box::new(LobbySlotDisplay::new(left_size.x, slot, state_receiver, player_receiver)));
            // slot_senders.insert(slot, (state_sender, player_sender));
        }

        Self {
            actions: ActionQueue::new(),
            slots,
            init_pending: true,
            // slot_senders,

            lobby_info,
            selected_beatmap: None,
            selected_mode: Some(CurrentPlaymodeHelper::new().0.clone()),

            gameplay_preview: GameplayPreview::new(true, true, Arc::new(|s|s.background_game_settings.multiplayer_menu_enabled)),
            
            current_mods: ModManagerHelper::new(),
        }
    }

    pub async fn refresh_data(&mut self, old_info: &Option<CurrentLobbyInfo>) {
        let Some(info) = &**self.lobby_info else { info!("no current lobby, leaving multiplayer"); return self.quit_lobby().await };

        // update slots
        for (slot, state) in info.slots.iter() {
            let our_slot = self.slots.get_mut(*slot as usize).unwrap();
            // let (status, player) = self.slot_senders.get(slot).unwrap();
            our_slot.status = state.clone();
            our_slot.is_host = &LobbySlot::Filled { user: info.host } == state; //if let LobbySlot::Filled { user } = &state { user == &info.host } else { false };

            if let LobbySlot::Filled { user } = state {
                let username = info.player_usernames.get(user).cloned().unwrap_or("?".to_owned());
                let Some(lobby_user) = info.players.iter().find(|u|&u.user_id == user) else {
                    warn!("couldnt find lobby user, probably in a bad state. lobby info below:");
                    warn!("{info:#?}");
                    our_slot.update_text();
                    continue;
                };
                our_slot.player = Some(LobbyPlayerInfo::new(lobby_user.clone(), username));
            } else {
                our_slot.player = None;
            }

            our_slot.update_text();
        }
        
        // update beatmap
        if old_info.as_ref().map(|i|&i.current_beatmap) != Some(&info.current_beatmap) {
            if let Some(beatmap) = &info.current_beatmap {
                self.selected_beatmap = BEATMAP_MANAGER.read().await.get_by_hash(&beatmap.hash);
                self.selected_mode = Some(beatmap.mode.clone());
                GlobalValueManager::update(Arc::new(CurrentPlaymode(beatmap.mode.clone())));

                if let Some(beatmap) = &self.selected_beatmap {
                    self.actions.push(BeatmapMenuAction::Set(beatmap.clone(), true));
                    // BEATMAP_MANAGER.write().await.set_current_beatmap(game, beatmap, true).await;
                } else {
                    self.actions.push(BeatmapMenuAction::Remove);
                    // BEATMAP_MANAGER.write().await.remove_current_beatmap(game).await;
                }

                let new_state = match self.selected_beatmap {
                    Some(_) => LobbyUserState::NotReady,
                    None => LobbyUserState::NoMap,
                };
                tokio::spawn(OnlineManager::update_lobby_state(new_state));
            } else {
                self.actions.push(BeatmapMenuAction::Remove);
                // BEATMAP_MANAGER.write().await.remove_current_beatmap(game).await;
                tokio::spawn(OnlineManager::update_lobby_state(LobbyUserState::NoMap));
            }
        }

        // if we just became the host, show a notif
        if info.is_host() && (old_info.is_none() || !old_info.as_ref().unwrap().is_host()) {
            NotificationManager::add_text_notification("You are now the host!", 3000.0, Color::PURPLE_AMETHYST).await;
        }

        // // if the server wants us to load the map and we arent already doing that, do it
        // if info.play_pending && self.beatmap_loader.is_none() {
        //     if let Some((map, mode)) = self.selected_beatmap.as_ref().zip(self.selected_mode.clone()) {
        //         let map = map.clone();
        //         let mode = mode;
        //         let f = async move {manager_from_playmode(mode, &map).await};
        //         self.beatmap_loader = Some(AsyncLoader::new(f));
        //     }
        // }

    }

    async fn quit_lobby(&mut self) {
        self.actions.push(MenuAction::MultiplayerAction(MultiplayerManagerAction::QuitMulti));
    }

    fn is_host(&self) -> bool {
        if self.lobby_info.is_none() { return false; }
        let info = self.lobby_info();
        info.host == info.our_user_id
    }

    fn lobby_info(&self) -> &CurrentLobbyInfo {
        if let Some(info) = &**self.lobby_info {
            info
        } else {
            panic!("tried getting current lobby info when not in lobby")
        }
    }

    async fn open_mods_dialog(&mut self) {
        let Some(mut playmode) = self.selected_mode.clone() else {
            NotificationManager::add_text_notification("Lobby does not currently have a playmode selected", 3000.0, Color::RED).await;
            return
        };

        // this shouldnt be necessary but just in case
        if let Some(map) = &self.selected_beatmap {
            playmode = map.check_mode_override(playmode.clone());
        }

        let groups = get_gamemode_info(&playmode)
            .map(|info|info.get_mods())
            .unwrap_or_default();

        self.actions.push(MenuMenuAction::AddDialog(Box::new(ModDialog::new(groups).await), false))
    }

    fn get_beatmap_button(&self, font_size: f32) -> IcedElement {
        use iced_elements::*;
        let Some(data) = &**self.lobby_info else { return EmptyElement.into_element() };
        let Some((lobby_map, mode)) = data.current_beatmap.as_ref().zip(self.selected_mode.as_ref()) else { 
            return Text::new("No Beatmap Selected").size(font_size).into_element()
        };
        
        let content = if let Some(beatmap) = self.selected_beatmap.as_ref().filter(|b|b.beatmap_hash == lobby_map.hash) {
            let mode = beatmap.check_mode_override(mode.clone());
            col!(
                Text::new(format!("{} - {}", beatmap.artist, beatmap.title)),
                Text::new(format!("{} by {}", beatmap.version, beatmap.creator)),

                // difficulty info
                diff_info(beatmap.clone(), mode, &self.current_mods)
                    .map(|s|Text::new(s).into_element())
                    .unwrap_or(EmptyElement.into_element());
                
            )
        } else {
            col!(
                // beatmap version string
                Text::new(lobby_map.title.clone()),
                // download prompt
                Text::new("Click here to open beatmap download page");
            )
        };

        Button::new(content)
            .on_press(Message::new_menu(self, "beatmap_select", MessageType::Click))
            .into_element()
    }
}

#[async_trait]
impl AsyncMenu for LobbyMenu {
    fn get_name(&self) -> &'static str { "lobby_menu" }

    async fn update(&mut self) -> Vec<MenuAction> {
        if self.init_pending {
            self.init_pending = false;
            self.refresh_data(&None).await;
        }
        
        // check for lobby updates
        let old_info = self.lobby_info.clone();
        if self.lobby_info.update() {
            self.refresh_data(&old_info).await;
            if self.lobby_info.is_none() { return self.actions.take() }
        }

        // // check audio state
        // let mut song_done = false;
        // match AudioManager::get_song().await {
        //     Some(song) => {
        //         if !song.is_playing() && !song.is_paused() { song_done = true; }
        //     }
        //     _ => song_done = true,
        // }

        // update our menu game
        self.gameplay_preview.update().await;

        self.actions.take()
    }

    fn view(&self, _values: &mut ShuntingYardValues) -> IcedElement {
        use iced_elements::*;
        
        row!(
            // player list
            col!(
                make_scrollable(
                    self.slots.iter().map(|s|s.view()).collect(), 
                    "lobby_user_list"
                );
                width = Fill,
                height = Fill
            ),

            // beatmap and gameplay preview
            col!(
                // beatmap button
                self.get_beatmap_button(30.0),

                // gameplay preview
                self.gameplay_preview.widget()
                ;
                width = Fill,
                height = Fill
            );
            width = Fill,
            height = Fill,
            spacing = 5.0
        )
    }
    
    async fn handle_message(&mut self, message: Message, values: &mut ShuntingYardValues) {
        let Some(tag) = message.tag.as_string() else { return }; 

        let slot = message.message_type.as_number().map(|n|n as u8);

        match (&*tag, slot) {
            ("beatmap_select", _) => {
                if self.is_host() {
                    let mut menu = BeatmapSelectMenu::new().await;
                    menu.select_action = BeatmapSelectAction::Back;
                    self.actions.push(MenuMenuAction::SetMenu(Box::new(menu)));
                } else {
                    // player, cant change map. check if we have it, and if not, open a page to download it
                    let hash = self.lobby_info().current_beatmap.as_ref().map(|b|&b.hash);

                    let current_map = CurrentBeatmapHelper::new().0.clone();
                    let current_hash = current_map.as_ref().map(|u|&u.beatmap_hash);
                    if hash.is_some() && hash != current_hash {
                        let hash = hash.unwrap().clone();
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
                }
            }
            
            ("start", _) if self.is_host() => { tokio::spawn(OnlineManager::lobby_map_start()); }
            ("ready", _) => { tokio::spawn(OnlineManager::update_lobby_state(LobbyUserState::Ready)); }
            ("unready", _) => { tokio::spawn(OnlineManager::update_lobby_state(LobbyUserState::NotReady)); }
            ("leave", _) => return self.quit_lobby().await,


            ("slot_status", Some(slot)) if self.is_host() => {
                let lobby_info = self.lobby_info();
                match lobby_info.slots.get(&slot) {
                    Some(LobbySlot::Empty)  => { tokio::spawn(OnlineManager::update_lobby_slot(slot, LobbySlot::Locked)); },
                    Some(LobbySlot::Locked) => { tokio::spawn(OnlineManager::update_lobby_slot(slot, LobbySlot::Empty)); },
                    // kick
                    Some(LobbySlot::Filled{user}) if &lobby_info.our_user_id != user => {
                        tokio::spawn(OnlineManager::update_lobby_slot(slot, LobbySlot::Locked));
                    }
                    _ => {}
                };
            }
            // move to slot
            ("slot", Some(slot)) => {
                let lobby_info = self.lobby_info();

                if let Some(s) = lobby_info.slots.get(&slot) {
                    if s.is_free() || (s.is_locked() && self.is_host()) {
                        tokio::spawn(OnlineManager::move_lobby_slot(slot));
                    }
                }
            }

            // open user profile
            ("slot_user", Some(slot)) => {
                let lobby_info = self.lobby_info();
                
                if let Some(LobbySlot::Filled { user }) = lobby_info.slots.get(&slot) {
                    let dialog = LobbyPlayerDialog::new(*user, *user == lobby_info.our_user_id, self.is_host());
                    self.actions.push(MenuMenuAction::AddDialog(Box::new(dialog), false));
                }
            }

            _ => {}
        }
    }




    //     if key == Key::Escape {
    //         self.quit_lobby(game).await;
    //     }

    //     if key == Key::M && mods.ctrl {
    //         self.open_mods_dialog(game).await;
    //     }
    // }

}

fn diff_info(beatmap: Arc<BeatmapMeta>, mode: String, mods: &ModManager) -> Option<String> {
    let info = get_gamemode_info(&mode)?; 
    let diff = get_diff(&beatmap, &mode, mods);
    let diff_meta = BeatmapMetaWithDiff::new(beatmap.clone(), diff);

    Some(info.get_diff_string(&diff_meta, mods))
}