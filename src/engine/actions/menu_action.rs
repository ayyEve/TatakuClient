use crate::prelude::*;

#[derive(Debug)]
pub enum MenuMenuAction {
    // /// Set the current menu
    // SetMenu(Box<dyn AsyncMenu>),

    /// Set the menu to a custom menu with the provided identifier
    SetMenu(String),

    /// Go to the previous menu
    /// 
    /// This is mainly a helper fn for spec and multi
    PreviousMenu(&'static str),

    // /// Add a dialog
    // /// 
    // /// dialog, allow_duplicates
    // AddDialog(Box<dyn Dialog>, bool),

    /// Set the menu to a custom menu with the provided identifier
    AddDialogCustom(String, bool),
}

impl From<MenuMenuAction> for TatakuAction {
    fn from(value: MenuMenuAction) -> Self { Self::Menu(value) }
}