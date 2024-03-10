use crate::prelude::*;
const BUTTON_SIZE:Vector2 = Vector2::new(300.0, 50.0);

pub struct LobbyPlayerDialog {
    num: usize,

    user_id: u32,
    should_close: bool,
    is_self: bool,
    we_are_host: bool,
}
impl LobbyPlayerDialog {
    pub fn new(user_id: u32, is_self: bool, we_are_host: bool) -> Self {
        Self {
            num: 0,
            user_id,
            should_close: false,

            is_self,
            we_are_host,
        }
    }
}

#[async_trait]
impl Dialog for LobbyPlayerDialog {
    fn name(&self) -> &'static str { "lobby_player_dialog" }
    fn get_num(&self) -> usize { self.num }
    fn set_num(&mut self, num: usize) { self.num = num }
    fn should_close(&self) -> bool { self.should_close }
    async fn force_close(&mut self) { self.should_close = true; }


    async fn handle_message(&mut self, message: Message, values: &mut ShuntingYardValues) {
        let Some(tag) = message.tag.as_string() else { return }; 

        match &*tag {
            "close" => self.should_close = true,
            "make_host" => {
                tokio::spawn(OnlineManager::lobby_change_host(self.user_id));
                self.should_close = true;
            }
            "kick" => {
                tokio::spawn(OnlineManager::lobby_kick_user(self.user_id));
                self.should_close = true;
            }

            _ => {}
        }
    }
    
    fn view(&self) -> IcedElement {
        use iced_elements::*;

        col!(
            // make host
            (self.we_are_host && !self.is_self).then(||Button::new(Text::new("Transfer Host")).on_press(Message::new_dialog(self, "make_host", MessageType::Click)).into_element())
                .unwrap_or_else(||EmptyElement.into_element()),
            // kick
            (self.we_are_host && !self.is_self).then(||Button::new(Text::new("Kick")).on_press(Message::new_dialog(self, "kick", MessageType::Click)).into_element())
                .unwrap_or_else(||EmptyElement.into_element()),
            // close
            Button::new(Text::new("Close")).on_press(Message::new_dialog(self, "close", MessageType::Click));
        )

    }
}