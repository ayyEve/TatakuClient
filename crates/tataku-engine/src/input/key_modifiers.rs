#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

#[cfg(feature="graphics")] 
impl From<KeyModifiers> for iced::keyboard::Modifiers {
    fn from(val: KeyModifiers) -> Self {
        use iced::keyboard::Modifiers;
        let mut mods = Modifiers::empty();
        mods.set(Modifiers::CTRL, val.ctrl);
        mods.set(Modifiers::ALT, val.alt);
        mods.set(Modifiers::SHIFT, val.shift);
        mods
    }
}

#[cfg(feature="graphics")]
impl From<iced::keyboard::Modifiers> for KeyModifiers {
    fn from(value: iced::keyboard::Modifiers) -> Self {
        Self {
            ctrl: value.control(),
            alt: value.alt(),
            shift: value.shift(),
        }
    }
}