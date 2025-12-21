use crate::*;

pub type MessageValue = Box<dyn std::any::Any + Send + Sync>;

#[derive(Debug)]
pub struct Message {
    pub source: MessageSource,
    pub tag: ArcStr,
    pub target: Option<MessageTarget>,
    pub value: MessageValue,
}
impl Message {
    pub fn new(
        source: MessageSource,
        tag: impl Into<ArcStr>,
        target: Option<MessageTarget>,
        value: MessageValue,
    ) -> Self {
        Self {
            source,
            tag: tag.into(),
            target,
            value,
        }
    }

    pub fn with_value(mut self, message: MessageValue) -> Self {
        self.value = message;
        self
    }
}

#[derive(Clone, Debug)]
pub enum MessageTarget {
    /// A specific node id
    Node(NodeId),

    /// An element with the provided id
    ElementId(CowStr),

    /// An element with the provided class
    ElementClass(CowStr),
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum MessageSource {
    #[default] Menu,
    Dialog(usize),
}
impl MessageSource {
    pub fn is_menu(&self) -> bool {
        matches!(self, Self::Menu)
    }
}
