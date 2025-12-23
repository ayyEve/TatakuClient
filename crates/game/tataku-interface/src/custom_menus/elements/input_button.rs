use std::marker::PhantomData;

use crate::{menu_widgets::InputButtonType, prelude::*};

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub struct InputButtonElement<T: InputButtonType> {
    #[serde(rename = "@optional", default)] optional: bool,
    #[serde(rename = "@variable")] var: engine::VariablePathResolver,
    #[serde(default)] on_input: Wrapped<Vec<BuildableAction>>,

    _t: PhantomData<T>
}
impl<T: InputButtonType> InputButtonElement<T> {
    pub fn build(&self) -> widgets::InputButton<T> {
        let on_input = self.on_input.inner.clone();
        let on_input = (!on_input.is_empty()).then_some(on_input);

        widgets::InputButton::new(self.var.clone().into())
            .optional(self.optional)
            .on_change(on_input)
    }
}
