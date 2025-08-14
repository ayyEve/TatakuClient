use crate::prelude::*;

#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct BuildableInput {
    /// The name of the variable
    #[serde(rename="@name")] pub name: String,
    /// The path the variable should be put in
    #[serde(rename="@path")] pub path: String,
    /// What type of variable is it?
    #[serde(rename="@type")] pub value_type: BuildableInputType,
    /// Is it required? default is false
    #[serde(rename="@required", default)] pub required: bool,

    /// A test value, only used when testing
    #[serde(rename="@testValue", alias="test", default)] pub test_value: Option<TatakuValue>,
    #[serde(rename="@default", alias="default", default)] pub default_value: Option<TatakuValue>,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub enum BuildableInputType {
    /// type doesnt matter
    Any,

    /// u32
    Integer,

    /// f32
    Float,

    /// bool
    Bool,

    /// string
    String,
}
impl BuildableInputType {
    pub fn default_value(&self) -> TatakuValue {
        match self {
            Self::Any => TatakuValue::from("hi"),
            Self::Bool => false.into(),
            Self::Float => 0.0f32.into(),
            Self::Integer => 0u32.into(),
            Self::String => String::new().into(),
        }
    }
    pub fn check_type(&self, value: &TatakuValue) -> bool {
        #[allow(clippy::match_like_matches_macro, reason = "using the matches macro would be not very readable")]
        match (self, value) {
            (_, TatakuValue::None) => false,
            (Self::Any, _) => true,
            (Self::Bool, TatakuValue::Bool(_)) => true,
            (Self::Integer, TatakuValue::U32(_) | TatakuValue::U64(_)) => true,
            (Self::Float, TatakuValue::F32(_)) => true,
            (Self::String, TatakuValue::String(_)) => true,

            _ => false
        }
    }
}


#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct BuildableInputsTag {
    #[serde(rename = "$value")] pub list: Vec<BuildableInput>,
}
impl BuildableInputsTag {
    pub fn init(
        &self, 
        mut variables: BuildableInputArguments, 
        values: &mut dyn Reflect
    ) -> Result<(), Vec<BuildableInputError>> {
        let mut errors = Vec::new();
        for variable in self.list.iter() {
            if !variables.0.contains_key(&variable.name) && variable.required && variable.default_value.is_none() {
                errors.push(BuildableInputError {
                    variable: variable.name.clone(),
                    error_type: BuildableInputErrorType::Missing,
                });
            }
            // TODO: remove previous non-required variables to avoid using previously stored values
            let Some(v) = variables.get(&variable.name).or(variable.default_value.as_ref()) else { continue };

            if !variable.value_type.check_type(v) {
                let input = variables.remove(&variable.name).unwrap();
                errors.push(BuildableInputError {
                    variable: variable.name.clone(),
                    error_type: BuildableInputErrorType::WrongType {
                        expected: variable.value_type,
                        input
                    }
                }); 
            }
        }

        if errors.is_empty() {
            for variable in self.list.iter() {
                let Some(v) = variables.remove(&variable.name) else { continue };
                let path = &variable.path;
                let r = match v {
                    TatakuValue::Bool(value) => values.reflect_insert(path, value),
                    TatakuValue::F32(value) => values.reflect_insert(path, value),
                    TatakuValue::U32(value) => values.reflect_insert(path, value),
                    TatakuValue::U64(value) => values.reflect_insert(path, value as u32),
                    TatakuValue::String(value) => values.reflect_insert(path, value),
                    TatakuValue::Reflect(v) => values.impl_insert(ReflectPath::new(path), v),
                    TatakuValue::None => unreachable!("None is filtered out by check_type")
                };

                if let Err(e) = r {
                    error!("Error inserting into values: {e:?}. Thing will probably explode soon");
                }
            }

            Ok(())
        } else {
            Err(errors)
        }
    } 
}


#[derive(Debug)]
pub struct BuildableInputError {
    pub variable: String,
    pub error_type: BuildableInputErrorType
}

#[derive(Debug)]
pub enum BuildableInputErrorType {
    Missing,
    WrongType {
        expected: BuildableInputType,
        input: TatakuValue,
    }
}
impl Display for BuildableInputErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing => write!(f, "Missing"),
            Self::WrongType { 
                expected, 
                input 
            } => write!(f, "Wrong Type (Expected {expected:?}, Got: {})", input.type_name()),
        }
    }
}