use crate::prelude::*;

#[derive(Clone, Debug, ChainableInitializer)]
pub struct TatakuVariable {
    /// The actual underlying value
    pub value: TatakuValue,

    /// The display name of this value
    /// should really only be used for strings, primarily for enums
    pub display: Option<Cow<'static, str>>,

    /// Should this variable persist through game state changes?
    #[chain]
    pub persist: bool,

    /// Who can write to this variable?
    #[chain]
    pub access: TatakuVariableAccess,
}
impl TatakuVariable {
    pub fn new(value: impl Into<TatakuValue>) -> Self {
        Self {
            value: value.into(),
            display: None,
            persist: true,
            access: TatakuVariableAccess::ReadOnly,
        }
    }

    /// shorthand for display:none, persist: true, access:GameOnly
    pub fn new_game(value: impl Into<TatakuValue>) -> Self {
        Self {
            value: value.into(),
            display: None,
            persist: true,
            access: TatakuVariableAccess::GameOnly,
        }
    }

    /// shorthand for display:none, persist: true, access:Any
    pub fn new_any(value: impl Into<TatakuValue>) -> Self {
        Self {
            value: value.into(),
            display: None,
            persist: true,
            access: TatakuVariableAccess::Any,
        }
    }


    pub fn display(mut self, display: impl Into<Cow<'static, str>>) -> Self {
        self.display = Some(display.into());
        self
    }
    
    pub fn get_display(&self) -> String {
        match &self.display {
            Some(s) => s.to_string(),
            None => self.value.as_string(),
        }
    }
}

impl AsRef<TatakuValue> for TatakuVariable {
    fn as_ref(&self) -> &TatakuValue {
        &self.value
    }
}

impl Deref for TatakuVariable {
    type Target = TatakuValue;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl PartialEq for TatakuVariable {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TatakuVariableAccess {
    /// Can only be read, never written to
    ReadOnly,

    /// Can only be written to by the game
    GameOnly,

    /// Can be written to from anywhere
    Any,
}
impl TatakuVariableAccess {
    pub fn check_access(&self, write: &TatakuVariableWriteSource) -> bool {
        use TatakuVariableWriteSource as Source;
        match (self, write) {
            (Self::Any, _) => true,
            (Self::ReadOnly, _) => false,
            
            (Self::GameOnly, Source::Game) => true,
            (Self::GameOnly, Source::Menu) => false,
        }
    }
}


#[derive(Copy, Clone, Debug)]
pub enum TatakuVariableWriteSource {
    /// Write from Game
    Game,

    /// Write from Custom Menu
    Menu,

    // /// Write from plugin 
    // CustomUI,
}
