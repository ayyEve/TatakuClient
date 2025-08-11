use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct SwitchElement {
    #[serde(rename = "@id", default)] id: Option<String>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: String,

    // #[serde(rename = "@condition", alias = "@cond")] condition: String,
    #[serde(rename = "case")] cases: Vec<CaseElement>,
    #[serde(rename = "default", default)] default_case: Option<ElementTag>,
}
impl CustomElement for SwitchElement {
    fn build(&self) -> Box<dyn Widget> {
        WidgetContainer::new_boxed(
            self.style.clone(),
            "switch",
            self.id.clone(),
            self.class_list.clone(),
            SwitchWidget::new(
                self
                    .cases
                    .iter()
                    .map(|i| SwitchWidgetCase {
                        cond: i.cond.clone(),
                        widget: i.element.build(),
                    })
                    .collect(),

                self
                    .default_case
                    .as_ref()
                    .map(|i| i.build()),

                // BuildableCondition::Unbuilt(self.condition.clone())
            )
            .boxed()
        )
    }
}


#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
pub struct CaseElement {
    #[serde(alias="@cond")] cond: BuildableCondition,
    #[serde(rename="$value")] element: Element,
}
