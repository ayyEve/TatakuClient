use crate::prelude::*;
use ui::widget::Widget;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SwitchElement {
    // #[serde(rename = "@condition", alias = "@cond")] condition: ArcStr,
    #[serde(rename = "case")] cases: Vec<CaseElement>,
    #[serde(rename = "default", default)] default_case: Option<Wrapped<Element>>,
}
impl SwitchElement {
    pub fn build(&self) -> widgets::SwitchWidget {
        widgets::SwitchWidget::new(
            self
                .cases
                .iter()
                .map(|i| widgets::SwitchWidgetCase {
                    cond: i.cond.clone(),
                    widget: i.element.build().boxed(),
                })
                .collect(),

            self
                .default_case
                .as_ref()
                .map(|i| i.inner.build().boxed()),
        )
    }
}


#[derive(Deserialize)]
#[derive(Clone, Debug, PartialEq)]
pub struct CaseElement {
    #[serde(alias="@cond")] cond: BuildableCondition,
    #[serde(rename="$value")] element: Element,
}
