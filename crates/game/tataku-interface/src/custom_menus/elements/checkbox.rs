use crate::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CheckboxElement {
    /// value as a calc string
    #[serde(alias = "@value", default)] value: ArcStr,
    
    /// what to run on click
    #[serde(default)] on_click: Wrapped<Vec<BuildableAction>>,
}

#[cfg(feature="graphics")]
impl CheckboxElement {
    pub fn build(&self) -> widgets::Checkbox {
        let value = widgets::CheckboxValue::Condition(self.value.clone().into(), false);
        let on_toggle = widgets::CheckboxOnToggle::from_buildables(self.on_click.inner.clone());

        widgets::Checkbox::new(value)
            .on_toggle_maybe(on_toggle)
    }
}
