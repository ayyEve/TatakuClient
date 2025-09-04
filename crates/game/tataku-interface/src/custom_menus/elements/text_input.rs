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

    #[serde(rename = "@placeholder", default)] placeholder_attribute: Option<ArcStr>,
    #[serde(default)] placeholder: Option<Wrapped<BuildableText>>,

    #[serde(default)] on_input: Wrapped<BuildableAction>,
    #[serde(default)] on_submit: Wrapped<BuildableAction>,
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
impl CustomElement for TextInputElement {
    fn build(&self) -> Box<dyn Widget<TatakuAction>> {
        WidgetContainer::new_boxed(
            self.style.clone(),
            "textInput",
            self.id.clone(),
            self.class_list.clone(),
            TextInput::new(
                self.placeholder(),
                BuildableText::Variable { 
                    variable: VariablePathResolver::new(self.variable.clone())
                }
            )
            .secure(self.is_password)
            .on_input(self.on_input.inner.clone())
            .on_submit(self.on_submit.inner.clone())
            .boxed()
        )
    }

}
