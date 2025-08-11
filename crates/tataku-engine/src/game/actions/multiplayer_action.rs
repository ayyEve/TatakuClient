use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum MultiplayerAction {
    // Leave multiplayer mode
    ExitMultiplayer,

    /// Start multiplayer mode
    StartMultiplayer,

    /// Join the provided lobby with the provided password
    JoinLobby {
        lobby_id: u32, 
        password: String
    },

    /// Leave our current lobby
    LeaveLobby,

    /// Create a lobby
    CreateLobby {
        name: String, 
        password: String, 
        private: bool, 
        players: u8,
    },

    /// Change the beatmap
    SetBeatmap {
        hash: Md5Hash,
        mode: Option<Arc<str>>,
    },

    InviteUser {
        user_id: u32,
    },

    /// Perform a lobby action
    LobbyAction(LobbyAction),
}

impl From<MultiplayerAction> for TatakuAction {
    fn from(value: MultiplayerAction) -> Self { Self::Multiplayer(value) }
}


#[derive(Clone, Debug)]
pub enum LobbyAction {
    /// Start the match
    Start,

    /// Ready up
    Ready,
    
    /// Unready 
    Unready,

    /// Leave the current lobby
    Leave,

    /// Open a link to the lobby's beatmap
    OpenMapLink,

    /// Send a skip request
    SendSkipRequest,

    /// Notify the lobby that we've completed the map
    MapComplete(Box<Score>),

    /// Notify the lobby of our current score data
    ScoreUpdate(Box<Score>),

    /// Perform an action on a slot
    SlotAction(LobbySlotAction),

    /// Set our user state
    SetState(LobbyUserState),

    /// Notify the lobby that our map as been loaded
    LoadComplete,

    /// Notify the lobby that our mods changed
    UpdateMods(ModManager),

    /// Give host to a user id
    ChangeHost(u32),
}

#[derive(Clone, Debug)]
pub enum LobbySlotAction {
    /// Kick the player in the provided slot
    Kick(u8),

    /// Transfer host to the user in the provided slot
    TransferHost(u8),

    /// Move to the provided slot
    MoveTo(u8),

    /// Lock the provided slot
    Lock(u8),
    
    /// Unlock the provided slot
    Unlock(u8),

    /// Show the profile for the user in the provided slot
    ShowProfile(u8),
}

impl From<LobbyAction> for TatakuAction {
    fn from(value: LobbyAction) -> Self {
        Self::Multiplayer(MultiplayerAction::LobbyAction(value))
    }
}
