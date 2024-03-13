use crate::prelude::*;

/// helper struct for menu actions
#[derive(Default)]
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
    pub fn extend(&mut self, actions: Vec<MenuAction>) {
        self.0.extend(actions.into_iter())
    }
}
