use crate::prelude::*;
use ui::widget::Widget;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub struct KeyButtonElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: ArcStr,

    #[serde(rename = "@optional", default)] optional: bool,
    #[serde(rename = "@variable")] var: engine::VariablePathResolver,
    #[serde(default)] on_input: Wrapped<Vec<BuildableAction>>,
}
impl CustomElement for KeyButtonElement {
    fn build(&self) -> Box<dyn Widget<actions::Action>> {
        widgets::WidgetContainer::new_boxed(
            self.style.clone(),
            "keyButton",
            self.id.clone(),
            self.class_list.clone(),
            widgets::KeyButton::new(self.var.clone())
            .optional(self.optional)
            .on_change(self.on_input.inner.clone())
            .boxed()
        )
    }
}
