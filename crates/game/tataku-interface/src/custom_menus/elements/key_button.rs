use crate::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub struct KeyButtonElement {
    #[serde(rename = "@optional", default)] optional: bool,
    #[serde(rename = "@variable")] var: engine::VariablePathResolver,
    #[serde(default)] on_input: Wrapped<Vec<BuildableAction>>,
}
impl KeyButtonElement {
    pub fn build(&self) -> widgets::KeyButton {
        widgets::KeyButton::new(self.var.clone())
            .optional(self.optional)
            .on_change(self.on_input.inner.clone())
    }
}
