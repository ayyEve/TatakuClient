use crate::prelude::*;


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[derive(Serialize, Deserialize)]
#[derive(Reflect)]
pub enum Key {
    /// The '1' key over the letters.
    Key1,
    /// The '2' key over the letters.
    Key2,
    /// The '3' key over the letters.
    Key3,
    /// The '4' key over the letters.
    Key4,
    /// The '5' key over the letters.
    Key5,
    /// The '6' key over the letters.
    Key6,
    /// The '7' key over the letters.
    Key7,
    /// The '8' key over the letters.
    Key8,
    /// The '9' key over the letters.
    Key9,
    /// The '0' key over the 'O' and 'P' keys.
    Key0,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    /// The Escape key, next to F1.
    Escape,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    /// Print Screen/SysRq.
    Snapshot,
    /// Scroll Lock.
    Scroll,
    /// Pause/Break key, next to Scroll lock.
    Pause,

    /// `Insert`, next to Backspace.
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,

    Left,
    Up,
    Right,
    Down,

    /// The Backspace key, right over Enter.
    Backspace,
    /// The Enter key.
    Enter,
    /// The space bar.
    Space,

    /// The "Compose" key on Linux.
    Compose,

    Caret,

    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadDivide,
    NumpadDecimal,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    NumpadMultiply,
    NumpadSubtract,

    Apostrophe,
    Apps,
    Asterisk,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Equals,
    Grave,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Mute,
    NextTrack,
    NoConvert,
    Period,
    Plus,
    Power,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Stop,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
}

#[cfg(feature="graphics")]
impl Key {
    pub fn from_input(input: &KeyInput) -> Option<Self> {
        use winit::keyboard::KeyLocation::*;

        if let winit::keyboard::Key::<winit::keyboard::SmolStr>::Character(txt) = &input.logical {
            return match (txt.as_str(), input.location) {
                ("1", Standard) => Some(Self::Key1),
                ("2", Standard) => Some(Self::Key2),
                ("3", Standard) => Some(Self::Key3),
                ("4", Standard) => Some(Self::Key4),
                ("5", Standard) => Some(Self::Key5),
                ("6", Standard) => Some(Self::Key6),
                ("7", Standard) => Some(Self::Key7),
                ("8", Standard) => Some(Self::Key8),
                ("9", Standard) => Some(Self::Key9),
                ("0", Standard) => Some(Self::Key0),
                ("-", Standard) => Some(Self::Minus),
                ("+", Standard) => Some(Self::Plus),
                ("=", Standard) => Some(Self::Equals),
                ("/", Standard) => Some(Self::Slash),
                ("\\", Standard) => Some(Self::Backslash),
                ("*", Standard) => Some(Self::Asterisk),
                (".", Standard) => Some(Self::Period),
                (",", Standard) => Some(Self::NumpadComma),

                ("1", Numpad) => Some(Self::Numpad1),
                ("2", Numpad) => Some(Self::Numpad2),
                ("3", Numpad) => Some(Self::Numpad3),
                ("4", Numpad) => Some(Self::Numpad4),
                ("5", Numpad) => Some(Self::Numpad5),
                ("6", Numpad) => Some(Self::Numpad6),
                ("7", Numpad) => Some(Self::Numpad7),
                ("8", Numpad) => Some(Self::Numpad8),
                ("9", Numpad) => Some(Self::Numpad9),
                ("0", Numpad) => Some(Self::Numpad0),
                ("-", Numpad) => Some(Self::NumpadSubtract),
                ("+", Numpad) => Some(Self::NumpadAdd),
                ("=", Numpad) => Some(Self::NumpadEquals),
                ("/", Numpad) => Some(Self::NumpadDivide),
                ("*", Numpad) => Some(Self::NumpadMultiply),
                (".", Numpad) => Some(Self::NumpadDecimal),
                (",", Numpad) => Some(Self::NumpadComma),

                ("A"|"a", _) => Some(Self::A),
                ("B"|"b", _) => Some(Self::B),
                ("C"|"c", _) => Some(Self::C),
                ("D"|"d", _) => Some(Self::D),
                ("E"|"e", _) => Some(Self::E),
                ("F"|"f", _) => Some(Self::F),
                ("G"|"g", _) => Some(Self::G),
                ("H"|"h", _) => Some(Self::H),
                ("I"|"i", _) => Some(Self::I),
                ("J"|"j", _) => Some(Self::J),
                ("K"|"k", _) => Some(Self::K),
                ("L"|"l", _) => Some(Self::L),
                ("M"|"m", _) => Some(Self::M),
                ("N"|"n", _) => Some(Self::N),
                ("O"|"o", _) => Some(Self::O),
                ("P"|"p", _) => Some(Self::P),
                ("Q"|"q", _) => Some(Self::Q),
                ("R"|"r", _) => Some(Self::R),
                ("S"|"s", _) => Some(Self::S),
                ("T"|"t", _) => Some(Self::T),
                ("U"|"u", _) => Some(Self::U),
                ("V"|"v", _) => Some(Self::V),
                ("W"|"w", _) => Some(Self::W),
                ("X"|"x", _) => Some(Self::X),
                ("Y"|"y", _) => Some(Self::Y),
                ("Z"|"z", _) => Some(Self::Z),

                ("`", _) => Some(Self::Grave),
                ("@", _) => Some(Self::At),
                ("(", _) => Some(Self::LBracket),
                (")", _) => Some(Self::RBracket),
                ("_", _) => Some(Self::Underline),

                ("'", _) => Some(Self::Apostrophe),
                (":", _) => Some(Self::Colon),
                (";", _) => Some(Self::Semicolon),

                _ => None
            };
        }

        let winit::keyboard::Key::Named(named_key) = &input.logical else { return None };

        use winit::keyboard::NamedKey;
        match (named_key, input.location) {
            (NamedKey::Escape, _) => Some(Self::Escape),
            (NamedKey::F1, _) => Some(Self::F1),
            (NamedKey::F2, _) => Some(Self::F2),
            (NamedKey::F3, _) => Some(Self::F3),
            (NamedKey::F4, _) => Some(Self::F4),
            (NamedKey::F5, _) => Some(Self::F5),
            (NamedKey::F6, _) => Some(Self::F6),
            (NamedKey::F7, _) => Some(Self::F7),
            (NamedKey::F8, _) => Some(Self::F8),
            (NamedKey::F9, _) => Some(Self::F9),
            (NamedKey::F10, _) => Some(Self::F10),
            (NamedKey::F11, _) => Some(Self::F11),
            (NamedKey::F12, _) => Some(Self::F12),
            (NamedKey::F13, _) => Some(Self::F13),
            (NamedKey::F14, _) => Some(Self::F14),
            (NamedKey::F15, _) => Some(Self::F15),
            (NamedKey::F16, _) => Some(Self::F16),
            (NamedKey::F17, _) => Some(Self::F17),
            (NamedKey::F18, _) => Some(Self::F18),
            (NamedKey::F19, _) => Some(Self::F19),
            (NamedKey::F20, _) => Some(Self::F20),
            (NamedKey::F21, _) => Some(Self::F21),
            (NamedKey::F22, _) => Some(Self::F22),
            (NamedKey::F23, _) => Some(Self::F23),
            (NamedKey::F24, _) => Some(Self::F24),
            (NamedKey::PrintScreen, _) => Some(Self::Snapshot),
            (NamedKey::ScrollLock, _) => Some(Self::Scroll),
            (NamedKey::Pause, _) => Some(Self::Pause),
            (NamedKey::Insert, _) => Some(Self::Insert),
            (NamedKey::Home, _) => Some(Self::Home),
            (NamedKey::Delete, _) => Some(Self::Delete),
            (NamedKey::End, _) => Some(Self::End),
            (NamedKey::PageDown, _) => Some(Self::PageDown),
            (NamedKey::PageUp, _) => Some(Self::PageUp),
            (NamedKey::ArrowLeft, _) => Some(Self::Left),
            (NamedKey::ArrowUp, _) => Some(Self::Up),
            (NamedKey::ArrowRight, _) => Some(Self::Right),
            (NamedKey::ArrowDown, _) => Some(Self::Down),
            (NamedKey::Backspace, _) => Some(Self::Backspace),
            (NamedKey::Space, _) => Some(Self::Space),

            (NamedKey::Compose, _) => Some(Self::Compose),
            (NamedKey::NumLock, _) => Some(Self::Numlock),
            (NamedKey::Tab, _) => Some(Self::Tab),

            (NamedKey::Enter, Numpad) => Some(Self::NumpadEnter),
            (NamedKey::Enter, Standard) => Some(Self::Enter),

            (NamedKey::Convert, _) => Some(Self::Convert),


            (NamedKey::Alt, Left) => Some(Self::LAlt),
            (NamedKey::Control, Left) => Some(Self::LControl),
            (NamedKey::Shift, Left) => Some(Self::LShift),
            (NamedKey::Super, Left) => Some(Self::LWin),

            (NamedKey::Alt, Right) => Some(Self::LAlt),
            (NamedKey::Control, Right) => Some(Self::LControl),
            (NamedKey::Shift, Right) => Some(Self::LShift),
            (NamedKey::Super, Right) => Some(Self::LWin),

            (NamedKey::MediaStop, _) => Some(Self::MediaStop),
            (NamedKey::Power, _) => Some(Self::Power),

            (NamedKey::Copy, _) => Some(Self::Copy),
            (NamedKey::Paste, _) => Some(Self::Paste),
            (NamedKey::Cut, _) => Some(Self::Cut),

            _ => None
        }
    }

}

#[cfg(feature="graphics")]
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct KeyInput {
    pub physical: winit::keyboard::PhysicalKey,
    pub logical: winit::keyboard::Key,
    pub location: winit::keyboard::KeyLocation,
    pub text: Option<winit::keyboard::SmolStr>,
    pub repeat: bool,
}
#[cfg(feature="graphics")]
impl KeyInput {
    pub fn from_event(event: winit::event::KeyEvent) -> Self {
        Self {
            physical: event.physical_key,
            logical: event.logical_key,
            location: event.location,
            text: event.text,
            repeat: event.repeat,
        }
    }
    pub fn as_key(&self) -> Option<Key> {
        if self.repeat { return None }
        Key::from_input(self)
    }
    pub fn is_key(&self, key: Key) -> bool {
        self.as_key() == Some(key)
    }
}


#[cfg(feature="graphics")]
pub struct KeyCollection(pub Vec<KeyInput>);
#[cfg(feature="graphics")]
impl KeyCollection {
    pub fn new(v: Vec<KeyInput>) -> Self {
        Self(v)
    }

    pub fn has_key(&self, key: Key) -> bool {
        self.0.iter().any(|i| i.is_key(key))
    }
    pub fn remove_key(&mut self, key: Key) {
        self.0.retain(|k| !k.is_key(key));
    }

    pub fn has_and_remove(&mut self, key: Key) -> bool {
        let mut has = false;
        
        self.0.retain(|k| if k.is_key(key) { has = true; false} else { true });

        has
    }
}
