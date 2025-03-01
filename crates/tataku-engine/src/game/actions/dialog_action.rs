use crate::prelude::*;

/// NOTE: which dialog is actioned is determined by the node id that sent the message
#[derive(Copy, Clone, Debug)]
pub enum DialogAction {
    Close,
    MoveDialog(Vector2),
    ResizeDialog(Vector2),
}
