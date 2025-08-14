use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Message {
    pub owner: MessageOwner,
    pub tag: Arc<String>,
    pub value: MessageValue,
}
impl Message {
    pub fn new(
        owner: MessageOwner, 
        item_tag: impl Into<String>, 
        message: MessageValue
    ) -> Self {
        Self {
            owner,
            tag: Arc::new(item_tag.into()),
            value: message,
        }
    }

    /// helper to make a click message for the given menu and item tag
    pub fn click(owner: MessageOwner, item_tag: impl Into<String>) -> Self {
        Self::new(owner, item_tag, MessageValue::Click)
    }

    pub fn with_value(mut self, message: MessageValue) -> Self {
        self.value = message;
        self
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
pub enum MessageValue {
    Click,
    Text(String),
    Number(usize),
    
    Value(TatakuValue),

    Custom(Arc<dyn std::any::Any + Send + Sync>),
}
#[allow(unused)]
impl MessageValue {
    message_type!(as_text, as_text_ref, Text, String, String);
    message_type!(as_number, as_number_ref, Number, usize);
    message_type!(as_value, as_value_ref, Value, TatakuValue);

    pub fn as_number2(&self) -> Option<usize> {
        if let Some(num) = self.as_number_ref() { return Some(*num) }
        match self.as_value_ref()? {
            TatakuValue::U32(n) => Some(*n as usize),
            TatakuValue::U64(n) => Some(*n as usize),

            _ => None
        }
    }

    pub fn downcast<T:Send+Sync+'static>(&self) -> Arc<T> {
        let Self::Custom(t) = self else { panic!("nope") };
        t.clone().downcast().unwrap()
    }
    pub fn try_downcast<T:Send+Sync+'static>(self) -> Option<Arc<T>> {
        let Self::Custom(t) = self else { return None };
        t.downcast().ok()
    }

    pub fn try_downcast_ref<T:Send+Sync+'static>(&self) -> Option<&T> {
        let Self::Custom(t) = self else { return None };
        t.downcast_ref()
    }
}



#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum MessageOwner {
    #[default] Menu,
    Dialog(usize),
}
impl MessageOwner {
    pub fn is_menu(&self) -> bool {
        matches!(self, Self::Menu)
    }

    pub fn is_eq(self, other: Self) -> bool {
        match (self, other) {
            (Self::Menu, Self::Menu) => true,
            (Self::Dialog(n), Self::Dialog(n2)) => n == n2,
            _ => false,
        }
    }

    /// Click message helper
    pub fn click(self, tag: impl Into<String>) -> Message {
        Message::click(self, tag)
    }
}
