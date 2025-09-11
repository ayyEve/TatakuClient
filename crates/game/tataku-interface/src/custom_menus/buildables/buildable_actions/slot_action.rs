use crate::prelude::*;
use common::reflect::*;
use tataku::TatakuValue;


#[derive(Clone, Debug, PartialEq)]
pub struct BuildableSlot {
    pub slot: BuildableValue,
    pub action: BuildableSlotAction,
}

impl<'de> Deserialize<'de> for BuildableSlot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        #[derive(Deserialize)]
        struct Slot(BuildableValue, BuildableSlotAction);

        let Slot(slot, action) = Slot::deserialize(deserializer)?;

        Ok(BuildableSlot {
            slot,
            action,
        })
    }
}


impl BuildableSlot {
    pub fn get_action(
        &self, 
        values: &mut dyn Reflect, 
        passed_in: Option<&TatakuValue>,
    ) -> Option<actions::multiplayer::LobbySlotAction> {
        let slot = match &self.slot {
            BuildableValue::None => {
                error!("slot is none?? ({:?})", self.action);
                return None;
            }
            BuildableValue::Calc { .. } => unreachable!("Calc should be built"),

            BuildableValue::CalcParsed { 
                calc, 
                calc_str 
            } => match calc.resolve(values) {
                Ok(v) => v,
                Err(e) => {
                    error!("Error with calc '{calc_str}': {e:?}");
                    return None;
                }
            },
            
            BuildableValue::Value(value) => Cow::Borrowed(value),
            
            BuildableValue::Variable(var) => {
                let path = var.resolve_path(values).ok()?;

                let var = values.reflect_as_number(&path).ok()?;
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
                    ReflectNumber::F16(n) => TatakuValue::F32(n.to_f32()),
                    ReflectNumber::BF16(n) => TatakuValue::F32(n.to_f32()),
                };
                Cow::Owned(var)
            }
            BuildableValue::PassedIn => Cow::Owned(passed_in.cloned()?),
        };

        let Some(slot_num) = slot.as_u32() else {
            warn!("Couldn't cast slot to u32 ({:?})", self.action);
            return None;
        };
        let slot = slot_num as u8;

        match self.action {
            BuildableSlotAction::ShowProfile => Some(actions::multiplayer::LobbySlotAction::ShowProfile(slot)),
            BuildableSlotAction::Move => Some(actions::multiplayer::LobbySlotAction::MoveTo(slot)),
            BuildableSlotAction::TransferHost => Some(actions::multiplayer::LobbySlotAction::TransferHost(slot)),
            BuildableSlotAction::Lock => Some(actions::multiplayer::LobbySlotAction::Lock(slot)),
            BuildableSlotAction::Unlock => Some(actions::multiplayer::LobbySlotAction::Unlock(slot)),
            BuildableSlotAction::Kick => Some(actions::multiplayer::LobbySlotAction::Kick(slot)),
        }
    }

    pub fn build(&mut self) {
        self.slot.build();
    }
}

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug, PartialEq, Eq)]
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
