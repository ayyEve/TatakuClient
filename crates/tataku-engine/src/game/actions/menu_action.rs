use crate::prelude::*;

#[derive(Debug)]
pub enum MenuAction {
    /// Set the menu to the provided menu identifier
    SetMenu(Cow<'static, str>),

    /// Go to the previous menu
    /// 
    /// NOTE these are predefined previous menus, not built on a stack
    /// TODO: should we make it a stack?
    PreviousMenu(Cow<'static, str>),

    /// Add a custom dialog with the provided identifier, and if multiple of the same dialog are allowed
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
