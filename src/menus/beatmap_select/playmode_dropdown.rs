use crate::prelude::*;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayModeDropdown {
    Mode(PlayMode)
}
impl Dropdownable for PlayModeDropdown {
    fn variants() -> Vec<Self> {
        AVAILABLE_PLAYMODES.iter().map(|p|Self::Mode(p.to_owned().to_owned())).collect()
    }

    fn display_text(&self) -> String {
        let Self::Mode(s) = self;
        gamemode_display_name(&s).to_owned()
    }

    fn from_string(s:String) -> Self {
        Self::Mode(s)
    }
}

