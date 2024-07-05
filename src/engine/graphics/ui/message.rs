use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Message {
    pub owner: MessageOwner,
    pub tag: MessageTag,
    pub message_type: MessageType,
}
impl Message {
    pub fn new(owner: MessageOwner, item_tag: impl Into<MessageTag>, message: MessageType) -> Self {
        Self {
            owner,
            tag: item_tag.into(),
            message_type: message,
        }
    }

    pub fn new_dialog(dialog: &impl Dialog, item_tag: impl Into<MessageTag>, message: MessageType) -> Self {
        Self {
            owner: MessageOwner::new_dialog(dialog),
            tag: item_tag.into(),
            message_type: message,
        }
    }

    /// helper to make a click message for the given menu and item tag
    pub fn click(owner: MessageOwner, item_tag: impl Into<MessageTag>) -> Self {
        Self::new(owner, item_tag, MessageType::Click)
    }


    pub fn with_type(mut self, message: MessageType) -> Self {
        self.message_type = message;
        self
    }
}

#[derive(Clone, Debug)]
pub enum MessageTag {
    Number(usize),
    String(String),
    Beatmap(Arc<BeatmapMeta>),
    GameplayMod(GameplayMod)
}
impl MessageTag {
    pub fn as_string(self) -> Option<String> {
        match self {
            Self::String(s) => Some(s),
            _ => None
        }
    }

    pub fn as_number(self) -> Option<usize> {
        match self {
            Self::Number(n) => Some(n),
            _ => None
        }
    }
    
}

impl From<&str> for MessageTag {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}
impl From<String> for MessageTag {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}
impl From<&String> for MessageTag {
    fn from(value: &String) -> Self {
        Self::String(value.clone())
    }
}

impl From<usize> for MessageTag {
    fn from(value: usize) -> Self {
        Self::Number(value)
    }
}
impl From<Arc<BeatmapMeta>> for MessageTag {
    fn from(value: Arc<BeatmapMeta>) -> Self {
        Self::Beatmap(value)
    }
}

impl From<GameplayMod> for MessageTag {
    fn from(value: GameplayMod) -> Self {
        Self::GameplayMod(value)
    }
}


macro_rules! message_type {
    ($a1:ident, $a2:ident, $t:ident, $t2:ty) => {
        pub fn $a1(self) -> Option<$t2> {
            let Self::$t(v) = self else { return None };
            Some(v)
        }

        pub fn $a2(&self) -> Option<&$t2> {
            let Self::$t(v) = self else { return None };
            Some(v)
        }
    };
    
    ($a1:ident, $a2:ident, $t:ident, $t2:ty, $t3: ident) => {
        pub fn $a1(self) -> Option<$t2> {
            let Self::$t(v) = self else { 
                let Self::Value(TatakuValue::$t3(v)) = self else { return None };
                return Some(v);
            };
            Some(v)
        }

        pub fn $a2(&self) -> Option<&$t2> {
            let Self::$t(v) = self else { 
                let Self::Value(TatakuValue::$t3(v)) = self else { return None };
                return Some(v);
            };
            Some(v)
        }
    }
}

#[derive(Debug, Clone)]
pub enum MessageType {
    Click,
    Text(String),
    Key(Key),
    Number(usize),
    Float(f32),
    Toggle(bool),
    Dropdown(String),
    
    Value(TatakuValue),
    Multi(Vec<Message>),

    Custom(Arc<dyn std::any::Any + Send + Sync>),
    GameplayManagerId(Arc<u32>),
    CustomMenuAction(CustomMenuAction, Option<TatakuValue>),
}
#[allow(unused)]
impl MessageType {
    message_type!(as_text, as_text_ref, Text, String, String);
    message_type!(as_key, as_key_ref, Key, Key);
    message_type!(as_number, as_number_ref, Number, usize);
    message_type!(as_float, as_float_ref, Float, f32, F32);
    message_type!(as_toggle, as_toggle_ref, Toggle, bool, Bool);
    message_type!(as_dropdown, as_dropdown_ref, Dropdown, String, String);
    message_type!(as_value, as_value_ref, Value, TatakuValue);

    pub fn as_number2(&self) -> Option<usize> {
        if let Some(num) = self.as_number_ref() { return Some(*num) }

        let Some(val) = self.as_value_ref() else { return None };
        match val {
            TatakuValue::U32(n) => Some(*n as usize),
            TatakuValue::U64(n) => Some(*n as usize),

            _ => None
        }
    }

    pub fn downcast<T:Send+Sync+'static>(self) -> Arc<T> {
        let Self::Custom(t) = self else { panic!("nope") };
        t.downcast().unwrap()
    }
    pub fn try_downcast<T:Send+Sync+'static>(self) -> Option<Arc<T>> {
        let Self::Custom(t) = self else { return None };
        t.downcast().ok()
    }

    pub fn try_downcast_ref<T:Send+Sync+'static>(&self) -> Option<&Arc<T>> {
        let Self::Custom(t) = self else { return None };
        t.downcast_ref()
    }
}



#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum MessageOwner {
    #[default]
    Menu,
    Dialog(&'static str, usize),
}
impl MessageOwner {
    pub fn new_dialog(dialog: &impl Dialog) -> Self {
        Self::Dialog(dialog.name(), dialog.get_num())
    }

    pub fn is_menu(&self) -> bool {
        match self {
            Self::Menu => true,
            _ => false
        }
    }

    pub fn check_dialog(&self, dialog: &Box<dyn Dialog>) -> bool {
        let Self::Dialog(name, number) = self else { return false };
        name == &dialog.name() && number == &dialog.get_num()
    }

    /// Click message helper
    pub fn click(self, tag: impl Into<MessageTag>) -> Message {
        Message::click(self, tag)
    }
    /// Float message helper
    pub fn float(self, tag: impl Into<MessageTag>, val: f32) -> Message {
        Message::new(self, tag, MessageType::Float(val))
    }
}