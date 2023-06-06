mod menu_elements;

pub use menu_elements::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct KeyModifiers {
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool
}
