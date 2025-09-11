use crate::*;
use common::reflect::*;

#[repr(C)]
#[derive(Reflect)]
#[derive(Copy, Clone, Debug, Default)]
pub enum AffectsCombo {
    /// add one to the combo
    Increment,

    /// do nothing to the combo
    #[default]
    Ignore,

    /// reset the combo
    Reset
}
