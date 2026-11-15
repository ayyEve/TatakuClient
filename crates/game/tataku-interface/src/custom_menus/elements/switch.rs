use crate::prelude::*;
#[cfg(feature="graphics")] use ui::widget::Widget;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SwitchElement {
    // #[serde(rename = "@condition", alias = "@cond")] condition: ArcStr,
    on: Option<Wrapped<BuildableValue>>,
    #[serde(rename = "@on_enum")] on_enum: Option<engine::VariablePathResolver>,

    #[serde(rename = "case")] cases: Vec<CaseElement>,
    #[serde(rename = "default", default)] default_case: Option<Wrapped<Element>>,
}

#[cfg(feature="graphics")]
impl SwitchElement {
    pub fn build(&self) -> widgets::SwitchWidget {

        let on: Option<widgets::SwitchWidgetValue> 
        = match (self.on.clone(), self.on_enum.clone()) {
            (Some(v), None) => Some(v.inner.into()),
            (None, Some(path)) => Some(path.into()),

            (Some(_), Some(_)) => {
                error!("Switch element has both on and on_enum! {self:?}");
                None
            }

            (None, None) => None
        };

        widgets::SwitchWidget::new(
            self
                .cases
                .iter()
                .filter_map(CaseElement::build)
                .collect(),

            self
                .default_case
                .as_ref()
                .map(|i| i.inner.build().boxed()),
        )
        .value_maybe(on)
    }
}


#[derive(Deserialize)]
#[derive(Clone, Debug, PartialEq)]
struct CaseElement {
    #[serde(alias="@cond")] cond: Option<BuildableCondition>,
    #[serde(alias="@value")] value_attr: Option<tataku::TatakuValue>,
    value: Option<Wrapped<BuildableValue>>,
    #[serde(rename="$value")] element: Element,
}

#[cfg(feature="graphics")]
impl CaseElement {
    fn build(&self) -> Option<widgets::SwitchWidgetCase> {
        Some(widgets::SwitchWidgetCase::new(
            self.build_cond()?,
            self.element.build().boxed()
        ))
    }

    fn build_cond(&self) -> Option<widgets::SwitchWidgetCaseCond> {
        let value = match (&self.value, &self.value_attr) {
            (Some(v), None) => Some(v.inner.clone()),
            (None, Some(value)) => Some(BuildableValue::Value(value.clone())),

            (Some(_), Some(_)) => {
                error!("Case element has both value and value_attr! {self:?}");
                None
            }

            (None, None) => None
        };

        match (self.cond.clone(), value) {
            (Some(cond), None) => Some(cond.into()),
            (None, Some(value)) => Some(value.into()),

            (Some(_), Some(_)) => {
                error!("Case element has both cond and value! {self:?}");
                None
            }
            (None, None) => {
                error!("Case element has neither cond nor value! {self:?}");
                None
            }
        }
    }
}
