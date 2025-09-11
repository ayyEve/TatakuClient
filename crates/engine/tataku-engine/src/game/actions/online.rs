use crate::*;

use common::{
    packets::*,
    network::spectator::SpectatorFrame,
};

#[derive(Clone, Debug)]
pub enum OnlineAction {
    ChatAction(actions::chat::ChatAction),

    SpectateHost { host_id: u32 },
    StopSpectating { host_id: u32 },
    // TODO: do we want to batch these? they're techncially already batched before being sent to the server
    SendSpectatorFrame {
        frame: Box<SpectatorFrame>,
        force: bool,
    },

    /// Send a packet to the network
    Packet(Box<PacketId>),
}
impl OnlineAction {
    pub fn packet(packet: impl Into<PacketId>) -> Self {
        Self::Packet(Box::new(packet.into()))
    }
}
impl From<PacketId> for OnlineAction {
    fn from(value: PacketId) -> Self {
        Self::Packet(Box::new(value))
    }
}
impl From<MultiplayerPacket> for OnlineAction {
    fn from(value: MultiplayerPacket) -> Self {
        Self::Packet(Box::new(value.into()))
    }
}
impl From<ChatPacket> for OnlineAction {
    fn from(value: ChatPacket) -> Self {
        Self::Packet(Box::new(value.into()))
    }
}
impl From<actions::chat::ChatAction> for OnlineAction {
    fn from(value: actions::chat::ChatAction) -> Self {
        Self::ChatAction(value)
    }
}
impl From<OnlineAction> for actions::Action {
    fn from(value: OnlineAction) -> Self {
        actions::Action::Online(value)
    }
}

#[derive(Debug)]
pub enum OnlineEvent {
    Connected,
    Disconnected,

    LoggedIn {
        user_id: u32,
        username: String,
    },

    // Packet(Box<PacketId>),
    SpectatorEvent(SpectatorEvent),
    MultiplayerPacket(Box<MultiplayerPacket>),
}

#[derive(Debug)]
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
