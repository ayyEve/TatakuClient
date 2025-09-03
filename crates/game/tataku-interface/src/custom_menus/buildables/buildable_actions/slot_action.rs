use crate::prelude::*;

#[derive(Deserialize)]
#[derive(Clone, Debug, PartialEq)]
pub struct BuildableSlot {
    pub slot: BuildableValue,
    #[serde(rename="$value")] pub action: BuildableSlotAction,
}
impl BuildableSlot {
    pub fn get_action(
        &self, 
        values: &mut dyn Reflect, 
        passed_in: Option<&TatakuValue>,
    ) -> Option<LobbySlotAction> {
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
            BuildableSlotAction::ShowProfile => Some(LobbySlotAction::ShowProfile(slot)),
            BuildableSlotAction::Move => Some(LobbySlotAction::MoveTo(slot)),
            BuildableSlotAction::TransferHost => Some(LobbySlotAction::TransferHost(slot)),
            BuildableSlotAction::Lock => Some(LobbySlotAction::Lock(slot)),
            BuildableSlotAction::Unlock => Some(LobbySlotAction::Unlock(slot)),
            BuildableSlotAction::Kick => Some(LobbySlotAction::Kick(slot)),
        }
    }

    pub fn build(&mut self, values: &dyn Reflect) {
        self.slot.resolve_pre(values);
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
