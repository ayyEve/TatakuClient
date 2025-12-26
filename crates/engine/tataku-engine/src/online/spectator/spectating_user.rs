use crate::*;
use common::reflect::*;

#[derive(Reflect)]
#[derive(Clone, Debug)]
pub struct SpectatingUser {
    pub user_id: u32,
    pub username: String,
}
impl SpectatingUser {
    pub fn new(user_id: u32, username: impl Into<String>) -> Self {
        Self {
            user_id,
            username: username.into()
        }
    }
}
