use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct TextInputElement {
    #[serde(rename = "@id", default)] id: Option<String>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: String,

    #[serde(rename = "@variable")] variable: String,
    #[serde(rename = "@password", default)] is_password: bool,

    #[serde(default)] placeholder: BuildableTextTag,
    #[serde(default)] on_input: Option<BuildableActionTag>,
    #[serde(default)] on_submit: Option<BuildableActionTag>,
}
impl CustomElement for TextInputElement {
    fn build(&self) -> Box<dyn Widget> {
        WidgetContainer::new_boxed(
            self.style.clone(),
            "textInput",
            self.id.clone(),
            self.class_list.clone(),
            TextInput::new(
                self.placeholder.value.clone(),
                BuildableText::Variable { 
                    variable: VariablePathResolver::new(self.variable.clone())
                }
            )
            .secure(self.is_password)
            .on_input_maybe(self.on_input.as_deref().cloned())
            .on_submit_maybe(self.on_submit.as_deref().cloned())
            .boxed()
        )
    }

}
