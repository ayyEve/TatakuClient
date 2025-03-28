use crate::prelude::*;
use lua::*;

#[derive(Clone, Debug)]
pub enum CustomMenuMultiplayerAction {
    /// Join a lobby
    JoinLobby { 
        lobby_id: CustomEventValueType, 
        password: Option<CustomEventValueType> 
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
    StartMultiplayer,

    // slot actions
    SlotAction(CustomMultiplayerSlot),

}
impl CustomMenuMultiplayerAction {
    pub fn into_action(self, values: &mut dyn Reflect, passed_in: Option<TatakuValue>) -> Option<MultiplayerAction> {
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
                    .get_action()
                    .map(|action| MultiplayerAction::LobbyAction(LobbyAction::SlotAction(action)))
            }
            
            Self::JoinLobby { lobby_id, password } => Some(MultiplayerAction::JoinLobby { 
                lobby_id: lobby_id.resolve(values, passed_in.clone())?.as_u32().ok()?, 
                password: password.and_then(|i| i.resolve(values, passed_in)).map(|i| i.as_string()).unwrap_or_default(),
            }),
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
impl FromLua for CustomMenuMultiplayerAction {
    fn from_lua(lua_value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        const THIS_TYPE: &str = "CustomMenuMultiplayerAction"; 

        #[cfg(feature="debug_custom_menus")] info!("Reading {THIS_TYPE}");
        match lua_value {
            LuaValue::String(str) => {
                #[cfg(feature="debug_custom_menus")] info!("Is String");
                match &*str.to_str()? {
                    "start_match" => Ok(Self::StartMatch),
                    "leave" => Ok(Self::Leave),
                    "ready" => Ok(Self::Ready),
                    "unready" => Ok(Self::Unready),
                    "open_map_link" => Ok(Self::OpenMapLink),

                    "start" => Ok(Self::StartMultiplayer),
                    "quit" => Ok(Self::Quit),

                    other => Err(FromLuaConversionError { 
                        from: "String", 
                        to: THIS_TYPE.to_owned(), 
                        message: Some(format!("Invalid {THIS_TYPE} action: {other}")) 
                    }),
                }
            }
            LuaValue::Table(table) => {
                #[cfg(feature="debug_custom_menus")] info!("Is Table");
                
                let id = table.get::<String>("id")?;
                match &*id {
                    "start_match" => Ok(Self::StartMatch),
                    "leave" => Ok(Self::Leave),
                    "quit" => Ok(Self::Quit),

                    "ready" => Ok(Self::Ready),
                    "unready" => Ok(Self::Unready),
                    "open_map_link" => Ok(Self::OpenMapLink),

                    "join_lobby" => Ok(Self::JoinLobby { 
                        lobby_id: table.get("lobby_id")?, 
                        password: table.get("password")?,
                    }),

                    other => {
                        // try to get a slot action
                        if let Ok(slot_action) = CustomMultiplayerSlot::from_table(other, &table) {
                            Ok(Self::SlotAction(slot_action))
                        } else {
                            Err(FromLuaConversionError { 
                                from: "Table", 
                                to: THIS_TYPE.to_owned(), 
                                message: Some(format!("Could not determine {THIS_TYPE} action: {other}")) 
                            })
                        }
                    }
                }
            }

            other => Err(FromLuaConversionError { 
                from: other.type_name(), 
                to: THIS_TYPE.to_owned(), 
                message: None
            })
        }
    
    }
}


#[derive(Clone, Debug)]
pub struct CustomMultiplayerSlot {
    action: CustomMultiplayerSlotAction,
    slot: CustomEventValueType,
}
impl CustomMultiplayerSlot {
    fn from_table(id: &str, table: &LuaTable) -> LuaResult<Self> {
        let slot_table = table.get("slot")?;
        let slot = CustomEventValueType::from_lua(&slot_table)?;

        match id {
            "show_slot_profile" => Ok(Self {
                action: CustomMultiplayerSlotAction::ShowSlotProfile,
                slot
            }),
            "move_to_slot" => Ok(Self {
                action: CustomMultiplayerSlotAction::MoveToSlot,
                slot,
            }),
            "transfer_host_to_slot" => Ok(Self {
                action: CustomMultiplayerSlotAction::TransferHostToSlot,
                slot,
            }),
            "lock_slot" => Ok(Self {
                action: CustomMultiplayerSlotAction::LockSlot,
                slot,
            }),
            "unlock_slot" => Ok(Self {
                action: CustomMultiplayerSlotAction::UnlockSlot,
                slot,
            }),
            "kick_slot" => Ok(Self {
                action: CustomMultiplayerSlotAction::KickSlot,
                slot,
            }),
            
            _ => Err(FromLuaConversionError { 
                from: "Table", 
                to: "CustomMultiplayerSlot".to_owned(), 
                message: None 
            })
        }
    }

    // fn build(&mut self, values: &ValueCollection, passed_in: Option<TatakuValue>) {
    //     let Some(slot) = self.slot.resolve(values, passed_in) else {
    //         error!("Couldn't resolve slot: {:?} ({:?})", self.slot, self.action);
    //         return;
    //     };

    //     let Ok(slot_num) = slot.as_u32() else {
    //         warn!("Couldn't cast slot to u32");
    //         return;
    //     };

    //     self.slot = CustomEventValueType::Value(TatakuVariable::new_any(TatakuValue::U32(slot_num)));
    // }

    fn get_action(&self) -> Option<LobbySlotAction> {
        let slot = match &self.slot {
            CustomEventValueType::None => {
                error!("slot is none?? ({:?})", self.action);
                return None;
            }
            CustomEventValueType::Value(val) => val,
            CustomEventValueType::Variable(_) => {
                error!("slot is variable?? ({:?})", self.action);
                return None;
            }
            CustomEventValueType::PassedIn => {
                error!("slot is passed in?? ({:?})", self.action);
                return None;
            }
        };

        let Ok(slot_num) = slot.as_u32() else {
            warn!("Couldn't cast slot to u32 ({:?})", self.action);
            return None;
        };
        let slot = slot_num as u8;

        match self.action {
            CustomMultiplayerSlotAction::ShowSlotProfile => Some(LobbySlotAction::ShowProfile(slot)),
            CustomMultiplayerSlotAction::MoveToSlot => Some(LobbySlotAction::MoveTo(slot)),
            CustomMultiplayerSlotAction::TransferHostToSlot => Some(LobbySlotAction::TransferHost(slot)),
            CustomMultiplayerSlotAction::LockSlot => Some(LobbySlotAction::Lock(slot)),
            CustomMultiplayerSlotAction::UnlockSlot => Some(LobbySlotAction::Unlock(slot)),
            CustomMultiplayerSlotAction::KickSlot => Some(LobbySlotAction::Kick(slot)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum CustomMultiplayerSlotAction {

    /// Show the user profile for a slot
    ShowSlotProfile,

    /// Move yourself to the slot
    MoveToSlot,

    /// Transfer host to the user in the slot
    TransferHostToSlot,

    /// Lock the slot
    LockSlot,
    
    /// Unlock the slot
    UnlockSlot,

    /// Kick the user in the slot
    KickSlot,
}
