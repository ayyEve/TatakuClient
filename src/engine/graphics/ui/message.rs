use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Message {
    pub owner: MessageOwner,
    // pub menu_name: String,
    // pub dialog_num: usize,

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

    pub fn new_menu_raw(name: &'static str, item_tag: impl Into<MessageTag>, message: MessageType) -> Self {
        Self::new(MessageOwner::Menu(name), item_tag, message)
    }

    pub fn new_menu(menu: &impl AsyncMenu, item_tag: impl Into<MessageTag>, message: MessageType) -> Self {
        Self::new(MessageOwner::new_menu(menu), item_tag, message)
    }

    pub fn new_dialog(dialog: &impl Dialog, item_tag: impl Into<MessageTag>, message: MessageType) -> Self {
        Self {
            owner: MessageOwner::new_dialog(dialog),
            tag: item_tag.into(),
            message_type: message,
        }
    }

    /// helper to make a click message for the given menu and item tag
    pub fn menu_click(menu: &impl AsyncMenu, item_tag: impl Into<MessageTag>) -> Self {
        Self::new_menu(menu, item_tag, MessageType::Click)
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


#[derive(Debug, Clone)]
pub enum MessageType {
    Click,
    Text(String),
    Key(Key),
    Number(usize),
    Float(f32),
    Toggle(bool),
    Dropdown(String),

    Custom(Arc<dyn std::any::Any + Send + Sync>)
}
impl MessageType {
    pub fn as_text(self) -> Option<String> {
        let Self::Text(t) = self else { return None };
        Some(t)
    }
    pub fn as_key(self) -> Option<Key> {
        let Self::Key(k) = self else { return None };
        Some(k)
    }
    pub fn as_number(self) -> Option<usize> {
        let Self::Number(n) = self else { return None };
        Some(n)
    }
    pub fn as_float(self) -> Option<f32> {
        let Self::Float(n) = self else { return None };
        Some(n)
    }
    pub fn as_toggle(self) -> Option<bool> {
        let Self::Toggle(b) = self else { return None };
        Some(b)
    }
    pub fn as_dropdown(self) -> Option<String> {
        let Self::Dropdown(s) = self else { return None };
        Some(s)
    }

    pub fn downcast<T:Send+Sync+'static>(self) -> Arc<T> {
        let Self::Custom(t) = self else { panic!("nope") };
        t.downcast().unwrap()
    }
    
}


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MessageOwner {
    Menu(&'static str),
    Dialog(&'static str, usize),
}
impl MessageOwner {
    pub fn new_menu(menu: &impl AsyncMenu) -> Self {
        Self::Menu(menu.get_name())
    }
    pub fn new_dialog(dialog: &impl Dialog) -> Self {
        Self::Dialog(dialog.name(), dialog.get_num())
    }

    pub fn check_menu(&self, menu: &Box<dyn AsyncMenu>) -> bool {
        let Self::Menu(name) = self else { return false };
        name == &menu.get_name()
    }

    pub fn check_dialog(&self, dialog: &Box<dyn Dialog>) -> bool {
        let Self::Dialog(name, number) = self else { return false };
        name == &dialog.name() && number == &dialog.get_num()
    }
}
