use crate::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub enum BuildableMultiplayerAction {
    /// Join a lobby
    CreateLobby {
        name: Wrapped<BuildableValue>,
        #[serde(default)]
        password: Option<Wrapped<BuildableValue>>,
        private: Wrapped<BuildableValue>,
    },

    /// Join a lobby
    JoinLobby {
        #[serde(rename="$value")]
        lobby_id: BuildableValue,
        password: Option<BuildableValue>
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
    Slot {
        #[serde(rename = "$value")]
        slot: BuildableSlot,
    },
}
impl BuildableMultiplayerAction {
    pub fn resolve(
        &self, 
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

            Self::Slot { slot } => {
                slot
                    .get_action(values, passed_in)
                    .map(|action| MultiplayerAction::LobbyAction(LobbyAction::SlotAction(action)))
            }

            Self::JoinLobby { lobby_id, password } => Some(MultiplayerAction::JoinLobby {
                lobby_id: lobby_id.resolve(values, passed_in)?.as_u32()?,
                password: password.as_ref()
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
                name: name.inner.resolve(values, passed_in)?.as_string(),
                password: password.as_ref()
                    .and_then(|i| i.inner
                        .resolve(values, passed_in)
                        .map(|t| t.as_string())
                    )
                    .unwrap_or_default(),
                private: private.inner
                    .resolve(values, passed_in)
                    .map(|i| i.as_bool())
                    .unwrap_or_default(),
                players: 16
            })
        }
    }

    pub fn build(&mut self) {
        match self {
            Self::Slot { slot } => {
                slot.build();
            }

            Self::JoinLobby {
                lobby_id,
                password
            } => {
                lobby_id.build();
                if let Some(password) = password.as_mut() {
                    password.build();
                }
            }

            _ => {}
        }
    }
}
