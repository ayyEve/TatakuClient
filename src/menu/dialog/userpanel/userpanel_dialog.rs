use crate::prelude::*;


pub struct UserPanel {
    chat: Option<Chat>,

    /// user_id, user
    users: HashMap<u32, PanelUser>,

    should_close: bool
}
impl UserPanel {
    pub fn new() -> Self {
        Self {
            chat: None,
            users: HashMap::new(),
            should_close: false,
        }
    }
}

impl Dialog<Game> for UserPanel {
    fn should_close(&self) -> bool {self.should_close}
    fn get_bounds(&self) -> Rectangle {
        let window_size = Settings::window_size();
        Rectangle::bounds_only(
            Vector2::zero(), 
            window_size
        )
    }
    

    fn on_mouse_down(&mut self, pos:&Vector2, button:&MouseButton, mods:&KeyModifiers, game:&mut Game) -> bool {
        for (_, i) in self.users.iter_mut() {
            if i.on_click(*pos, *button, *mods) {
                // self.selected_user = Some(u.user_id);
                let user_id = i.user.user_id;
                let username = i.user.username.clone();

                // user menu dialog
                let mut user_menu_dialog = NormalDialog::new("User Options");

                // if u.game.starts_with("Tataku") {
                    user_menu_dialog.add_button("Spectate", Box::new(move |dialog, game| {
                        OnlineManager::start_spectating(user_id);
                        //TODO: wait for a spec response from the server before setting the mode
                        game.queue_state_change(GameState::Spectating(SpectatorManager::new()));
                        dialog.should_close = true;
                    }));
                // }

                let clone = ONLINE_MANAGER.clone();
                user_menu_dialog.add_button("Send Message", Box::new(move |dialog, game| {
                    if let Some(chat) = Chat::new() {
                        game.add_dialog(Box::new(chat));
                    }
                    
                    let username = username.clone();
                    let clone = clone.clone();
                    tokio::spawn(async move {
                        let mut lock = clone.lock().await;
                        let channel = ChatChannel::User{username};
                        if !lock.chat_messages.contains_key(&channel) {
                            lock.chat_messages.insert(channel.clone(), Vec::new());
                        }
                    });

                    dialog.should_close = true;
                }));

                user_menu_dialog.add_button("Close", Box::new(|dialog, _game| {
                    dialog.should_close = true;
                }));

                game.add_dialog(Box::new(user_menu_dialog));
            }
        }
        true
    }

    fn on_mouse_move(&mut self, pos:&Vector2, _g:&mut Game) {
        for (_, i) in self.users.iter_mut() {
            i.on_mouse_move(*pos)
        }
    }

    fn update(&mut self, _game:&mut Game) {

        // update users from online manager
        if let Ok(om) = ONLINE_MANAGER.try_lock() {
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



    fn draw(&mut self, args:&RenderArgs, depth: &f64, list: &mut Vec<Box<dyn Renderable>>) {
        //TODO: move the set_pos code to update or smth
        let mut counter = 0;
        
        for (_, u) in self.users.iter_mut() {
            let users_per_col = 2;
            let x = USER_ITEM_SIZE.x * (counter % users_per_col) as f64;
            let y = USER_ITEM_SIZE.y * (counter / users_per_col) as f64;
            u.set_pos(Vector2::new(x, y));

            counter += 1;
            u.draw(*args, Vector2::zero(), *depth, list);
        }
        
    }

}