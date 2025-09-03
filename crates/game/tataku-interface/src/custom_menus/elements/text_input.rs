use crate::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextInputElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: ArcStr,

    #[serde(rename = "@variable")] variable: String,
    #[serde(rename = "@password", default)] is_password: bool,

    #[serde(alias = "@placeholder", default)] placeholder: BuildableText,
    #[serde(default)] on_input: BuildableAction,
    #[serde(default)] on_submit: BuildableAction,
}
impl CustomElement for TextInputElement {
    fn build(&self) -> Box<dyn Widget<TatakuAction>> {
        WidgetContainer::new_boxed(
            self.style.clone(),
            "textInput",
            self.id.clone(),
            self.class_list.clone(),
            TextInput::new(
                self.placeholder.clone(),
                BuildableText::Variable { 
                    variable: VariablePathResolver::new(self.variable.clone())
                }
            )
            .secure(self.is_password)
            .on_input(self.on_input.clone())
            .on_submit(self.on_submit.clone())
            .boxed()
        )
    }

}
