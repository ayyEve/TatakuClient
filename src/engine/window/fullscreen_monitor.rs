use super::MONITORS;
use crate::prelude::*;

#[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum FullscreenMonitor {
    None,
    Monitor(usize),
}
impl Dropdownable for FullscreenMonitor {
    fn variants() -> Vec<Self> {
        [Self::None].into_iter().chain((0..MONITORS.read().len()).into_iter().map(|t|Self::Monitor(t))).collect()
    }

    fn display_text(&self) -> String {
        match self {
            Self::None => "None".to_owned(),
            Self::Monitor(num) => MONITORS
                .read()
                .get(*num)
                .map(|s|format!("({num}). {s}"))
                .unwrap_or_else(||"None".to_owned())
        }
    }

    fn from_string(s:String) -> Self {
        match s.parse::<usize>() {
            Err(_) => Self::None,
            Ok(num) => Self::Monitor(num)
        }
    }
}
