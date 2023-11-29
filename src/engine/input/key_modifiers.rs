#[derive(Copy, Clone, Default, Debug)]
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