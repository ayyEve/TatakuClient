use crate::*;

#[derive(Clone, Debug)]
pub enum ChatAction {
    SendMessage {
        channel: String,
        message: String,
    },

    OpenChannel {
        channel: String,
        password: Option<String>,
    },

    CloseChannel {
        channel: String,
    },
}
impl From<ChatAction> for actions::Action {
    fn from(value: ChatAction) -> Self {
        actions::Action::Online(actions::online::OnlineAction::ChatAction(value))
    }
}
