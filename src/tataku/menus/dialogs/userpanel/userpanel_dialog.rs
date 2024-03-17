use crate::prelude::*;
pub struct UserPanel {
    num: usize,
    actions: ActionQueue,

    chat: Chat,

    /// user_id, user
    users: HashMap<u32, PanelUser>,

    should_close: bool,
    // window_size: Arc<WindowSize>
}
impl UserPanel {
    pub fn new() -> Self {
        Self {
            num: 0,
            actions: ActionQueue::new(),

            chat: Chat::new(),
            users: HashMap::new(),
            should_close: false,
            // window_size: WindowSize::get(),
        }
    }
}

#[async_trait]
impl Dialog for UserPanel {
    fn name(&self) -> &'static str { "UserPanel" }
    fn get_num(&self) -> usize { self.num }
    fn set_num(&mut self, num: usize) { self.num = num }

    fn should_close(&self) -> bool { self.should_close }
    // fn get_bounds(&self) -> Bounds { Bounds::new(Vector2::ZERO, self.window_size.0) }
    async fn force_close(&mut self) { self.should_close = true; }
    
    async fn handle_message(&mut self, message: Message, _values: &mut ValueCollection) {
        let Some(tag) = message.tag.as_string() else { return }; 


        match &*tag {
            "user" => {
                let user = message.message_type.downcast::<OnlineUser>();

                let user_id = user.user_id;
                let username = user.username.clone();

                // user menu dialog
                let mut user_menu_dialog = GenericDialog::new("User Options");

                let owner = MessageOwner::new_dialog(self);

                // spectate
                if user.game.starts_with("Tataku") {
                    user_menu_dialog.add_button("Spectate", Arc::new(move |dialog| {
                        OnlineManager::start_spectating(user_id);
                        dialog.should_close = true;
                        None
                    }));
                }

                // message
                user_menu_dialog.add_button("Send Message", Arc::new(move |dialog| {
                    dialog.add_action(GameAction::HandleMessage(Message::new(
                        owner,
                        "open_chat",
                        MessageType::Text(username.clone())
                    )));
                    dialog.should_close = true;
                    None
                }));

                // add/remove friend
                let is_friend = OnlineManager::get().await.friends.contains(&user_id);
                let friend_txt = if is_friend {"Remove Friend"} else {"Add Friend"};
                user_menu_dialog.add_button(friend_txt, Arc::new(move |dialog| {
                    dialog.add_action(GameAction::HandleMessage(Message::new(
                        owner,
                        "add_remove_friend",
                        MessageType::Number(user_id as usize)
                    )));
                    dialog.should_close = true;
                    None
                }));

                // invite to lobby
                user_menu_dialog.add_button("Invite to Lobby", Arc::new(move |dialog| {
                    dialog.add_action(MultiplayerAction::InviteUser{ user_id });
                    dialog.should_close = true;
                    None
                }));


                // close menu
                user_menu_dialog.add_button("Close", Arc::new(|dialog| {
                    dialog.should_close = true;
                    None
                }));

                // self.actions.push(MenuMenuAction::AddDialog(Box::new(user_menu_dialog), false));
            }

            "open_chat" => {
                let Some(username) = message.message_type.as_text() else { return };
                self.chat.selected_channel = Some(ChatChannel::from_name(username))
            }

            "add_remove_friend" => {
                let MessageType::Number(friend_id) = message.message_type else { return };
                let friend_id = friend_id as u32;

                let manager = OnlineManager::get().await;
                let is_friend = !manager.friends.contains(&friend_id);

                manager.send_packet(ChatPacket::Client_UpdateFriend {friend_id, is_friend}).await;
            }
            _ => {}
        }

    }


    fn view(&self) -> IcedElement {
        use iced_elements::*;
        
        col!(
            //TODO:!!!!!!!!!!!!!!!!!!!!!! add panel

            self.chat.view();

            width = Fill, 
            height = Fill
        )
    }
    
    async fn update(&mut self, values: &mut ValueCollection) -> Vec<TatakuAction> { 
        self.chat.update(values).await;
        
        // update users from online manager
        if let Some(om) = OnlineManager::try_get() {
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

        self.actions.take() 
    }

    // async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
    //     self.chat.draw(offset, list).await;
    //     //TODO: move the set_pos code to update or smth
    //     let mut counter = 0;
        
    //     for (_, u) in self.users.iter_mut() {
    //         let users_per_col = 2;
    //         let x = USER_ITEM_SIZE.x * (counter % users_per_col) as f32;
    //         let y = USER_ITEM_SIZE.y * (counter / users_per_col) as f32;
    //         u.set_pos(Vector2::new(x, y));

    //         counter += 1;
    //         u.draw(offset, list);
    //     }
    // }
    
}

#[derive(Clone)]
pub enum UserPanelEvent {
    OpenChat(String),
    AddRemoveFriend(u32)
}
