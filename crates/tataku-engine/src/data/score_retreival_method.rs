use crate::prelude::*;

#[derive(Reflect)]
#[reflect(from_string = "auto")]
#[reflect(display = "display")]
#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub enum ScoreRetreivalMethod {
    #[default]
    Local,
    LocalMods,
    Global,
    GlobalMods,

    OgGame,
    OgGameMods,
    // Friends,
    // FriendsMods
}
impl ScoreRetreivalMethod {
    pub fn list() -> Vec<Self> {
        vec![
            Self::Local,
            Self::LocalMods,
            
            Self::Global,
            Self::GlobalMods,

            Self::OgGame,
            Self::OgGameMods,

            // Self::Friends,
            // Self::FriendsMods
        ]
    }

    pub fn filter_by_mods(&self) -> bool {
        match self {
            Self::Local 
            | Self::OgGame 
            // | Self::Friends
            | Self::Global => false,

            Self::LocalMods
            // | Self::FriendsMods
            | Self::OgGameMods
            | Self::GlobalMods => true,
        }
    }
}
impl Display for ScoreRetreivalMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
