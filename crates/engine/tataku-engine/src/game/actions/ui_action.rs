use crate::prelude::*;
use tataku_ui::prelude::*;

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug2)]
pub enum UiActionType {
    Refresh,
    MarkDirty,
    // UpdateStyle(Box<CssStyle>),
    UpdateStyleWith(#[debug(skip)] Arc<dyn Fn(&mut CssStyle) + Send + Sync>),

    /// Only update the display of a node. 
    OverrideDisplay(Option<DisplayType>),

    /// Rebuild the contexts for this node and its children
    ContextChanged,

    /// Run a dialog action
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
