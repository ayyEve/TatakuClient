#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

#[cfg(feature="graphics")] 
impl Into<iced::keyboard::Modifiers> for KeyModifiers {
    fn into(self) -> iced::keyboard::Modifiers {
        use iced::keyboard::Modifiers;
        let mut mods = Modifiers::empty();
        mods.set(Modifiers::CTRL, self.ctrl);
        mods.set(Modifiers::ALT, self.alt);
        mods.set(Modifiers::SHIFT, self.shift);
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