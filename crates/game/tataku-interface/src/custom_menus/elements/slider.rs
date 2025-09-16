use crate::prelude::*;
use tataku::TatakuValue;
use widgets::SliderValue;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub struct SliderElement {
    #[serde(rename = "@variable")] var: engine::VariablePathResolver,

    #[serde(rename = "@min", default)] min_attribute: Option<TatakuValue>,
    #[serde(rename = "min", default)] min: Option<Wrapped<BuildableValue>>,

    #[serde(rename = "@max", default)] max_attribute: Option<TatakuValue>,
    #[serde(rename = "max", default)] max: Option<Wrapped<BuildableValue>>,

    #[serde(rename = "@step", default)] step_attribute: Option<TatakuValue>,
    #[serde(rename = "step", default)] step: Option<Wrapped<BuildableValue>>,

    #[serde(default)] on_input: Wrapped<Vec<BuildableAction>>,
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
                value: 0.0, 
                error_printed: false
            },

            BuildableValue::Value(TatakuValue::Bool(_)) => SliderValue::Error,
            BuildableValue::Value(TatakuValue::None) => SliderValue::Error,
            BuildableValue::Value(TatakuValue::Reflect(_))
                => unreachable!("cannot deserialize into TatakuValue::Reflect"),

            BuildableValue::Variable(var) => SliderValue::Variable { variable: var, value: 0.0, error_printed: false, },
            buildable => SliderValue::Buildable { buildable, value: 0.0 },
        }
    }
}

impl SliderElement {
    pub fn build(&self) -> widgets::Slider {
        let min = self.min_attribute.clone()
            .map(BuildableValue::Value)
            .or(self.min.clone()
                .map(|v| v.inner)
            )
            .map(Self::resolve)
            .unwrap();

        let max = self.max_attribute.clone()
            .map(BuildableValue::Value)
            .or(self.max.clone()
                .map(|v| v.inner)
            )
            .map(Self::resolve)
            .unwrap();

        let mut step = self.step_attribute.clone()
            .map(BuildableValue::Value)
            .or(self.step.clone()
                .map(|v| v.inner)
            )
            .map(Self::resolve);
        
        if matches!(step, Some(SliderValue::Error)) { step = None };

        widgets::Slider::new(
            min,
            max,
            self.var.clone(),
            Some(self.on_input.inner.clone()),
        )
        .step_maybe(step)
    }
}
