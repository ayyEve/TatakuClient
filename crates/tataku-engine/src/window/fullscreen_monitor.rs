
#[cfg(feature="graphics")]
use super::MONITORS;
use crate::prelude::*;

#[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
#[derive(Reflect)]
#[cfg_attr(feature="graphics", reflect(display = "display"))]
#[cfg_attr(not(feature="graphics"), reflect(display = "debug"))]
pub enum FullscreenMonitor {
    None,
    Monitor(usize),
}
#[cfg(feature="graphics")]
impl Dropdownable2 for FullscreenMonitor {
    type T = Self;
    fn variants() -> Vec<Self::T> {
        [Self::None]
            .into_iter()
            .chain((0..MONITORS.read().len())
            .map(Self::Monitor))
            .collect()
    }
}

#[cfg(feature="graphics")]
impl Display for FullscreenMonitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => "none".fmt(f),
            Self::Monitor(num) => MONITORS
                .read()
                .get(*num)
                .map_or_else(
                    || "None".to_owned(), 
                    |s| format!("({num}). {s}")
                ).fmt(f)
        }
    }
}
