use crate::*;
use common::reflect::*;

#[derive(Reflect)]
#[reflect(display = "display")]
#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum FullscreenMonitor {
    #[default]
    None,
    Monitor(ArcStr),
}
impl std::fmt::Display for FullscreenMonitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => "none".fmt(f),
            Self::Monitor(name) => name.fmt(f),
        }
    }
}
