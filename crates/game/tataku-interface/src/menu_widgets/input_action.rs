use crate::prelude::*;
use common::reflect::*;
use tataku::TatakuValue;
use ui::{
    tree::*,
    message::*,
};

type MessageCallback<T> = Box<dyn Fn(&T) -> Option<Message> + Send + Sync>;
type ActionCallback<T> = Box<dyn Fn(&T) -> Option<actions::Action> + Send + Sync>;
type ReflectCallback<T> = Box<dyn Fn(&T, &mut dyn Reflect) + Send + Sync>;

pub enum InputAction<T> {
    MessageCallback(MessageCallback<T>),
    ActionCallback(ActionCallback<T>),
    ReflectCallback(ReflectCallback<T>),
    Custom(Vec<BuildableAction>)
}
impl<T: Clone + Reflect> InputAction<T> {
    pub fn run(
        &self,
        value: &T,
        node: &NodeId,
        messages: &mut Vec<Message>,
        actions: &mut actions::ActionQueue,
        values: &mut dyn Reflect,
    ) {
        match self {
            Self::MessageCallback(callback)
                => if let Some(message) = callback(value) {
                    messages.push(message);
                },
            Self::ActionCallback(callback)
                => if let Some(action) = callback(value) {
                    actions.push(action);
                },
            Self::Custom(a) => {
                let passed_in = TatakuValue::from_reflection(
                    Box::new(value.clone())
                ).ok();
                let passed_in = passed_in.as_ref();

                // todo: error on failed
                let a = a.iter()
                    .cloned()
                    .filter_map(|action| action.resolve(node, values, passed_in));

                actions.extend(a);
            }
            Self::ReflectCallback(callback)
                => callback(value, values),
        }

    }
}

impl<T> From<BuildableAction> for InputAction<T> {
    fn from(value: BuildableAction) -> Self {
        vec![value].into()
    }
}
impl<T> From<Vec<BuildableAction>> for InputAction<T> {
    fn from(mut actions: Vec<BuildableAction>) -> Self {
        for action in actions.iter_mut() {
            action.build();
        }

        Self::Custom(actions)
    }
}
impl<T, A: Fn(&T) -> Option<Message> + Send + Sync + 'static> From<A> for InputAction<T> {
    fn from(value: A) -> Self {
        Self::MessageCallback(Box::new(value))
    }
}
