use crate::prelude::*;
use widgets::WidgetText;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub struct TextInputElement {
    #[serde(rename = "@variable")] variable: engine::VariablePathResolver,
    #[serde(rename = "@password", default)] is_password: bool,

    #[serde(rename = "@placeholder", default)] placeholder_attribute: Option<ArcStr>,
    #[serde(default)] placeholder: Option<Wrapped<BuildableText>>,

    #[serde(default)] on_input: Wrapped<Vec<BuildableAction>>,
    #[serde(default)] on_submit: Wrapped<Vec<BuildableAction>>,
}
impl TextInputElement {
    fn placeholder(&self) -> WidgetText {
        let placeholder = self.placeholder.clone()
            .map(|p| p.inner)
            .or(self.placeholder_attribute.clone()
                .map(BuildableText::Text)
            )
            .unwrap_or_default();

        match placeholder.clone() {
            BuildableText::Text(t) | BuildableText::Locale(t) => WidgetText::String(t.to_string().into()),
            buildable => WidgetText::Custom { custom: vec![buildable], cached: String::new() }
        }
    }
}
impl TextInputElement {
    pub fn build(&self) -> widgets::TextInput {
        let on_input = self.on_input.inner.clone();
        let on_input = (!on_input.is_empty()).then_some(on_input);

        let on_submit = self.on_submit.inner.clone();
        let on_submit = (!on_submit.is_empty()).then_some(on_submit);

        widgets::TextInput::new(
            self.placeholder(),
            BuildableText::Variable {
                variable: self.variable.clone()
            }
        )
        .secure(self.is_password)
        .on_input(on_input)
        .on_submit(on_submit)
    }

}
