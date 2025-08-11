use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct CheckboxElement {
    #[serde(rename = "@id", default)] id: Option<String>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: String,

    /// buildable text in the body
    #[serde(default)] text: Option<BuildableTextTag>,
    /// just a raw text string
    #[serde(rename = "@text", default)] text_attribute: Option<String>,

    /// value as buildable path in body
    #[serde(default)] value: Option<TatakuValue>,
    /// value as a calc string
    #[serde(rename = "@value", default)] value_calc: Option<String>,
    
    /// what to run on click
    #[serde(default)] on_click: Option<BuildableActionTag>,
}
impl CheckboxElement {
    fn get_text(&self) -> CheckboxText {
        if let Some(text) = self.text.clone() {
            CheckboxText::Buildable(text.value, String::new())
        } else if let Some(text) = self.text_attribute.clone() {
            CheckboxText::Static(text)
        } else {
            CheckboxText::Static(String::new())
        }
    }
    fn get_value(&self) -> CheckboxValue {
        if let Some(value) = self.value.clone() {
            match value {
                TatakuValue::Bool(value) => CheckboxValue::Static(value),
                TatakuValue::String(variable) => CheckboxValue::Variable {
                    path: variable.into(),
                    cache: false,
                    failed: false,
                },

                other => {
                    warn!("invalid checkbox value: {:?}", other);
                    CheckboxValue::Static(false)
                },
            }
        } else if let Some(value) = self.value_calc.clone() {
            CheckboxValue::condition(value)
        } else {
            CheckboxValue::Static(false)
        }
    }
}

impl CustomElement for CheckboxElement {
    fn build(&self) -> Box<dyn Widget> {
        let text = self.get_text();
        let value = self.get_value();

        WidgetContainer::new_boxed(
            self.style.clone(), 
            "checkbox",
            self.id.clone(),
            self.class_list.clone(),
            Checkbox::new(text, value)
                .on_toggle_maybe(self.on_click.clone().map(|i| i.action))
                .boxed()
        )
    }
}
