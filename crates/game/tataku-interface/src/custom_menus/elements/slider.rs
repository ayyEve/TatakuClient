use crate::prelude::*;

#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct SliderElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: ArcStr,

    #[serde(rename = "@variable")] variable: VariablePathResolver,

    #[serde(rename = "@min", default)] min_attribute: Option<TatakuValue>,
    #[serde(rename = "@max", default)] max_attribute: Option<TatakuValue>,
    #[serde(rename = "@step", default)] step_attribute: Option<TatakuValue>,

    #[serde(rename = "min", default)] min_tag: Option<BuildableValueTag>,
    #[serde(rename = "max", default)] max_tag: Option<BuildableValueTag>,
    #[serde(rename = "step", default)] step_tag: Option<BuildableValueTag>,

    #[serde(default)] on_input: Option<BuildableActionTag>,
}
impl SliderElement {
    #[allow(clippy::ref_option, reason="matches input variable signature")]
    fn resolve(
        attribute: &Option<TatakuValue>,
        tag: &Option<BuildableValueTag>,
    ) -> SliderValue {
        if let Some(attribute) = attribute {
            match attribute {
                TatakuValue::F32(n) => SliderValue::Static(*n),
                TatakuValue::U32(n) => SliderValue::Static(*n as f32),
                TatakuValue::U64(n) => SliderValue::Static(*n as f32),
                TatakuValue::String(variable) => SliderValue::Variable { 
                    variable: variable.clone().into(), 
                    value: 0.0 
                },

                TatakuValue::Bool(_) => SliderValue::Error,
                TatakuValue::None => SliderValue::Error,
                TatakuValue::Reflect(_) 
                    => unreachable!("cannot deserialize into TatakuValue::Reflect"),
            }
        } else if let Some(tag) = tag {
            SliderValue::Buildable {
                buildable: tag.value.clone(),
                value: 0.0,
            }
        } else {
            SliderValue::Error
        }
    }
}

impl CustomElement for SliderElement {
    fn build(&self) -> Box<dyn Widget<TatakuAction>> {
        let min = Self::resolve(&self.min_attribute, &self.min_tag);
        let max = Self::resolve(&self.max_attribute, &self.max_tag);
        let mut step = Some(Self::resolve(
            &self.step_attribute, 
            &self.step_tag,
        ));
        
        if matches!(step, Some(SliderValue::Error)) { step = None };

        WidgetContainer::new_boxed(
            self.style.clone(),
            "slider",
            self.id.clone(),
            self.class_list.clone(),
            Slider::new(
                min,
                max,
                self.variable.clone(),
                self.on_input.as_deref().cloned(),
            )
            .step_maybe(step)
            .boxed()
        )
    }
}
