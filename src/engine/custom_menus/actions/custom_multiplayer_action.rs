use crate::prelude::*;
use rlua::{ Value, FromLua, Error::FromLuaConversionError };

#[derive(Clone, Debug)]
pub enum CustomMenuMultiplayerAction {
    // /// Join a lobby
    // JoinLobby { lobby_id: u32, password: Option<String> },

    /// Open the link to the lobby's beatmap
    OpenMapLink,

    /// Ready up
    Ready,

    /// Ready down?
    Unready,

    /// Leave a lobby
    Leave,

    /// Quit multiplayer
    Quit,


    // slot actions
    SlotAction(CustomMultiplayerSlot)
}
impl CustomMenuMultiplayerAction {
    pub fn into_action(self, _values: &mut ValueCollection) -> Option<MultiplayerAction> {
        match self {
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
            // TODO!!!! 
            // Self::JoinLobby { lobby_id } => MultiplayerAction::JoinLobby { lobby_id, password: String::new() },
        }
    }
    
    pub fn resolve(&mut self, values: &ValueCollection) {
        match self {
            Self::SlotAction(slot_action) => {
                slot_action.resolve(values);
            }

            _ => {}
        }
    }
}
impl<'lua> FromLua<'lua> for CustomMenuMultiplayerAction {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        const THIS_TYPE: &str = "CustomMenuMultiplayerAction"; 

        #[cfg(feature="debug_custom_menus")] info!("Reading {THIS_TYPE}");
        match lua_value {
            Value::String(str) => {
                #[cfg(feature="debug_custom_menus")] info!("Is String");
                match str.to_str()? {
                    "leave" => Ok(Self::Leave),
                    "quit" => Ok(Self::Quit),
                    "ready" => Ok(Self::Ready),
                    "unready" => Ok(Self::Unready),
                    "open_map_link" => Ok(Self::OpenMapLink),

                    other => Err(FromLuaConversionError { from: "String", to: THIS_TYPE, message: Some(format!("Invalid {THIS_TYPE} action: {other}")) }),
                }
            }
            Value::Table(table) => {
                #[cfg(feature="debug_custom_menus")] info!("Is Table");
                
                let id = table.get::<_, String>("id")?;
                match &*id {
                    "leave" => Ok(Self::Leave),
                    "quit" => Ok(Self::Quit),

                    "ready" => Ok(Self::Ready),
                    "unready" => Ok(Self::Unready),
                    "open_map_link" => Ok(Self::OpenMapLink),

                    other => {
                        // try to get a slot action
                        if let Ok(slot_action) = CustomMultiplayerSlot::from_table(other, &table) {
                            Ok(Self::SlotAction(slot_action))
                        } else {
                            Err(FromLuaConversionError { 
                                from: "Table", 
                                to: THIS_TYPE, 
                                message: Some(format!("Could not determine {THIS_TYPE} action: {other}")) 
                            })
                        }
                    }
                }
            }

            other => Err(FromLuaConversionError { from: other.type_name(), to: THIS_TYPE, message: None })
        }
    
    }
}


#[derive(Clone, Debug)]
pub struct CustomMultiplayerSlot {
    action: CustomMultiplayerSlotAction,
    slot: CustomEventValueType,
}
impl CustomMultiplayerSlot {
    fn from_table(id: &str, table: &rlua::Table) -> rlua::Result<Self> {
        let slot_table = table.get("slot")?;
        let slot = CustomEventValueType::from_lua(&slot_table)?;

        match &*id {
            "show_slot_profile" => Ok(Self{
                action: CustomMultiplayerSlotAction::ShowSlotProfile,
                slot
            }),
            "move_to_slot" => Ok(Self{
                action: CustomMultiplayerSlotAction::MoveToSlot,
                slot,
            }),
            "transfer_host_to_slot" => Ok(Self {
                action: CustomMultiplayerSlotAction::TransferHostToSlot,
                slot,
            }),
            "lock_slot" => Ok(Self{
                action: CustomMultiplayerSlotAction::LockSlot,
                slot,
            }),
            "unlock_slot" => Ok(Self {
                action: CustomMultiplayerSlotAction::UnlockSlot,
                slot,
            }),
            "kick_slot" => Ok(Self{
                action: CustomMultiplayerSlotAction::KickSlot,
                slot,
            }),
            
            _ => Err(FromLuaConversionError { from: "Table", to: "CustomMultiplayerSlot", message: None })
        }
    }

    fn resolve(&mut self, values: &ValueCollection) {
        let Some(slot) = self.slot.resolve(values) else {
            error!("Couldn't resolve slot: {:?} ({:?})", self.slot, self.action);
            return;
        };

        let Ok(slot_num) = slot.as_u32() else {
            warn!("Couldn't cast slot to u32");
            return;
        };

        self.slot = CustomEventValueType::Value(CustomElementValue::U32(slot_num));
    }

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

    ///
    MoveToSlot,

    /// 
    TransferHostToSlot,

    ///
    LockSlot,
    
    ///
    UnlockSlot,

    ///
    KickSlot,
}
