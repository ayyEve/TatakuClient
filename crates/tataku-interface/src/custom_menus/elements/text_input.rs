use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct TextInputElement {
    #[serde(rename = "@id", default)] id: Option<String>,
    #[serde(rename = "@class", default)] class_list: ClassList,

    /// unparsed style string, parsed when the element is built
    #[serde(rename = "@style", default)] style: String,

    #[serde(rename = "@variable")] variable: String,
    #[serde(rename = "@password", default)] is_password: bool,

    #[serde(default)] placeholder: BuildableText,
    #[serde(alias="onInput", default)] on_input: Option<BuildableActionTag>,
    #[serde(alias="onSubmit", default)] on_submit: Option<BuildableActionTag>,
}

impl CustomElement for TextInputElement {
    fn build(&self, _shell: &mut ElementBuildShell<'_>) -> Box<dyn Widget> {
        WidgetContainer::new_boxed(
            self.style.clone(),
            "textInput",
            self.id.clone(),
            self.class_list.clone(),
            TextInput::new(
                self.placeholder.clone(),
                BuildableText::from(BuildableTextInner::Variable(self.variable.clone()))
            )
            .secure(self.is_password)
            .on_input_maybe(self.on_input.as_deref().cloned())
            .on_submit_maybe(self.on_submit.as_deref().cloned())
            .boxed()
        )
    }

}
