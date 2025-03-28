use crate::prelude::*;
use crate::prelude::ui::*;

pub struct UserPanel {
    actions: ActionQueue,

    chat: Chat,

    /// user_id, user
    users: HashMap<u32, PanelUser>,
    // window_size: Arc<WindowSize>

    node: Box<dyn Widget>,
    node_id: NodeId,

    owner: MessageOwner,
}
impl UserPanel {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            actions: ActionQueue::new(),

            chat: Chat::new(),
            users: HashMap::new(),
            // window_size: WindowSize::get(),

            node: EmptyWidget::new_boxed(),
            node_id: EMPTY_NODE,
            owner: MessageOwner::Menu,
        }
    }


    fn build_view(&self) -> Box<dyn Widget> {
        EmptyWidget::new_boxed()
        // col!(
        //     //TODO:!!!!!!!!!!!!!!!!!!!!!! add panel

        //     // self.chat.view(values);

        // ).boxed()
    }
}

#[async_trait]
impl Widget for UserPanel {
    fn name(&self) -> Cow<'static, str> { "user_panel".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId>  {
        let child = self.node.layout(shell)?;
        self.owner = shell.owner;
        
        self.node_id = shell.tree.new_with_children(
            Style::default(), 
            &[ child ]
        )?;

        Ok(self.node_id)
    }

    fn draw(
        &self, 
        shell: &mut DrawShell<'_>, 
    ) {
        self.node.draw(shell);
    }

    async fn handle_message(
        &mut self, 
        message: &Message, 
        _values: &mut dyn Reflect,
        _actions: &mut ActionQueue,
    ) {
        let Some(tag) = message.tag.as_string() else { return }; 

        let owner = self.owner;
        match &**tag {
            "user" => {
                let user = message.value.downcast::<OnlineUser>();

                let user_id = user.user_id;
                let username = user.username.clone();

                // user menu dialog
                let mut user_menu_dialog = GenericDialog::new("User Options");
                let node_id = self.node_id;

                // spectate
                if user.game.starts_with("Tataku") {
                    user_menu_dialog.add_button("Spectate", Arc::new(move |_, actions| {
                        OnlineManager::start_spectating(user_id);
                        actions.push(UiAction::new(node_id, DialogAction::Close));
                        None
                    }));
                }

                // message
                user_menu_dialog.add_button("Send Message", Arc::new(move |_, actions| {
                    actions.push(GameAction::HandleMessage(Message::new(
                        owner,
                        "open_chat",
                        MessageValue::Text(username.clone())
                    )));
                    actions.push(UiAction::new(node_id, DialogAction::Close));
                    None
                }));

                // add/remove friend
                let is_friend = false;  // FIXME: OnlineManager::get().await.friends.contains(&user_id);
                let friend_txt = if is_friend {"Remove Friend"} else {"Add Friend"};
                user_menu_dialog.add_button(friend_txt, Arc::new(move |_, actions| {
                    actions.push(GameAction::HandleMessage(Message::new(
                        owner,
                        "add_remove_friend",
                        MessageValue::Number(user_id as usize)
                    )));
                    actions.push(UiAction::new(node_id, DialogAction::Close));
                    None
                }));

                // invite to lobby
                user_menu_dialog.add_button("Invite to Lobby", Arc::new(move |_, actions| {
                    actions.push(MultiplayerAction::InviteUser{ user_id });
                    actions.push(UiAction::new(node_id, DialogAction::Close));
                    None
                }));


                // close menu
                user_menu_dialog.add_button("Close", Arc::new(move |_, actions| {
                    actions.push(UiAction::new(node_id, DialogAction::Close));
                    None
                }));

                // self.actions.push(MenuMenuAction::AddDialog(Box::new(user_menu_dialog), false));
            }

            "open_chat" => {
                let Some(username) = message.value.as_text_ref() else { return };
                self.chat.selected_channel = Some(ChatChannel::from_name(username.clone()))
            }

            "add_remove_friend" => {
                let MessageValue::Number(friend_id) = message.value else { return };
                let friend_id = friend_id as u32;

                // FIXME:
                // let mut manager = OnlineManager::get_mut().await;
                // let is_friend = !manager.friends.contains(&friend_id);

                // manager.send_packet(ChatPacket::Client_UpdateFriend {friend_id, is_friend}).await;
            }
            _ => {}
        }

    }

    
    fn update(
        &mut self, 
        shell: &mut UpdateShell<'_>,
        actions: &mut ActionQueue,
    ) { 
        self.chat.update(shell, actions);
        
        // FIXME:
        // // update users from online manager
        // if let Some(om) = OnlineManager::try_get() {
        //     for user in om.users.values() {
        //         if let Ok(u) = user.try_lock() {
        //             if let std::collections::hash_map::Entry::Vacant(e) = self.users.entry(u.user_id) {
        //                 e.insert(PanelUser::new(u.clone()));
        //             } else {
        //                 self.users.get_mut(&u.user_id).unwrap().user = u.clone()
        //             }
        //         }
        //     }
        // }

        actions.extend(self.actions.take())
    }
}

// impl Dialog for UserPanel {
//     fn get_num(&self) -> usize { self.num }
//     fn set_num(&mut self, num: usize) { self.num = num }

//     fn should_close(&self) -> bool { self.should_close }
//     // fn get_bounds(&self) -> Bounds { Bounds::new(Vector2::ZERO, self.window_size.0) }
//     fn force_close(&mut self) { self.should_close = true; }
    

//     // async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
//     //     self.chat.draw(offset, list).await;
//     //     //TODO: move the set_pos code to update or smth
//     //     let mut counter = 0;
        
//     //     for (_, u) in self.users.iter_mut() {
//     //         let users_per_col = 2;
//     //         let x = USER_ITEM_SIZE.x * (counter % users_per_col) as f32;
//     //         let y = USER_ITEM_SIZE.y * (counter / users_per_col) as f32;
//     //         u.set_pos(Vector2::new(x, y));

//     //         counter += 1;
//     //         u.draw(offset, list);
//     //     }
//     // }
    
// }

#[derive(Clone)]
pub enum UserPanelEvent {
    OpenChat(String),
    AddRemoveFriend(u32)
}
