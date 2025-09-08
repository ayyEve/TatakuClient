use crate::prelude::*;

/// An action that deals with the Mod manager
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub enum BuildableModAction {
    /// Add the specified mod
    AddMod {
        #[serde(rename="$value")]
        value: BuildableValue
    },

    /// Remove the specified mod
    RemoveMod {
        #[serde(rename="$value")]
        value: BuildableValue
    },

    /// Toggle the specified mod
    ToggleMod {
        #[serde(rename="$value")]
        value: BuildableValue
    },

    /// Set the gameplay speed to the specified value
    SetSpeed {
        #[serde(rename="$value")]
        value: BuildableValue
    },

    /// add/subtract to/from the gameplay speed by the specified amount
    AddSpeed {
        #[serde(rename="$value")]
        value: BuildableValue
    },
}
impl BuildableModAction {
    pub fn resolve(
        &self, 
        values: &mut dyn Reflect,
        passed_in: Option<&TatakuValue>,
    ) -> Option<ModAction> {
        match self {
            Self::AddMod { value } => value
                .resolve(values, passed_in)
                .and_then(|n| n.string_maybe().cloned())
                .map(ModAction::AddMod),

            Self::RemoveMod { value } => value
                .resolve(values, passed_in)
                .and_then(|n| n.string_maybe().cloned())
                .map(ModAction::RemoveMod),

            Self::ToggleMod { value } => value
                .resolve(values, passed_in)
                .and_then(|n| n.string_maybe().cloned())
                .map(ModAction::ToggleMod),

            Self::SetSpeed { value } => value
                .resolve(values, passed_in)
                .and_then(|n| n.as_f32())
                .map(ModAction::SetSpeed),

            Self::AddSpeed { value } => value
                .resolve(values, passed_in)
                .and_then(|n| n.as_f32())
                .map(ModAction::AddSpeed),
        }
    }

    pub fn build(&mut self) {
        match self {
            Self::AddMod { value } => value.build(),
            Self::RemoveMod { value } => value.build(),
            Self::ToggleMod { value } => value.build(),
            Self::SetSpeed { value } => value.build(),
            Self::AddSpeed { value } => value.build(),
        }
    }
}
