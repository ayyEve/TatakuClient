use crate::prelude::*;

/// helper for managing lists of spectators
#[derive(Default, Clone)]
pub struct SpectatorList {
    pub list: Vec<SpectatingUser>,
    pub updated: bool,
}
impl SpectatorList {
    pub fn add(&mut self, user: SpectatingUser) {
        self.list.push(user);
        self.updated = true;
    }
    pub fn remove(&mut self, user_id: u32) {
        let Some((index, _)) = self.list.iter().enumerate().find(|(_,u)|u.user_id == user_id) else { return };
        self.list.remove(index);
        self.updated = true;
    }
}