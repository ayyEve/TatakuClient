
use crate::prelude::*;

#[derive(Default)]
pub struct LobbyListComponent {
    actions: ActionQueue,
}

#[async_trait]
impl Widgetable for LobbyListComponent {
    async fn handle_message(&mut self, message: &Message, _values: &mut dyn Reflect) -> Vec<TatakuAction> { 
        let Some(tag) = message.tag.clone().as_string() else { return self.actions.take() };

        // might add more later idk
        #[allow(clippy::single_match)]
        match &*tag {
            "lobby.join" => {
                if let Some(id) = message.message_type.clone().as_number2() {
                    // TODO: show a dialog instead
                    self.actions.push(MultiplayerAction::JoinLobby { lobby_id: id as u32, password: String::new() });
                } else {
                    warn!("lobby id not number: {:?}", message.message_type)
                }
            }
            _ => {}
        }


        self.actions.take()
    }
}