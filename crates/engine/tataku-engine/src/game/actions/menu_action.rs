use crate::prelude::*;
use tataku_ui::prelude::*;


#[derive(Debug2)]
pub enum MenuAction {
    /// Set the menu to the provided menu identifier
    SetMenu {
        id: CowStr,
    },

    // /// Go to the previous menu
    // /// 
    // /// NOTE these are predefined previous menus, not built on a stack
    // /// TODO: should we make it a stack?
    // PreviousMenu(CowStr),

    /// Add a custom dialog
    AddDialog {
        id: CowStr,
        options: Box<DialogCreateOptions>,
    },

    #[debug(skip)]
    AddDialogRaw {
        dialog: Box<dyn Widget<TatakuAction>>,
        options: Box<DialogCreateOptions>,
    },
}
impl MenuAction {
    pub fn set_menu(menu: impl Into<CowStr>) -> Self {
        Self::SetMenu {
            id: menu.into(),
        }
    }
}
impl Clone for MenuAction {
    fn clone(&self) -> Self {
        match self {
            Self::AddDialogRaw { .. } => panic!("Trying to clone AddDialogRaw!"),
            Self::SetMenu { id } => Self::SetMenu { id: id.clone() },

            // MenuAction::PreviousMenu(c) 
            //     => Self::PreviousMenu(c.clone()),

            MenuAction::AddDialog { 
                id, 
                options, 
            } => Self::AddDialog { 
                id: id.clone(), 
                options: options.clone(), 
            },
        }
    }
}

impl From<MenuAction> for TatakuAction {
    fn from(value: MenuAction) -> Self { Self::Menu(value) }
}

#[derive(Deserialize)]
#[derive(ChainableInitializer)]
#[derive(Clone, Debug, Default2)]
pub struct DialogCreateOptions {
    #[serde(rename = "@allow_multiple")]
    #[chain] pub allow_multiple: bool,
    #[serde(rename = "@resizable")]
    #[chain] pub resizable: bool,
    #[serde(rename = "@draggable")]
    #[chain] pub draggable: bool,

    #[serde(rename = "@title")]
    #[default(Cow::Borrowed(""))]
    #[chain] pub title: CowStr,
    
    #[serde(skip)]
    pub location: DialogLocation,
    
    #[serde(skip)]
    #[default(true)]
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
