use crate::prelude::*;
use common::reflect::*;
use tataku::TatakuValue;
use ui::{
    tree::*,
    message::*,
};

type Callback = Box<dyn Fn() -> Option<Message> + Send + Sync>;

#[derive(From)]
pub enum ContextMenuAction {
    Callback(Callback),
    Buildable(BuildableAction),
}
impl ContextMenuAction {
    pub fn build(&mut self) {
        if let Self::Buildable(b) = self {
            b.build();
        }
    }

    pub fn run(
        &self,
        node: &NodeId,
        passed_in: Option<&TatakuValue>,
        values: &mut dyn Reflect,
        actions: &mut actions::ActionQueue,
        messages: &mut Vec<Message>,
    ) {
        match self {
            Self::Callback(cb) => {
                if let Some(m) = cb() {
                    messages.push(m);
                }
            }
            Self::Buildable(b) => {
                if let Some(action) = b.clone().resolve(
                    node, 
                    values, 
                    passed_in,
                ) {
                    actions.push(action);
                }
            },
        }
    }
}
