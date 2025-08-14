// TODO: convert this to bits
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}
