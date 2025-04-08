use crate::prelude::*;

#[derive(Debug)]
pub enum OnlineAction {
    SpectateHost { host_id: u32 },
    StopSpectating { host_id: u32 },
    // TODO: do we want to batch these?
    SendSpectatorFrame {
        frame: Box<SpectatorFrame>,
        force: bool,
    },
}
impl From<OnlineAction> for TatakuAction {
    fn from(value: OnlineAction) -> Self {
        TatakuAction::Online(value)
    }
}

pub enum OnlineEvent {
    LoggedIn {
        user_id: u32,
        username: String,
    },
    Disconnected,

    TatakuAction(TatakuAction),
    SpectatorEvent(SpectatorEvent),
    MultiplayerPacket(Box<MultiplayerPacket>),
    MultiplayerLobbyInvite {
        inviter_id: u32,
        inviter_username: String,
        lobby: LobbyInfo
    }
}

pub enum SpectatorEvent {
    SpectatingHost {
        host_id: u32,
        host_username: String,
    },

    SpectatorJoined {
        user_id: u32,
        username: String,
    },

    SpectatorLeft {
        user_id: u32,
    },

    SpectatorFrame {
        host: u32,
        frame: Box<SpectatorFrame>,
    },
}
