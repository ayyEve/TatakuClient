use crate::prelude::*;

#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
#[serde(from="String")]
pub enum BuildableCondition {
    Unbuilt(String),
    Built(BuildableCalc, String),
    Failed,
}
impl BuildableCondition {
    pub fn is_unbuilt(&self) -> bool {
        matches!(self, Self::Unbuilt(_))
    }

    pub fn build(&mut self) {
        let BuildableCondition::Unbuilt(s) = self else { return };
        match BuildableCalc::parse(format!("{s} == true")) {
            Ok(built) 
                => *self = BuildableCondition::Built(built, s.clone()),
                
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
            Self::Built(calc, calc_str) => {
                match calc.resolve(values).map(|n| n.as_bool()) {
                    Ok(true) => BuildableConditionResult::True,
                    Ok(false) => BuildableConditionResult::False,
                    Err(e) => {
                        error!("Error with shunting yard calc. calc_str: '{calc_str}' calc: {calc:?}, error: {e:?}");
                        BuildableConditionResult::Error(e)
                    }
                }
            }
        }
    }
}

impl From<String> for BuildableCondition {
    fn from(value: String) -> Self {
        Self::Unbuilt(value)
    }
}

#[derive(PartialEq, Debug)]
pub enum BuildableConditionResult<'a> {
    Failed,
    Unbuilt(&'a String),
    True,
    False,
    Error(BuildableShuntingYardError)
}
