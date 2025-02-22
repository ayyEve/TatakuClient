use crate::prelude::*;

#[derive(Clone)]
pub struct OnlineUser {
    pub user_id: u32,
    pub username: String,

    pub action: Option<UserAction>,
    pub action_text: Option<String>,
    pub mode: Option<PlayMode>,

    pub game: String,
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
        }
    }
}
impl Default for OnlineUser {
    fn default() -> Self {
        Self { 
            user_id: Default::default(), 
            username: Default::default(), 
            action: Default::default(), 
            action_text: Default::default(),
            mode: Default::default(),
            game: String::new(),
        }
    }
}