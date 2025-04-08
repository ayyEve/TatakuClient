use crate::prelude::*;

#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
pub struct BuildableSlot {
    pub slot: BuildableValueTag,
    #[serde(rename="$value")] pub action: BuildableSlotAction,
}
impl BuildableSlot {
    pub fn get_action(&self, values: &mut dyn Reflect, passed_in: Option<TatakuValue>) -> Option<LobbySlotAction> {
        let slot = match &self.slot.value {
            BuildableValue::None => {
                error!("slot is none?? ({:?})", self.action);
                return None;
            }
            BuildableValue::Value(val) => Cow::Borrowed(val),
            BuildableValue::Variable(var) => {
                let var = values.reflect_as_number(var).ok()?;
                let var = match var {
                    ReflectNumber::F32(n) => TatakuValue::F32(n),
                    ReflectNumber::F64(n) => TatakuValue::F32(n as f32),
                    ReflectNumber::U8(n) => TatakuValue::U32(n as u32),
                    ReflectNumber::I8(n) => TatakuValue::U32(n as u32),
                    ReflectNumber::U16(n) => TatakuValue::U32(n as u32),
                    ReflectNumber::I16(n) => TatakuValue::U32(n as u32),
                    ReflectNumber::U32(n) => TatakuValue::U32(n),
                    ReflectNumber::I32(n) => TatakuValue::U32(n as u32),
                    ReflectNumber::U64(n) => TatakuValue::U64(n),
                    ReflectNumber::I64(n) => TatakuValue::U64(n as u64),
                    ReflectNumber::U128(n) => TatakuValue::U64(n as u64),
                    ReflectNumber::I128(n) => TatakuValue::U64(n as u64),
                    ReflectNumber::Usize(n) => TatakuValue::U64(n as u64),
                    ReflectNumber::Isize(n) => TatakuValue::U64(n as u64),
                };
                Cow::Owned(var)
            }
            BuildableValue::PassedIn => Cow::Owned(passed_in?),
        };

        let Ok(slot_num) = slot.as_u32() else {
            warn!("Couldn't cast slot to u32 ({:?})", self.action);
            return None;
        };
        let slot = slot_num as u8;

        match self.action {
            BuildableSlotAction::ShowProfile => Some(LobbySlotAction::ShowProfile(slot)),
            BuildableSlotAction::Move => Some(LobbySlotAction::MoveTo(slot)),
            BuildableSlotAction::TransferHost => Some(LobbySlotAction::TransferHost(slot)),
            BuildableSlotAction::Lock => Some(LobbySlotAction::Lock(slot)),
            BuildableSlotAction::Unlock => Some(LobbySlotAction::Unlock(slot)),
            BuildableSlotAction::Kick => Some(LobbySlotAction::Kick(slot)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub enum BuildableSlotAction {

    /// Show the user profile for a slot
    ShowProfile,

    /// Move yourself to the slot
    Move,

    /// Transfer host to the user in the slot
    TransferHost,

    /// Lock the slot
    Lock,
    
    /// Unlock the slot
    Unlock,

    /// Kick the user in the slot
    Kick,
}
