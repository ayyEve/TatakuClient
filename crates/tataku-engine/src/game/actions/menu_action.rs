use crate::prelude::*;

#[derive(Debug)]
pub enum MenuAction {
    /// Set the menu to the provided menu identifier
    SetMenu {
        id: Cow<'static, str>,
        input: BuildableInputArguments,
    },

    /// Go to the previous menu
    /// 
    /// NOTE these are predefined previous menus, not built on a stack
    /// TODO: should we make it a stack?
    PreviousMenu(Cow<'static, str>),

    /// Add a custom dialog
    AddDialog {
        id: Cow<'static, str>,
        allow_duplicates: bool,
        input: BuildableInputArguments,
    },
}
impl MenuAction {
    pub fn set_menu(
        menu: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self::SetMenu {
            id: menu.into(),
            input: BuildableInputArguments::default()
        }
    }
}
impl From<MenuAction> for TatakuAction {
    fn from(value: MenuAction) -> Self { Self::Menu(value) }
}



#[derive(Clone, Debug, Default)]
pub struct BuildableInputArguments(pub HashMap<String, TatakuValue>);
impl BuildableInputArguments {
    pub fn insert(&mut self, key: impl ToString, value: impl Into<TatakuValue>) {
        self.0.insert(key.to_string(), value.into());
    }
    pub fn add(mut self, key: impl ToString, value: impl Into<TatakuValue>) -> Self {
        self.insert(key, value);
        self
    }
}
impl Deref for BuildableInputArguments {
    type Target = HashMap<String, TatakuValue>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for BuildableInputArguments {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}