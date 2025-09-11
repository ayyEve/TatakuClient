use crate::prelude::*;
use ui::widget::Widget;

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
    fn build(&self) -> Box<dyn Widget<actions::Action>> {
        let value = widgets::CheckboxValue::Condition(self.value.clone().into(), false);
        let on_toggle = widgets::CheckboxOnToggle::from_buildables(self.on_click.inner.clone());

        widgets::WidgetContainer::new_boxed(
            self.style.clone(),
            "checkbox",
            self.id.clone(),
            self.class_list.clone(),
            widgets::Checkbox::new(value)
                .on_toggle_maybe(on_toggle)
                .boxed()
        )
    }
}
