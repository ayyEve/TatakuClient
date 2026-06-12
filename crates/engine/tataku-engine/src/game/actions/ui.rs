use crate::*;
use ui::tree::NodeId;

#[derive(Debug)]
pub struct UiAction {
    pub node: NodeId,
    pub source: ui::MessageSource,
    pub action: UiActionType,
}
impl UiAction {
    pub fn new(
        node: NodeId, 
        source: ui::MessageSource,
        action: impl Into<UiActionType>,
    ) -> Self {
        Self {
            node,
            source, 
            action: action.into(),
        }
    }
}

#[derive(Debug2)]
pub enum UiActionType {
    // Refresh,
    // MarkDirty,
    // UpdateStyleWith(#[debug(skip)] Arc<dyn Fn(&mut ui::style::CssStyle) + Send + Sync>),

    // /// Only update the display of a node. 
    // OverrideDisplay(Option<ui::style::DisplayType>),

    /// Rebuild the contexts for this node and its children
    ContextChanged,

    /// Run a dialog action
    DialogAction(actions::dialog::DialogAction),
}

impl From<UiAction> for actions::Action {
    fn from(value: UiAction) -> Self {
        Self::Ui(value)
    }
}
impl From<actions::dialog::DialogAction> for UiActionType {
    fn from(value: actions::dialog::DialogAction) -> Self {
        Self::DialogAction(value)
    }
}
