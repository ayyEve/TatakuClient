use crate::prelude::*;

pub struct JoinLobbyDialog {
    num: usize,
    lobby_id: u32,
    // scrollable: ScrollableArea,
    should_close: bool,

    password: String,
}
impl JoinLobbyDialog {
    pub fn new(lobby_id: u32) -> Self {
        // const WIDTH:f32 = 500.0; 
        // let mut scrollable = ScrollableArea::new(Vector2::ZERO, Vector2::ZERO, ListMode::VerticalList);

        // // password
        // scrollable.add_item(Box::new(TextInput::new(Vector2::ZERO, Vector2::new(WIDTH, 50.0), "Password", "", Font::Main).with_tag("password")));

        // // done and close buttons 
        // {
        //     let mut button_scrollable = ScrollableArea::new(Vector2::ZERO, Vector2::new(WIDTH, 50.0), ListMode::Grid(GridSettings::new(Vector2::ZERO, HorizontalAlign::Center)));
        //     button_scrollable.add_item(Box::new(MenuButton::new(Vector2::ZERO, Vector2::new(100.0, 50.0), "Done", Font::Main).with_tag("done")));
        //     button_scrollable.add_item(Box::new(MenuButton::new(Vector2::ZERO, Vector2::new(100.0, 50.0), "Close", Font::Main).with_tag("close")));
        //     scrollable.add_item(Box::new(button_scrollable));
        // }
        // scrollable.set_size(Vector2::new(WIDTH, scrollable.get_elements_height()));

        Self {
            num: 0,
            lobby_id,
            // scrollable,
            password: String::new(),
            should_close: false,
        }
    }

}

#[async_trait]
impl Dialog for JoinLobbyDialog {
    fn name(&self) -> &'static str { "join_lobby_dialog" }
    fn title(&self) -> &'static str { "Join Lobby" }
    fn get_num(&self) -> usize { self.num }
    fn set_num(&mut self, num: usize) { self.num = num }
    fn should_close(&self) -> bool { self.should_close }
    async fn force_close(&mut self) { self.should_close = true; }


    async fn handle_message(&mut self, message: Message, _values: &mut ShuntingYardValues) {
        let Some(tag) = message.tag.as_string() else { return }; 

        match &*tag {
            "done" => {
                let id = self.lobby_id;
                let password = self.password.clone(); //get_value::<String>("password");
                tokio::spawn(async move { OnlineManager::join_lobby(id, password).await; });
                self.should_close = true;
            }

            "close" => self.should_close = true,

            _ => {}
        }
    }

    
    
    fn view(&self) -> IcedElement {
        use iced_elements::*;
        
        let owner = MessageOwner::new_dialog(self);
        col!(
            Text::new("Enter Password:"),

            TextInput::new("Password:", &self.password).on_input(move|t|Message::new(owner, "password", MessageType::Text(t))),

            row!(
                Button::new(Text::new("Join")).on_press(Message::new_dialog(self, "done", MessageType::Click)),
                Button::new(Text::new("Cancel")).on_press(Message::new_dialog(self, "close", MessageType::Click));
                width = Fill
            );

        )
    }
}
