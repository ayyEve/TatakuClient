use crate::prelude::*;

/// helper struct for menu actions
#[derive(Default, Debug)]
pub struct ActionQueue(Vec<TatakuAction>);
impl ActionQueue {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn take(&mut self) -> Vec<TatakuAction> {
        self.0.take()
    }

    pub fn push(&mut self, action: impl Into<TatakuAction>) {
        self.0.push(action.into())
    }
    pub fn extend(&mut self, actions: Vec<TatakuAction>) {
        self.0.extend(actions)
    }
}
