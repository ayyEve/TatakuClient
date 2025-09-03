use crate::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub struct KeyButtonElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: ArcStr,

    #[serde(rename = "@optional", default)] optional: bool,
    #[serde(rename = "@var")] var: VariablePathResolver,
    #[serde(default)] on_input: BuildableAction,
}
impl CustomElement for KeyButtonElement {
    fn build(&self) -> Box<dyn Widget<TatakuAction>> {
        WidgetContainer::new_boxed(
            self.style.clone(),
            "keyButton",
            self.id.clone(),
            self.class_list.clone(),
            KeyButton::new(self.var.clone())
            .optional(self.optional)
            .on_change(self.on_input.clone())
            .boxed()
        )
    }
}
