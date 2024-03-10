#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}
use iced::keyboard::Modifiers;
impl Into<Modifiers> for KeyModifiers {
    fn into(self) -> Modifiers {
        let mut mods = Modifiers::empty();
        mods.set(Modifiers::CTRL, self.ctrl);
        mods.set(Modifiers::ALT, self.alt);
        mods.set(Modifiers::SHIFT, self.shift);
        mods
    }
}
impl From<Modifiers> for KeyModifiers {
    fn from(value: Modifiers) -> Self {
        Self {
            ctrl: value.control(),
            alt: value.alt(),
            shift: value.shift(),
        }
    }
}