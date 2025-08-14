use crate::prelude::*;

#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct KeyButtonElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: ArcStr,

    #[serde(rename = "@variable")] variable: VariablePathResolver,
    #[serde(default)] on_input: Option<BuildableActionTag>,
}
impl CustomElement for KeyButtonElement {
    fn build(&self) -> Box<dyn Widget> {
        WidgetContainer::new_boxed(
            self.style.clone(),
            "keyButton",
            self.id.clone(),
            self.class_list.clone(),
            KeyButton::new(
               self.variable.clone(),
            )
            .on_change_maybe(self.on_input.as_deref().cloned())
            .boxed()
        )
    }
}
