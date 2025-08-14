use crate::prelude::*;
use tataku_ui::prelude::*;


#[derive(Debug2)]
pub enum MenuAction {
    /// Set the menu to the provided menu identifier
    SetMenu {
        id: CowStr,
        input: Box<BuildableInputArguments>,
    },

    /// Go to the previous menu
    /// 
    /// NOTE these are predefined previous menus, not built on a stack
    /// TODO: should we make it a stack?
    PreviousMenu(CowStr),

    /// Add a custom dialog
    AddDialog {
        id: CowStr,
        options: Box<DialogCreateOptions>,
        input: Box<BuildableInputArguments>,
    },

    #[debug(skip)]
    AddDialogRaw {
        dialog: Box<dyn Widget<TatakuAction>>,
        options: Box<DialogCreateOptions>,
    },
}
impl Clone for MenuAction {
    fn clone(&self) -> Self {
        match self {
            Self::AddDialogRaw { .. } => panic!("Trying to clone AddDialogRaw!"),
            Self::SetMenu { 
                id, 
                input 
            } => Self::SetMenu { id: id.clone(), input: input.clone() },

            MenuAction::PreviousMenu(c) 
                => Self::PreviousMenu(c.clone()),

            MenuAction::AddDialog { 
                id, 
                options, 
                input 
            } => Self::AddDialog { 
                id: id.clone(), 
                options: options.clone(), 
                input: input.clone()
            },
        }
    }
}

impl MenuAction {
    pub fn set_menu(menu: impl Into<CowStr>) -> Self {
        Self::SetMenu {
            id: menu.into(),
            input: Box::new(BuildableInputArguments::default())
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

#[derive(Clone, Debug)]
#[derive(ChainableInitializer)]
#[derive(Deserialize)]
pub struct DialogCreateOptions {
    #[serde(rename = "@allow_multiple")]
    #[chain] pub allow_multiple: bool,
    #[serde(rename = "@resizable")]
    #[chain] pub resizable: bool,
    #[serde(rename = "@draggable")]
    #[chain] pub draggable: bool,

    #[serde(rename = "@title")]
    #[chain] pub title: CowStr,
    
    #[serde(skip)]
    pub location: DialogLocation,
    
    #[serde(skip)]
    #[chain] pub background: bool,
}
impl DialogCreateOptions {
    pub fn merge(
        incoming: Self, 
        dialog_defaults: Self
    ) -> Self {
        Self {
            allow_multiple: incoming.allow_multiple && dialog_defaults.allow_multiple,
            resizable: incoming.resizable && dialog_defaults.resizable,
            draggable: incoming.draggable && dialog_defaults.draggable,
            location: incoming.location,
            title: if incoming.title.is_empty() { 
                dialog_defaults.title 
            } else { 
                incoming.title 
            },

            background: incoming.background,
        }
    }
}
impl Default for DialogCreateOptions {
    fn default() -> Self {
        Self {
            allow_multiple: false,
            resizable: false,
            draggable: false,
            title: Cow::Borrowed(""),
            location: DialogLocation::Auto,
            background: true,
        }
    }
}


#[derive(Clone, Debug, Default)]
pub enum DialogLocation {
    /// automatically determine the location
    #[default]
    Auto,

    /// fullscreen
    Fullscreen,

    /// near the cursor
    Cursor,
    /// centered on the screen
    Center,

    /// at a specific position
    Position(Vector2),
}