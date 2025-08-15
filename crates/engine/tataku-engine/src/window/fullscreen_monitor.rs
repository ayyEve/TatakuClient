
use crate::prelude::*;

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
#[derive(Reflect)]
#[reflect(display = "display")]
pub enum FullscreenMonitor {
    None,
    Monitor(ArcStr),
}
#[cfg(feature="graphics")]
impl Display for FullscreenMonitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => "none".fmt(f),
            Self::Monitor(name) => name.fmt(f),
        }
    }
}
