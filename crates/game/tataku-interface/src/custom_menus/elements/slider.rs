use crate::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub struct SliderElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: ArcStr,

    #[serde(rename = "@var")] var: VariablePathResolver,

    #[serde(rename = "min", alias = "@min", default)] min: BuildableValue,
    #[serde(rename = "max", alias = "@max", default)] max: BuildableValue,
    #[serde(rename = "step", alias = "@step", default)] step: BuildableValue,

    #[serde(default)] on_input: Option<BuildableAction>,
}
impl SliderElement {
    fn resolve(value: BuildableValue) -> SliderValue {
        match value {
            BuildableValue::None => SliderValue::Error,
            BuildableValue::Value(TatakuValue::F32(n)) => SliderValue::Static(n),
            BuildableValue::Value(TatakuValue::U32(n)) => SliderValue::Static(n as f32),
            BuildableValue::Value(TatakuValue::U64(n)) => SliderValue::Static(n as f32),
            BuildableValue::Value(TatakuValue::String(variable)) => SliderValue::Variable {
                variable: variable.clone().into(),
                value: 0.0
            },

            BuildableValue::Value(TatakuValue::Bool(_)) => SliderValue::Error,
            BuildableValue::Value(TatakuValue::None) => SliderValue::Error,
            BuildableValue::Value(TatakuValue::Reflect(_))
                => unreachable!("cannot deserialize into TatakuValue::Reflect"),

            BuildableValue::Variable(var) => SliderValue::Variable { variable: var, value: 0.0 },
            buildable => SliderValue::Buildable { buildable, value: 0.0 },
        }
    }
}

impl CustomElement for SliderElement {
    fn build(&self) -> Box<dyn Widget<TatakuAction>> {
        let min = Self::resolve(self.min.clone());
        let max = Self::resolve(self.max.clone());
        let mut step = Some(Self::resolve(self.step.clone()));
        
        if matches!(step, Some(SliderValue::Error)) { step = None };

        WidgetContainer::new_boxed(
            self.style.clone(),
            "slider",
            self.id.clone(),
            self.class_list.clone(),
            Slider::new(
                min,
                max,
                self.var.clone(),
                self.on_input.clone(),
            )
            .step_maybe(step)
            .boxed()
        )
    }
}
