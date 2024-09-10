use crate::prelude::*;

#[derive(Debug)]
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
        mode: Option<String>,
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


#[derive(Debug)]
pub enum LobbyAction {

    /// Ready up
    Ready,
    
    /// Unready 
    Unready,

    /// Leave the current lobby
    Leave,

    /// Open a link to the lobby's beatmap
    OpenMapLink,

    /// Perform an action on a slot
    SlotAction(LobbySlotAction),
}

#[derive(Debug)]
pub enum LobbySlotAction {
    /// Kick the player in the provided slot
    Kick(u8),

    /// Transfer host to the user in the provided slot
    TransferHost(u8),

    /// Move to the provided slot
    MoveTo(u8),

    Lock(u8),
    
    Unlock(u8),

    /// Show the profile for the user in the provided slot
    ShowProfile(u8),
}
