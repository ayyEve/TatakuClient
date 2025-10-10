use crate::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub struct GamepadButtonElement {
    #[serde(rename = "@optional", default)] optional: bool,
    #[serde(rename = "@variable")] var: engine::VariablePathResolver,
    #[serde(default)] on_input: Wrapped<Vec<BuildableAction>>,
}
impl GamepadButtonElement {
    pub fn build(&self) -> widgets::GamepadButtonInput {
        widgets::GamepadButtonInput::new(self.var.clone())
            .optional(self.optional)
            .on_change(Some(self.on_input.inner.clone()))
    }
}
