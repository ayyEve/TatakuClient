use crate::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CheckboxElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: ArcStr,

    /// value as a calc string
    #[serde(alias = "@value", default)] value: BuildableValue,
    
    /// what to run on click
    #[serde(default)] on_click: Option<BuildableAction>,
}
impl CheckboxElement {
    fn get_value(&self) -> CheckboxValue {
        match self.value.clone() {
            BuildableValue::None => CheckboxValue::Static(false),
            BuildableValue::Value(TatakuValue::Bool(b)) => CheckboxValue::Static(b),
            BuildableValue::Variable(var) => CheckboxValue::Variable {
                path: var,
                cache: false,
                failed: false,
             },
            BuildableValue::Calc(calc) => CheckboxValue::Condition(calc.into(), false),
            BuildableValue::CalcParsed { calc, calc_str } => CheckboxValue::Condition(BuildableCondition::Built(calc, calc_str), false),
            val => {
                error!("invalid checkbox (id = {:?}) value {val:?}", self.id);
                CheckboxValue::Static(false)
            },
        }
    }
}

impl CustomElement for CheckboxElement {
    fn build(&self) -> Box<dyn Widget<TatakuAction>> {
        let value = self.get_value();

        WidgetContainer::new_boxed(
            self.style.clone(), 
            "checkbox",
            self.id.clone(),
            self.class_list.clone(),
            Checkbox::new(value)
                .on_toggle_maybe(self.on_click.clone())
                .boxed()
        )
    }
}
