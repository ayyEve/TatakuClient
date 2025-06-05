use crate::prelude::*;

type MessageCallback<T> = Box<dyn Fn(&T) -> Message + Send + Sync>;
type ActionCallback<T> = Box<dyn Fn(&T) -> TatakuAction + Send + Sync>;
type ReflectCallback<T> = Box<dyn Fn(&T, &mut dyn Reflect) + Send + Sync>;

pub enum InputAction<T> {
    Message(Option<Message>),
    MessageCallback(MessageCallback<T>),
    ActionCallback(ActionCallback<T>),
    ReflectCallback(ReflectCallback<T>),
    Custom(BuildableAction),
    Multi(Vec<Self>),
}
impl<T:Clone + Reflect> InputAction<T> {
    pub fn run(
        &self, 
        value: &T,
        node: NodeId,
        messages: &mut Vec<Message>,
        actions: &mut ActionQueue,
        values: &mut dyn Reflect,
    ) {
        match self {
            Self::Message(None) => {},
            Self::Message(Some(m)) => messages.push(m.clone()),
            Self::MessageCallback(callback) 
                => messages.push(callback(value)),
            Self::ActionCallback(callback) 
                => actions.push(callback(value)),
            Self::Custom(b) => {
                let passed_in = TatakuValue::from_reflection(
                    Box::new(value.clone())
                ).ok();

                if let Some(action) = b.clone().into_action(
                    node, 
                    values, 
                    passed_in.as_ref()
                ) {
                    actions.push(action);
                }
            }
            Self::ReflectCallback(callback)
                => callback(value, values),

            Self::Multi(list) => {
                for action in list {
                    action.run(value, node, messages, actions, values);
                }
            }
        }

    }
}

impl<T> Default for InputAction<T> {
    fn default() -> Self { Self::Message(None) }
}
impl<T, A: Into<InputAction<T>>> From<Option<A>> for InputAction<T> {
    fn from(value: Option<A>) -> Self {
        let Some(value) = value else { return Self::Message(None) };
        value.into()
    }
}
impl<T> From<Message> for InputAction<T> {
    fn from(value: Message) -> Self {
        Self::Message(Some(value))
    }
}
impl<T> From<BuildableAction> for InputAction<T> {
    fn from(mut value: BuildableAction) -> Self {
        if let BuildableAction::Conditional { 
            cond, 
            .. 
        } = &mut value {
            cond.build();
        }

        Self::Custom(value)
    }
}
impl<T, A: Fn(&T) -> Message + Send + Sync + 'static> From<A> for InputAction<T> {
    fn from(value: A) -> Self {
        Self::MessageCallback(Box::new(value))
    }
}
