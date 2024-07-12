use super::MONITORS;
use crate::prelude::*;

#[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum FullscreenMonitor {
    None,
    Monitor(usize),
}
#[cfg(feature="graphics")]
impl Dropdownable2 for FullscreenMonitor {
    type T = Self;
    fn variants() -> Vec<Self::T> {
        [Self::None].into_iter().chain((0..MONITORS.read().len()).into_iter().map(|t|Self::Monitor(t))).collect()
    }
}

impl Display for FullscreenMonitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => "none".fmt(f),
            Self::Monitor(num) => MONITORS
                .read()
                .get(*num)
                .map(|s|format!("({num}). {s}"))
                .unwrap_or_else(||"None".to_owned()).fmt(f)
        }
    }
}