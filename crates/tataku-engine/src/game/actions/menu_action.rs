use crate::prelude::*;

#[derive(Debug)]
pub enum MenuAction {
    // /// Set the current menu
    // SetMenu(Box<dyn AsyncMenu>),

    /// Set the menu to a custom menu with the provided identifier
    SetMenu(Cow<'static, str>),

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
impl MenuAction {
    pub fn set_menu(menu: impl Into<Cow<'static, str>>) -> Self {
        Self::SetMenu(menu.into())
    }
}

impl From<MenuAction> for TatakuAction {
    fn from(value: MenuAction) -> Self { Self::Menu(value) }
}