use crate::*;
use tataku::Vector2;

/// NOTE: which dialog is actioned is determined by the node id that sent the message
#[derive(Copy, Clone, Debug)]
pub enum DialogAction {
    /// The dialog is requesting to be closed
    Close,

    /// The dialog should be moved to the provided position
    MoveDialog(Vector2),
    
    /// The dialog should be resized to the provided size
    ResizeDialog(Vector2),

    /// Bring the dialog to the front of the list
    BringToFront,
}
