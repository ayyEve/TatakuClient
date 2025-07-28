use crate::prelude::*;

type MessageCallback<T> = Box<dyn Fn(&T) -> Message + Send + Sync>;
type ActionCallback<T> = Box<dyn Fn(&T) -> TatakuAction + Send + Sync>;
type ReflectCallback<T> = Box<dyn Fn(&T, &mut dyn Reflect) + Send + Sync>;

pub enum InputAction<T> {
    Message(Option<Message>),
    MessageCallback(MessageCallback<T>),
    ActionCallback(ActionCallback<T>),
    ReflectCallback(ReflectCallback<T>),
    Custom {
        action: BuildableAction,
        built: bool,
    },
    Multi {
        list: Vec<Self>,
        built: bool,
    },
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
            Self::Custom { action, .. } => {
                let passed_in = TatakuValue::from_reflection(
                    Box::new(value.clone())
                ).ok();

                let mut action = action.clone();
                action.build(values);


                if let Some(action) = action.into_action(
                    node, 
                    values, 
                    passed_in.as_ref()
                ) {
                    actions.push(action);
                }
            }
            Self::ReflectCallback(callback)
                => callback(value, values),

            Self::Multi { list, .. } => {
                for action in list {
                    action.run(value, node, messages, actions, values);
                }
            }
        }

    }

    pub fn build(
        &mut self,
        values: &mut dyn Reflect,
    ) {
        match self {
            Self::Custom { 
                action, 
                built,
            } if !*built => {
                *built = true;
                action.build(values);
            }

            Self::Multi { 
                list, 
                built
            } if !*built => {
                *built = true;
                list
                    .iter_mut()
                    .for_each(|i| i.build(values));
            }

            _ => {}
        }
    }

    pub fn is_built(&self) -> bool {
        match self {
            Self::Multi { built, .. } => *built,
            Self::Custom { built, .. } => *built,
            _ => true
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

        Self::Custom {
            action: value,
            built: false,
        }
    }
}
impl<T, A: Fn(&T) -> Message + Send + Sync + 'static> From<A> for InputAction<T> {
    fn from(value: A) -> Self {
        Self::MessageCallback(Box::new(value))
    }
}

impl<T> From<Vec<InputAction<T>>> for InputAction<T> {
    fn from(value: Vec<InputAction<T>>) -> Self {
        Self::Multi { list: value, built: false }
    }
}
