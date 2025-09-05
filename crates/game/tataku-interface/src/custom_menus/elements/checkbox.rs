use crate::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CheckboxElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: ArcStr,

    /// value as a calc string
    #[serde(alias = "@value", default)] value: ArcStr,
    
    /// what to run on click
    #[serde(default)] on_click: Wrapped<Vec<BuildableAction>>,
}

impl CustomElement for CheckboxElement {
    fn build(&self) -> Box<dyn Widget<TatakuAction>> {
        let value = CheckboxValue::Condition(self.value.clone().into(), false);
        let on_toggle = CheckboxOnToggle::from_buildables(self.on_click.inner.clone());

        WidgetContainer::new_boxed(
            self.style.clone(),
            "checkbox",
            self.id.clone(),
            self.class_list.clone(),
            Checkbox::new(value)
                .on_toggle_maybe(on_toggle)
                .boxed()
        )
    }
}
