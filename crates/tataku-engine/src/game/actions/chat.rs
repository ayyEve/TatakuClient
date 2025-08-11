use crate::prelude::*;

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
impl From<ChatAction> for TatakuAction {
    fn from(value: ChatAction) -> Self {
        TatakuAction::Online(OnlineAction::ChatAction(value))
    }
}
