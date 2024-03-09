use crate::prelude::*;

/// helper struct for menu actions
pub struct ActionQueue(Vec<MenuAction>);
impl ActionQueue {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn take(&mut self) -> Vec<MenuAction> {
        self.0.take()
    }

    pub fn push(&mut self, action: impl Into<MenuAction>) {
        self.0.push(action.into())
    }
}
