use crate::prelude::*;

lazy_static::lazy_static! {
    static ref PANEL_QUEUE: Arc<(Mutex<MultiFuze<UserPanelEvent>>, AsyncMutex<MultiBomb<UserPanelEvent>>)> = {
        let (fuze, bomb) = MultiBomb::new();
        let fuze = Mutex::new(fuze);
        let bomb = AsyncMutex::new(bomb);

        Arc::new((fuze, bomb))
    };
}

pub struct UserPanel {
    chat: Chat,

    /// user_id, user
    users: HashMap<u32, PanelUser>,

    should_close: bool,
    window_size: Arc<WindowSize>
}
impl UserPanel {
    pub fn new() -> Self {
        Self {
            chat: Chat::new(),
            users: HashMap::new(),
            should_close: false,
            window_size: WindowSize::get(),
        }
    }
}

#[async_trait]
impl Dialog<Game> for UserPanel {
    fn name(&self) -> &'static str { "UserPanel" }
    fn should_close(&self) -> bool { self.should_close }
    fn get_bounds(&self) -> Bounds { Bounds::new(Vector2::ZERO, self.window_size.0) }
    async fn force_close(&mut self) { self.should_close = true; }
    
    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.window_size = window_size;
    }
    
    async fn on_key_press(&mut self, key:Key, mods:&KeyModifiers, game:&mut Game) -> bool {
        self.chat.on_key_press(key, mods, game).await;
        true
    }
    async fn on_key_release(&mut self, key:Key, mods:&KeyModifiers, game:&mut Game) -> bool {
        self.chat.on_key_release(key, mods, game).await;
        true
    }
    async fn on_text(&mut self, text:&String) -> bool {
        self.chat.on_text(text).await
    }

    async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, game:&mut Game) -> bool {
        self.chat.on_mouse_down(pos, button, mods, game).await;
        for (_, i) in self.users.iter_mut() {
            if i.on_click(pos, button, *mods) {
                let user_id = i.user.user_id;
                let username = i.user.username.clone();

                // user menu dialog
                let mut user_menu_dialog = GenericDialog::new("User Options");

                // spectate
                if i.user.game.starts_with("Tataku") {
                    user_menu_dialog.add_button("Spectate", Box::new(move |dialog, _game| {
                        OnlineManager::start_spectating(user_id);
                        dialog.should_close = true;
                    }));
                }

                // message
                user_menu_dialog.add_button("Send Message", Box::new(move |dialog, _game| {
                    PANEL_QUEUE.0.lock().ignite(UserPanelEvent::OpenChat(username.clone()));
                    dialog.should_close = true;
                }));

                // add/remove friend
                let is_friend = ONLINE_MANAGER.read().await.friends.contains(&user_id);
                let friend_txt = if is_friend {"Remove Friend"} else {"Add Friend"};
                user_menu_dialog.add_button(friend_txt, Box::new(move |dialog, _game| {
                    PANEL_QUEUE.0.lock().ignite(UserPanelEvent::AddRemoveFriend(user_id));
                    dialog.should_close = true;
                }));

                // invite to lobby
                if let Some(_lobby) = &*CurrentLobbyInfo::get() {
                    user_menu_dialog.add_button("Invite to Lobby", Box::new(move |dialog, _game| {
                        tokio::spawn(OnlineManager::invite_user(user_id));
                        dialog.should_close = true;
                    }));
                }


                // close menu
                user_menu_dialog.add_button("Close", Box::new(|dialog, _game| {
                    dialog.should_close = true;
                }));

                game.add_dialog(Box::new(user_menu_dialog), false);
            }
        }
        true
    }
    async fn on_mouse_up(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, game:&mut Game) -> bool {
        self.chat.on_mouse_up(pos, button, mods, game).await;
        true
    }
    async fn on_mouse_scroll(&mut self, delta:f32, game:&mut Game) -> bool {
        self.chat.on_mouse_scroll(delta, game).await
    }

    async fn on_mouse_move(&mut self, pos:Vector2, game:&mut Game) {
        self.chat.on_mouse_move(pos, game).await;

        for (_, i) in self.users.iter_mut() {
            i.on_mouse_move(pos)
        }
    }

    async fn update(&mut self, game:&mut Game) {
        self.chat.update(game).await;

        let mut bomb = PANEL_QUEUE.1.lock().await;
        while let Some(event) = bomb.exploded() {
            match event {
                UserPanelEvent::OpenChat(username) => self.chat.selected_channel = Some(ChatChannel::from_name(username)),

                UserPanelEvent::AddRemoveFriend(friend_id) => {
                    let manager = ONLINE_MANAGER.read().await;
                    let is_friend = !manager.friends.contains(&friend_id);

                    use futures_util::SinkExt;
                    send_packet!(manager.writer, create_packet!(PacketId::Client_UpdateFriend {friend_id, is_friend}));
                }
            }
        }


        // update users from online manager
        if let Ok(om) = ONLINE_MANAGER.try_read() {
            for (_, user) in &om.users {
                if let Ok(u) = user.try_lock() {
                    if !self.users.contains_key(&u.user_id) {
                        self.users.insert(u.user_id, PanelUser::new(u.clone()));
                    } else {
                        self.users.get_mut(&u.user_id).unwrap().user = u.clone()
                    }
                }
            }
        }
    }

    async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
        self.chat.draw(offset, list).await;
        //TODO: move the set_pos code to update or smth
        let mut counter = 0;
        
        for (_, u) in self.users.iter_mut() {
            let users_per_col = 2;
            let x = USER_ITEM_SIZE.x * (counter % users_per_col) as f32;
            let y = USER_ITEM_SIZE.y * (counter / users_per_col) as f32;
            u.set_pos(Vector2::new(x, y));

            counter += 1;
            u.draw(offset, list);
        }
    }
    
}

#[derive(Clone)]
pub enum UserPanelEvent {
    OpenChat(String),
    AddRemoveFriend(u32)
}
