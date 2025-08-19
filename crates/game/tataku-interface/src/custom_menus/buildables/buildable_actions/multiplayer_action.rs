use crate::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub enum BuildableMultiplayerAction {
    /// Join a lobby
    CreateLobby {  
        name: BuildableValueTag,
        password: Option<BuildableValueTag>,
        private: BuildableValueTag,
    },

    /// Join a lobby
    JoinLobby { 
        #[serde(alias="$value")] lobby_id: BuildableValue, 
        password: Option<BuildableValueTag> 
    },

    /// Open the link to the lobby's beatmap
    OpenMapLink,

    /// Start the match
    StartMatch,

    /// Ready up
    Ready,

    /// Ready down?
    Unready,

    /// Leave a lobby
    Leave,

    /// Quit multiplayer
    Quit,

    /// start multiplayer
    #[serde(alias="start")]
    StartMultiplayer,

    // slot actions
    #[serde(alias="slot")]
    SlotAction(BuildableSlot),
}
impl BuildableMultiplayerAction {
    pub fn into_action(
        self, 
        values: &mut dyn Reflect, 
        passed_in: Option<&TatakuValue>
    ) -> Option<MultiplayerAction> {
        match self {
            Self::StartMultiplayer => Some(MultiplayerAction::StartMultiplayer),
            Self::StartMatch => Some(MultiplayerAction::LobbyAction(LobbyAction::Start)),
            Self::OpenMapLink => Some(MultiplayerAction::LobbyAction(LobbyAction::OpenMapLink)),
            Self::Leave => Some(MultiplayerAction::LobbyAction(LobbyAction::Leave)),
            Self::Quit => Some(MultiplayerAction::ExitMultiplayer),
            Self::Ready => Some(MultiplayerAction::LobbyAction(LobbyAction::Ready)),
            Self::Unready => Some(MultiplayerAction::LobbyAction(LobbyAction::Unready)),

            Self::SlotAction(action) => {
                action
                    .get_action(values, passed_in)
                    .map(|action| MultiplayerAction::LobbyAction(LobbyAction::SlotAction(action)))
            }
            
            Self::JoinLobby { lobby_id, password } => Some(MultiplayerAction::JoinLobby { 
                lobby_id: lobby_id.resolve(values, passed_in)?.as_u32()?, 
                password: password
                    .and_then(|i| i
                        .resolve(values, passed_in)
                        .map(|t| t.as_string())
                    )
                    .unwrap_or_default(),
            }),

            Self::CreateLobby { 
                name, 
                password, 
                private 
            } => Some(MultiplayerAction::CreateLobby { 
                name: name.resolve(values, passed_in)?.as_string(), 
                password: password
                    .and_then(|i| i
                        .resolve(values, passed_in)
                        .map(|t| t.as_string())
                    )
                    .unwrap_or_default(), 
                private: private
                    .resolve(values, passed_in)
                    .map(|i| i.as_bool())
                    .unwrap_or_default(), 
                players: 16
            })
        }
    }
    
    pub fn build(&mut self, values: &dyn Reflect) {
        match self {
            Self::SlotAction(slot_action) => {
                slot_action.slot.resolve_pre(values);
            }

            Self::JoinLobby { 
                lobby_id, 
                password 
            } => {
                lobby_id.resolve_pre(values);
                if let Some(password) = password.as_mut() {
                    password.resolve_pre(values);
                }
            }

            _ => {}
        }
    }
}
