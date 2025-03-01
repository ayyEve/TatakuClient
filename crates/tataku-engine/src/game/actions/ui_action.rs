use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(Debug)]
pub struct UiAction {
    pub node: NodeId,
    pub action: UiActionType,
}
impl UiAction {
    pub fn new(node: NodeId, action: impl Into<UiActionType>) -> Self {
        Self {
            node,
            action: action.into(),
        }
    }
}

#[derive(Debug)]
pub enum UiActionType {
    Refresh,
    MarkDirty,
    UpdateStyle(Box<Style>),

    /// only update the display of a node. 
    /// 
    /// this should hopefully be cheaper than updating an entire style
    UpdateDisplay(ui::Display),

    /// rebuild the contexts for this node and its children
    ContextChanged,

    /// run a dialog action
    DialogAction(DialogAction),
}

impl From<UiAction> for TatakuAction {
    fn from(value: UiAction) -> Self {
        Self::Ui(value)
    }
}
impl From<DialogAction> for UiActionType {
    fn from(value: DialogAction) -> Self {
        Self::DialogAction(value)
    }
}