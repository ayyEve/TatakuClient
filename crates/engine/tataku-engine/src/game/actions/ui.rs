use crate::*;
use ui::tree::NodeId;
use ui::widget::UiOperation;

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
    UpdateStyleWith(#[debug(skip)] Arc<dyn Fn(&mut ui::style::CssStyle) + Send + Sync>),

    /// Only update the display of a node. 
    OverrideDisplay(Option<ui::style::DisplayType>),

    /// Rebuild the contexts for this node and its children
    ContextChanged,

    /// Run a dialog action
    DialogAction(actions::dialog::DialogAction),

    Operation(UiOperation),
}
impl From<UiOperation> for UiActionType {
    fn from(value: UiOperation) -> Self {
        Self::Operation(value)
    }
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
