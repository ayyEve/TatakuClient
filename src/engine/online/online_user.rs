use crate::prelude::*;

#[derive(Clone, Default)]
pub struct OnlineUser {
    pub user_id: u32,
    pub username: String,

    pub action: Option<UserAction>,
    pub action_text: Option<String>,
    pub mode: Option<PlayMode>,

    pub game: String,

    pub friend: bool,
}
impl OnlineUser {
    pub fn new(user_id:u32, username:String) -> Self {
        Self {
            user_id,
            username,
            action:None,
            action_text: None,
            mode: None,
            game: String::new(),

            friend: false,
        }
    }
}
