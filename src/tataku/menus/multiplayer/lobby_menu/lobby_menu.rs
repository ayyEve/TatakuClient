use crate::prelude::*;
use tokio::sync::mpsc::{Receiver, channel};

const SCORE_SEND_TIME:f32 = 1_000.0;
const CHANNEL_COUNT:usize = 2;

pub struct LobbyMenu {
    init_pending: bool,
    lobby_info: CurrentLobbyDataHelper,

    left_scrollable: ScrollableArea,
    right_scrollable: ScrollableArea,

    selected_beatmap: Option<Arc<BeatmapMeta>>,
    selected_mode: Option<String>,
    current_mods: ModManagerHelper,
    latest_beatmap_helper: LatestBeatmapHelper,

    menu: Option<Box<dyn AsyncMenu<Game>>>,
    on_beatmap_select: Option<Receiver<Option<(Arc<BeatmapMeta>, String)>>>,
    on_score_menu_close: Option<AsyncReceiver<()>>,

    beatmap_loader: Option<AsyncLoader<TatakuResult<IngameManager>>>,
    load_complete_sent: bool,
    manager: Option<Box<IngameManager>>,
    score_send_timer: Instant,
    
    menu_game: MenuGameHelper,

    since_last_escape: Instant,

    slot_senders: HashMap<u8, (AsyncSender<(LobbySlot, bool)>, AsyncSender<Option<LobbyPlayerInfo>>)>,
}
impl LobbyMenu {
    pub async fn new() -> Self {
        let lobby_info = CurrentLobbyDataHelper::new();
        let window_size = WindowSize::get().0;
        let left_size = Vector2::new(window_size.x / 3.0, window_size.y);
        let mut right_size = Vector2::new(window_size.x * (2.0/3.0) - 10.0, window_size.y);

        let mut left_scrollable = ScrollableArea::new(Vector2::ZERO, left_size, ListMode::VerticalList);
        let mut right_scrollable = ScrollableArea::new(left_size.x_portion() + 10.0, right_size, ListMode::VerticalList);
        right_scrollable.add_item(Box::new(BeatmapSelectButton::new(right_size.x)));


        {
            let mut buttons = ScrollableArea::new(Vector2::ZERO, Vector2::new(right_size.x - 10.0, 50.0), ListMode::Grid(GridSettings::new(Vector2::new(5.0, 0.0), HorizontalAlign::Center)));
            buttons.add_item(Box::new(MenuButton::new(Vector2::ZERO, Vector2::new(100.0, 50.0), "Leave", Font::Main).with_tag("leave")));
            buttons.add_item(Box::new(LobbyReadyButton::new()));
            // buttons.add_item(Box::new(MenuButton::new(Vector2::ZERO, Vector2::new(100.0, 50.0), "Start", Font::Main).with_tag("start")));
            right_scrollable.add_item(Box::new(buttons));
        }

        let mut slot_senders = HashMap::new();
        for slot in 0..16 {
            let (state_sender, state_receiver) = async_channel(CHANNEL_COUNT);
            let (player_sender, player_receiver) = async_channel(CHANNEL_COUNT);
            
            left_scrollable.add_item(Box::new(LobbySlotDisplay::new(left_size.x, slot, state_receiver, player_receiver)));
            slot_senders.insert(slot, (state_sender, player_sender));
        }

        right_size.y = right_scrollable.get_elements_height();
        let menu_game_bounds = Bounds::new(
            Vector2::new(left_size.x, right_size.y) + Vector2::ONE * 10.0, 
            Vector2::new(right_size.x, window_size.y / 2.0) - Vector2::ONE * 10.0
        );
        let mut menu_game = MenuGameHelper::new(true, true, Box::new(|s|s.background_game_settings.multiplayer_menu_enabled));
        menu_game.fit_to_area(menu_game_bounds).await;
        Self {
            init_pending: true,
            slot_senders,

            lobby_info,
            left_scrollable,
            right_scrollable,
            selected_beatmap: None,
            selected_mode: Some(CurrentPlaymodeHelper::new().0.clone()),
            
            latest_beatmap_helper: LatestBeatmapHelper::new(),
            menu_game,
            since_last_escape: Instant::now(),

            menu: None,
            on_beatmap_select: None,
            on_score_menu_close: None,
            current_mods: ModManagerHelper::new(),
            beatmap_loader: None,
            load_complete_sent: false,
            manager: None,
            score_send_timer: Instant::now(),
        }
    }

    pub async fn refresh_data(&mut self, old_info: &Option<CurrentLobbyInfo>, game: &mut Game) {
        let Some(info) = &**self.lobby_info else { info!("no current lobby, leaving multiplayer"); self.quit_lobby(game).await; return };

        // update slots
        for (slot, state) in info.slots.iter() {
            let (status, player) = self.slot_senders.get(slot).unwrap();
            let is_host = if let LobbySlot::Filled { user } = &state { user == &info.host } else { false };
            status.try_send((state.clone(), is_host)).unwrap();

            if let LobbySlot::Filled { user } = state {
                let username = info.player_usernames.get(user).cloned().unwrap_or("?".to_owned());
                let Some(lobby_user) = info.players.iter().find(|u|&u.user_id == user) else {
                    warn!("couldnt find lobby user, probably in a bad state. lobby info below:");
                    warn!("{info:#?}");
                    continue;
                };
                player.try_send(Some(LobbyPlayerInfo::new(lobby_user.clone(), username))).unwrap();
            } else {
                player.try_send(None).unwrap();
            }
        }
        
        // update beatmap
        if old_info.as_ref().map(|i|&i.current_beatmap) != Some(&info.current_beatmap) {
            if let Some(beatmap) = &info.current_beatmap {
                self.selected_beatmap = BEATMAP_MANAGER.read().await.get_by_hash(&beatmap.hash);
                self.selected_mode = Some(beatmap.mode.clone());
                GlobalValueManager::update(Arc::new(CurrentPlaymode(beatmap.mode.clone())));

                if let Some(beatmap) = &self.selected_beatmap {
                    BEATMAP_MANAGER.write().await.set_current_beatmap(game, beatmap, true).await;
                } else {
                    BEATMAP_MANAGER.write().await.remove_current_beatmap(game).await;
                }

                let new_state = match self.selected_beatmap {
                    Some(_) => LobbyUserState::NotReady,
                    None => LobbyUserState::NoMap,
                };
                tokio::spawn(OnlineManager::update_lobby_state(new_state));
            } else {
                BEATMAP_MANAGER.write().await.remove_current_beatmap(game).await;
                tokio::spawn(OnlineManager::update_lobby_state(LobbyUserState::NoMap));
            }
        }
        
        // update manager's score list
        if let Some(manager) = &mut self.manager {
            manager.score_list = info.player_scores.iter()
                .filter(|(u,_)|u != &&info.our_user_id) // make sure we dont re-add our own score in
                .map(|(_,s)|IngameScore::new(s.clone(), false, false)).collect();
            manager.score_list.sort_by(|a, b|b.score.score.cmp(&a.score.score));
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
        if info.should_play && self.manager.is_none() {
            if let Some(loader) = &self.beatmap_loader {
                if let Some(Ok(mut manager)) = loader.check().await {
                    manager.make_multiplayer();
                    manager.start().await;

                    self.manager = Some(Box::new(manager));
                    self.score_send_timer = Instant::now();
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

    async fn quit_lobby(&mut self, game: &mut Game) {
        tokio::spawn(OnlineManager::leave_lobby());
        game.queue_state_change(GameState::InMenu(Box::new(LobbySelect::new().await)));
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

    async fn open_mods_dialog(&self, game: &mut Game) {
        let Some(mut playmode) = self.selected_mode.clone() else {
            NotificationManager::add_text_notification("Lobby does not currently have a playmode selected", 3000.0, Color::RED).await;
            return
        };

        // this shouldnt be necessary but just in case
        if let Some(map) = &self.selected_beatmap {
            playmode = map.check_mode_override(playmode.clone());
        }

        let mut groups = Vec::new();
        if let Some(info) = get_gamemode_info(&playmode) {
            groups = info.get_mods();
        }

        game.add_dialog(Box::new(ModDialog::new(groups).await), false)
    }
}

#[async_trait]
impl AsyncMenu<Game> for LobbyMenu {
    fn get_name(&self) -> &str { "lobby_menu" }

    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        if let Some(manager) = &mut self.manager {
            manager.window_size_changed(window_size.clone()).await;
        }

        if let Some(menu) = &mut self.menu {
            menu.window_size_changed(window_size.clone()).await;
        }

        // update the scrollable sizes
        let left_size = Vector2::new(window_size.x / 3.0, window_size.y);
        let mut right_size = Vector2::new(window_size.x * (2.0/3.0) - 10.0, window_size.y / 2.0);
        
        self.left_scrollable.set_size(left_size);
        self.right_scrollable.set_pos(left_size.x_portion() + 10.0);
        self.right_scrollable.set_size(right_size);

        // beatmap select button
        {
            let btn = self.right_scrollable.items.get_mut(0).unwrap();
            btn.set_size(Vector2::new(right_size.x, btn.size().y));
        }
        // other buttons
        self.right_scrollable.items.get_mut(1).unwrap().set_size(Vector2::new(right_size.x - 10.0, 50.0));

        self.right_scrollable.refresh_layout();

        right_size.y = self.right_scrollable.get_elements_height();
        let menu_game_bounds = Bounds::new(
            Vector2::new(left_size.x, right_size.y) + Vector2::ONE * 10.0, 
            Vector2::new(right_size.x, window_size.y / 2.0) - Vector2::ONE * 10.0
        );
        self.menu_game.fit_to_area(menu_game_bounds).await;
    }

    async fn update(&mut self, game:&mut Game) {
        if self.init_pending {
            self.init_pending = false;
            self.refresh_data(&None, game).await;
        }
        
        // check for lobby updates
        let old_info = self.lobby_info.clone();
        if self.lobby_info.update() {
            self.refresh_data(&old_info, game).await;
            if self.lobby_info.is_none() { return }
        }

        // make sure these always run, or the mpsc channels will fill up and crash
        self.left_scrollable.update();
        self.right_scrollable.update();

        // if ingame, update manager
        if let Some(manager) = &mut self.manager {
            manager.update().await;

            if manager.completed {
                let score = manager.score.score.clone();
                tokio::spawn(OnlineManager::lobby_map_complete(score));
                tokio::spawn(OnlineManager::update_lobby_state(LobbyUserState::NotReady));

                // save and submit score
                game.ingame_complete(manager).await;

                // TODO: show score menu
                let mut score_menu = ScoreMenu::new(&manager.score, manager.beatmap.get_beatmap_meta(), false);
                let (sender, receiver) = async_channel(1);
                score_menu.make_lobby(sender);
                score_menu.on_change(true).await;
                self.menu = Some(Box::new(score_menu));
                self.on_score_menu_close = Some(receiver);

                // close manager
                self.manager = None;
            } else if self.score_send_timer.as_millis() >= SCORE_SEND_TIME {
                self.score_send_timer.elapsed_and_reset();
                let score = manager.score.score.clone();
                tokio::spawn(OnlineManager::lobby_update_score(score));
            }

            return;
        }

        // check audio state
        let mut song_done = false;
        match AudioManager::get_song().await {
            Some(song) => {
                if !song.is_playing() && !song.is_paused() { song_done = true; }
            }
            _ => song_done = true,
        }
        if song_done {
            if let Some(audio) = AudioManager::get_song().await {
                audio.play(true);
                self.menu_game.setup().await;
            }
        }

        // update our menu game
        self.menu_game.update().await;

        // if we're waiting to select a beatmap, check if its been selected
        if let Some(result) = self.on_beatmap_select.as_mut().and_then(|r|r.try_recv().ok()) {
            self.on_beatmap_select = None;
            self.menu = None;

            // only update the map if a map was selected (result is none if back/esc was pressed)
            if let Some((map, mode)) = result {
                // technically we should wait for the server's reply, hense why these are commented out
                // self.selected_beatmap = Some(map.clone());
                // self.selected_mode = Some(mode.clone());
                tokio::spawn(OnlineManager::update_lobby_beatmap(map, mode));
            } else if let Some(beatmap) = &self.selected_beatmap {
                // if nothing was selected, make sure we revert back to the previous beatmap
                BEATMAP_MANAGER.write().await.set_current_beatmap(game, beatmap, true).await;
            }
        }

        // if we're waiting for the score menu to close, check if its been closed
        if let Some(_) = self.on_score_menu_close.as_mut().and_then(|r|r.try_recv().ok()) {
            self.on_score_menu_close = None;
            self.menu = None;
        }

        // if we have a menu, update it
        if let Some(menu) = &mut self.menu {
            menu.update(game).await;
        }

        // if we're loading the beatmap, check if its done
        if let Some(loader) = &self.beatmap_loader {
            if !self.load_complete_sent && loader.is_complete() {
                self.load_complete_sent = true;
                tokio::spawn(OnlineManager::lobby_load_complete());
            }
        }

        // our mods changed, let the lobby know
        if self.current_mods.update() {
            let mods = self.current_mods.mods.clone();
            let speed = self.current_mods.speed;
            tokio::spawn(OnlineManager::lobby_update_mods(mods, speed));
        }
    
        // check if a new beatmap was added
        if self.latest_beatmap_helper.update() {
            // if the map that was just added is the lobby's map, set it as our current map
            let lobby_info = self.lobby_info();
            if let Some(beatmap) = &lobby_info.current_beatmap {

                let mut beatmap_manager = BEATMAP_MANAGER.write().await;
                if let Some(beatmap) = beatmap_manager.get_by_hash(&beatmap.hash) {
                    beatmap_manager.set_current_beatmap(game, &beatmap, true).await;
                    self.selected_beatmap = Some(beatmap);
                    
                    tokio::spawn(OnlineManager::update_lobby_state(LobbyUserState::NotReady));
                }
            }
        }
    }


    async fn draw(&mut self, list: &mut RenderableCollection) {
        if let Some(manager) = &mut self.manager {
            manager.draw(list).await;
            return;
        }

        if let Some(menu) = &mut self.menu {
            menu.draw(list).await;
            return;
        }

        self.menu_game.draw(list).await;

        self.left_scrollable.draw(Vector2::ZERO, list);
        self.right_scrollable.draw(Vector2::ZERO, list);
    }


    async fn on_text(&mut self, text:String) {
        if let Some(manager) = &mut self.manager {
            manager.on_text(&text, &KeyModifiers::default()).await;
            return;
        }

        if let Some(menu) = &mut self.menu {
            menu.on_text(text.clone()).await;
            return;
        }

        self.left_scrollable.on_text(text.clone());
        self.right_scrollable.on_text(text);
    }
    async fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
        if let Some(manager) = &mut self.manager {
            manager.mouse_down(button).await;
            return;
        }

        if let Some(menu) = &mut self.menu {
            menu.on_click(pos, button, mods, game).await;
            return;
        }

        if let Some(tag) = self.left_scrollable.on_click_tagged(pos, button, mods) {
            if tag.starts_with("slot_status_") {
                let slot = tag.trim_start_matches("slot_status_").parse::<u8>().unwrap();
                let lobby_info = self.lobby_info();
                if self.is_host() {
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
            } else if tag.starts_with("slot_") {
                let slot = tag.trim_start_matches("slot_").parse::<u8>().unwrap();
                let lobby_info = self.lobby_info();
                match (button, self.is_host(), lobby_info.slots.get(&slot)) {
                    // left click, move to slot
                    (MouseButton::Left, _, Some(LobbySlot::Empty)) | (MouseButton::Left, true, Some(LobbySlot::Locked)) => {
                        tokio::spawn(OnlineManager::move_lobby_slot(slot));
                    }
                    // right click, user status
                    (MouseButton::Right, host, Some(LobbySlot::Filled { user })) => {
                        let dialog = LobbyPlayerDialog::new(*user, *user == lobby_info.our_user_id, host);
                        game.add_dialog(Box::new(dialog), false);
                    }
                    _ => {}
                }
                
            }
        }
        
        if let Some(tag) = self.right_scrollable.on_click_tagged(pos, button, mods) {
            // info!("clicked: {tag}");
            match &*tag {
                "beatmap_select" => {
                    if self.is_host() {
                        let mut menu = BeatmapSelectMenu::new().await;
                        let (sender, receiver) = channel(1);
                        menu.select_action = BeatmapSelectAction::OnComplete(sender);
                        menu.on_change(true).await;

                        self.menu = Some(Box::new(menu));
                        self.on_beatmap_select = Some(receiver);
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

                                        #[allow(unused)]
                                        #[derive(Deserialize)]
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

                "start" if self.is_host() => { tokio::spawn(OnlineManager::lobby_map_start()); }
                "ready" => { tokio::spawn(OnlineManager::update_lobby_state(LobbyUserState::Ready)); }
                "unready" => { tokio::spawn(OnlineManager::update_lobby_state(LobbyUserState::NotReady)); }

                "leave" => {
                    self.quit_lobby(game).await;
                    return;
                }
                _ => {}
            }
        }
    }
    async fn on_click_release(&mut self, pos:Vector2, button:MouseButton, game:&mut Game) {
        if let Some(manager) = &mut self.manager {
            manager.mouse_up(button).await;
            return;
        }
        
        if let Some(menu) = &mut self.menu {
            menu.on_click_release(pos, button, game).await;
            return;
        }
        self.left_scrollable.on_click_release(pos, button);
        self.right_scrollable.on_click_release(pos, button);
    }

    async fn on_scroll(&mut self, delta:f32, game:&mut Game) {
        if let Some(manager) = &mut self.manager {
            manager.mouse_scroll(delta).await;
            return;
        }

        if let Some(menu) = &mut self.menu {
            menu.on_scroll(delta, game).await;
            return;
        }
        if self.left_scrollable.get_hover() { self.left_scrollable.on_scroll(delta); }
        if self.right_scrollable.get_hover() { self.right_scrollable.on_scroll(delta); }
    }
    async fn on_mouse_move(&mut self, pos:Vector2, game:&mut Game) {
        if let Some(manager) = &mut self.manager {
            manager.mouse_move(pos).await;
            return;
        }

        if let Some(menu) = &mut self.menu {
            menu.on_mouse_move(pos, game).await;
        }
        self.left_scrollable.on_mouse_move(pos);
        self.right_scrollable.on_mouse_move(pos);
    }
    async fn on_key_press(&mut self, key:Key, game:&mut Game, mods:KeyModifiers) {
        if let Some(manager) = &mut self.manager {
            // if ingame and escape is pressed
            if key == Key::Escape {
                if self.since_last_escape.elapsed_and_reset() < 1_000.0 {
                    self.quit_lobby(game).await;
                    return;
                } else {
                    NotificationManager::add_text_notification("Press escape again to quit the lobby", 3_000.0, Color::BLUE).await;
                }
            }

            manager.key_down(key, mods).await;
            return;
        }

        if let Some(menu) = &mut self.menu {
            menu.on_key_press(key, game, mods).await;
            return;
        }

        if key == Key::Escape {
            self.quit_lobby(game).await;
        }

        if key == Key::M && mods.ctrl {
            self.open_mods_dialog(game).await;
        }
    }
    async fn on_key_release(&mut self, key:Key, game:&mut Game) {
        if let Some(manager) = &mut self.manager {
            manager.key_up(key).await;
            return;
        }

        if let Some(menu) = &mut self.menu {
            menu.on_key_release(key, game).await;
            return;
        }
    }
}
impl ControllerInputMenu<Game> for LobbyMenu {}
