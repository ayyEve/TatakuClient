use crate::*;
use common::reflect::*;

#[repr(C)]
#[derive(Reflect)]
#[derive(Copy, Clone, Debug, Default)]
pub enum ComboMultiplier {
    /// There is no extra combo modifier
    #[default]
    None,

    /// Always multiply the score by a custom value
    Custom(f32),

    /// Every X, increase the score multiplier by Y
    /// Think of this as a graph with x being the combo count and y being the multiplier
    Linear {
        combo: u16,
        multiplier: f32,
        /// after this combo, stop increasing the multiplier
        combo_cap: Option<u16>,
    }
}