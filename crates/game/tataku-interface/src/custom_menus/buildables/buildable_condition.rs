use crate::prelude::*;
use common::reflect::*;
use engine::BuildableCalc;

#[derive(Deserialize)]
#[serde(from="String")]
#[derive(Clone, Debug, PartialEq)]
pub enum BuildableCondition {
    Unbuilt(ArcStr),
    Built(BuildableCalc),
    Failed,
}

#[cfg(feature="graphics")]
impl BuildableCondition {
    pub fn is_unbuilt(&self) -> bool {
        matches!(self, Self::Unbuilt(_))
    }

    pub fn build(&mut self) {
        let BuildableCondition::Unbuilt(s) = self 
        else { return };
        
        match BuildableCalc::parse(format!("{s} == true").into()) {
            Ok(built) 
                => *self = BuildableCondition::Built(built),
                
            Err(e) => {
                error!("Error building conditional: {e:?}");
                *self = BuildableCondition::Failed;
            }
        }
    }

    pub fn resolve<'a>(&'a self, values: &dyn Reflect) -> BuildableConditionResult<'a> {
        match self {
            Self::Failed => BuildableConditionResult::Failed,
            Self::Unbuilt(calc_str) => BuildableConditionResult::Unbuilt(calc_str),
            Self::Built(calc) => {
                match calc.resolve(values).map(|n| n.as_bool()) {
                    Ok(true) => BuildableConditionResult::True,
                    Ok(false) => BuildableConditionResult::False,
                    Err(e) => {
                        error!("Error with shunting yard calc. calc_str: '{}' calc: {calc:?}, error: {e:?}", calc.expr);
                        BuildableConditionResult::Error(e)
                    }
                }
            }
        }
    }
}

impl From<String> for BuildableCondition {
    fn from(value: String) -> Self {
        Self::Unbuilt(value.into())
    }
}
impl From<ArcStr> for BuildableCondition {
    fn from(value: ArcStr) -> Self {
        Self::Unbuilt(value)
    }
}

#[derive(PartialEq, Debug)]
pub enum BuildableConditionResult<'a> {
    Failed,
    Unbuilt(&'a str),
    True,
    False,
    Error(buildable_shunting_yard::Error)
}
impl From<bool> for BuildableConditionResult<'_> {
    fn from(value: bool) -> Self {
        if value {
            Self::True
        } else {
            Self::False
        }
    }
}
