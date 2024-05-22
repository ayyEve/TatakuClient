use crate::prelude::*;

pub struct CreateLobbyDialog {
    actions: ActionQueue,
    num: usize, 

    // scrollable: ScrollableArea,
    should_close: bool,


    name_text: String,
    password_text: String,
    is_private: bool, 
}
impl CreateLobbyDialog {
    pub fn new() -> Self {
        // const WIDTH:f32 = 500.0; 
        // let mut scrollable = ScrollableArea::new(Vector2::ZERO, Vector2::ZERO, ListMode::VerticalList);

        // // name
        // scrollable.add_item(Box::new(TextInput::new(Vector2::ZERO, Vector2::new(WIDTH, 50.0), "Lobby Name", "", Font::Main).with_tag("name")));

        // // password
        // scrollable.add_item(Box::new(TextInput::new(Vector2::ZERO, Vector2::new(WIDTH, 50.0), "Password", "", Font::Main).with_tag("password")));

        // // private
        // scrollable.add_item(Box::new(Checkbox::new(Vector2::ZERO, Vector2::new(WIDTH, 50.0), "Private", false, Font::Main).with_tag("private")));

        // // done and close buttons 
        // {
        //     let mut button_scrollable = ScrollableArea::new(Vector2::ZERO, Vector2::new(WIDTH, 50.0), ListMode::Grid(GridSettings::new(Vector2::ZERO, HorizontalAlign::Center)));
        //     button_scrollable.add_item(Box::new(MenuButton::new(Vector2::ZERO, Vector2::new(100.0, 50.0), "Done", Font::Main).with_tag("done")));
        //     button_scrollable.add_item(Box::new(MenuButton::new(Vector2::ZERO, Vector2::new(100.0, 50.0), "Close", Font::Main).with_tag("close")));
        //     scrollable.add_item(Box::new(button_scrollable));
        // }
        // scrollable.set_size(Vector2::new(WIDTH, scrollable.get_elements_height()));

        Self {
            actions: ActionQueue::new(),
            num: 0,
            // scrollable,
            should_close: false,

            name_text: String::new(),
            password_text: String::new(),
            is_private: false,
        }
    }

}

#[async_trait]
impl Dialog for CreateLobbyDialog {
    fn name(&self) -> &'static str { "create_lobby_dialog" }
    fn title(&self) -> &'static str { "Create a Lobby" }
    fn get_num(&self) -> usize { self.num }
    fn set_num(&mut self, num: usize) { self.num = num }

    fn should_close(&self) -> bool { self.should_close }
    // fn get_bounds(&self) -> Bounds { Bounds::new(Vector2::ZERO, self.scrollable.size()) }
    async fn force_close(&mut self) { self.should_close = true; }


    async fn handle_message(&mut self, message: Message, _values: &mut ValueCollection) {
        let Some(tag) = message.tag.as_string() else { return }; 

        match &*tag {
            "lobby_name" => {
                let Some(text) = message.message_type.as_text() else { return };
                self.name_text = text;
            }
            "lobby_password" => {
                let Some(text) = message.message_type.as_text() else { return };
                self.password_text = text;
            }
            "lobby_private" => {
                let Some(val) = message.message_type.as_toggle() else { return };
                self.is_private = val;
            }

            "done" => {
                let name = self.name_text.clone();
                let password = self.password_text.clone();
                let private = self.is_private;
                let players = 16;
                
                self.actions.push(MultiplayerAction::CreateLobby { name, password, private, players });
                // tokio::spawn(async move {
                //     OnlineManager::create_lobby(name, password, private, players).await
                // });
                self.should_close = true;
            }

            "close" => self.should_close = true,
            

            _ => {}
        }

    }

    
    async fn update(&mut self, _values: &mut ValueCollection) -> Vec<TatakuAction> { 
        self.actions.take()
    }


    fn view(&self, _values: &mut ValueCollection) -> IcedElement {
        use iced_elements::*;
        let owner = MessageOwner::new_dialog(self);
        col!(
            Text::new("Create Lobby: "),
            Text::new("    "),
            
            TextInput::new("Lobby Name", &self.name_text).on_input(move|t|Message::new(owner, "lobby_name", MessageType::Text(t))),
            TextInput::new("Lobby Password", &self.password_text).on_input(move|t|Message::new(owner, "lobby_password", MessageType::Text(t))),
            Checkbox::new("Private", self.is_private, move|v|Message::new(owner, "lobby_private", MessageType::Toggle(v))),

            row!(
                Button::new(Text::new("Done")).on_press(Message::new_dialog(self, "done", MessageType::Click)),
                Button::new(Text::new("Close")).on_press(Message::new_dialog(self, "close", MessageType::Click))
                ;
                width = Fill
            );

        )
    }
}
