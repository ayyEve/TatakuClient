use crate::prelude::*;

/// An action that deals with the Mod manager
#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BuildableModAction {
    /// Add the specified mod
    AddMod { 
        #[serde(rename="$value", alias="$text")] 
        value: BuildableValue
    },

    /// Remove the specified mod
    RemoveMod { 
        #[serde(rename="$value", alias="$text")] 
        value: BuildableValue
    },

    /// Toggle the specified mod
    ToggleMod { 
        #[serde(rename="$value", alias="$text")] 
        value: BuildableValue
    },

    /// Set the gameplay speed to the specified value
    SetSpeed { 
        #[serde(rename="$value", alias="$text")] 
        value: BuildableValue
    },

    /// add/subtract to/from the gameplay speed by the specified amount
    AddSpeed { 
        #[serde(rename="$value", alias="$text")] 
        value: BuildableValue
    },
}
impl BuildableModAction {
    pub fn into_action(
        self, 
        values: &mut dyn Reflect,
        passed_in: Option<&TatakuValue>,
    ) -> Option<ModAction> {
        match self {
            Self::AddMod { value } => value.resolve(values, passed_in).and_then(|n| n.string_maybe().cloned()).map(ModAction::AddMod),
            Self::RemoveMod { value } => value.resolve(values, passed_in).and_then(|n| n.string_maybe().cloned()).map(ModAction::RemoveMod),
            Self::ToggleMod { value } => value.resolve(values, passed_in).and_then(|n| n.string_maybe().cloned()).map(ModAction::ToggleMod),
            Self::SetSpeed { value } => value.resolve(values, passed_in).and_then(|n| n.as_f32().ok()).map(ModAction::SetSpeed),
            Self::AddSpeed { value } => value.resolve(values, passed_in).and_then(|n| n.as_f32().ok()).map(ModAction::AddSpeed),
        }
    }

    pub fn build(&mut self, values: &dyn Reflect) {
        match self {
            Self::AddMod { value } => value.resolve_pre(values),
            Self::RemoveMod { value } => value.resolve_pre(values),
            Self::ToggleMod { value } => value.resolve_pre(values),
            Self::SetSpeed { value } => value.resolve_pre(values),
            Self::AddSpeed { value } => value.resolve_pre(values),
        }
    }
}
