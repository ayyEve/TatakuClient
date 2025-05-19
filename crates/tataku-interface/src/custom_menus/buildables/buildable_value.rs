use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct BuildableValueTag {
    #[serde(rename="$value", alias="$text")] pub value: BuildableValue,
}
crate::impl_tag!(BuildableValueTag, BuildableValue, value);

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub enum BuildableValue {
    /// No value
    #[default] None,

    /// Literal value (number, string, bool)
    Value {
        #[serde(rename = "$value", default)] 
        value: Option<TatakuValue>,
        #[serde(rename = "@val", default)] 
        value_attribute: Option<TatakuValue>,
    },

    /// Get from a variable
    Variable {
        #[serde(rename="@var", alias="$value", default)] 
        var: String,
    },

    /// Defer the value to provided value.
    /// Basically, this is a path that points to another path
    Reference {
        #[serde(rename="$value", default)] 
        reference: Option<BuildableText>,
        #[serde(rename="$ref", default)] 
        reference_attribute: Option<String>,
    },

    /// Calculate the value from some calc string
    Calc {
        #[serde(rename="@calc", alias="$value")] 
        calc: String
    },

    #[serde(skip)]
    CalcParsed {
        calc: BuildableCalc, 
        calc_str: String,
    },

    /// Get value from a passed in value
    PassedIn,
}
impl BuildableValue {
    pub fn new_value(value: impl Into<TatakuValue>) -> Self {
        Self::Value {
            value: Some(value.into()),
            value_attribute: None
        }
    }

    /// pre-emptively resolve variables. used when the element's event requires values to be moved
    pub fn resolve_pre(&mut self, values: &dyn Reflect) {
        match self {
            Self::Variable { var } => {
                let Ok(val) = values.impl_get(ReflectPath::new(var)) else {
                    error!("custom event value is none! {var}");
                    *self = Self::None;
                    return;
                };
                let value = match TatakuValue::from_reflection(val) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("custom event value error: {var}, {e:?}");
                        *self = Self::None;
                        return
                    }
                };

                *self = Self::Value { value: Some(value), value_attribute: None };
            }
            Self::Calc { calc: calc_str } => {
                match BuildableCalc::parse(&calc_str) {
                    Ok(calc) => {
                        *self = Self::CalcParsed { calc, calc_str: calc_str.clone() }
                    }
                    Err(e) => {
                        error!("Error with calc '{calc_str}': {e:?}");
                        *self = Self::None;
                    }
                }
            }

            Self::Reference {
                reference,
                reference_attribute
            } => {
                match (reference, reference_attribute) {
                    (Some(r), _) => {
                        if let Err(e) = r.compute() {
                            error!("error with reference '{r:?}': {e:?}");
                            *self = Self::None;
                        }
                    },
                    (_, Some(_)) => {},
                    (None, None) => {
                        error!("Reference does not have a ref path!");
                        *self = Self::None;
                    }
                }
            }
            _ => {}
        }

    }

    pub fn resolve<'a:'b, 'b>(
        &'a self, 
        values: &'b dyn Reflect, 
        passed_in: &'b Option<TatakuValue>
    ) -> Option<Cow<'b, TatakuValue>> {
        match self {
            Self::None => None,
            Self::Value { 
                value, 
                value_attribute
            } => Some(Cow::Borrowed(value.as_ref().or(value_attribute.as_ref())?)),
            Self::Calc { .. } => unreachable!("Calc should be built!"),
            Self::CalcParsed { calc, .. } => {
                calc
                    .resolve(values)
                    .ok()
            }

            Self::Reference {
                reference,
                reference_attribute,
            } => {
                let var = reference
                    .as_ref()
                    .map(|r| r.to_string(values))
                    .or(reference_attribute.clone())
                    ?;

                
                let Ok(val) = values.impl_get(ReflectPath::new(&var)) else {
                    error!("custom event value is none! {var}");
                    return None;
                };
                let value = match TatakuValue::from_reflection(val) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("custom event value error: {var}, {e:?}");
                        return None
                    }
                };

                Some(Cow::Owned(value))
            }


            Self::Variable { var } => {
                let Ok(val) = values.impl_get(ReflectPath::new(var)) else {
                    error!("custom event value is none! {var}");
                    return None;
                };
                let value = match TatakuValue::from_reflection(val) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("custom event value error: {var}, {e:?}");
                        return None
                    }
                };

                Some(Cow::Owned(value))
            }
            Self::PassedIn => Some(Cow::Borrowed(passed_in.as_ref()?))
        }
    }

}

#[test]
fn test() {
    use quick_xml::de::from_str;

    #[derive(Deserialize, PartialEq, Debug)]
    struct Action { #[serde(rename="$value")] action: BuildableAction }
    
    assert_eq!(
        from_str::<Action>(r#"
            <action>
                <setValue key='hi'>
                    hi mom
                </setValue>
            </action>
        "#).unwrap(), 
        Action {
            action: BuildableAction::SetValue {
                key: "hi".to_string(),
                value: BuildableValue::Value {
                    value: Some(TatakuValue::String("hi mom".to_string())),
                    value_attribute: None
                }
            }
        }
    );

    assert_eq!(
        from_str::<Action>(r#"
            <action>
                <setValue key='hi2'>
                    100
                </setValue>
            </action>
        "#).unwrap(), 
        Action {
            action: BuildableAction::SetValue {
                key: "hi2".to_string(),
                value: BuildableValue::Value {
                    value: Some(TatakuValue::U32(100)),
                    value_attribute: None
                }
            }
        }
    );

    assert_eq!(
        from_str::<Action>(r#"
            <action>
                <setValue key='hello'>
                    <variable>tacos</variable>
                </setValue>
            </action>
        "#).unwrap(), 
        Action {
            action: BuildableAction::SetValue {
                key: "hello".to_string(),
                value: BuildableValue::Variable {
                    var: "tacos".to_owned()
                }
            }
        }
    );

    assert_eq!(
        from_str::<Action>(r#"
            <action>
                <setValue key='hello123'>
                    <passedIn/>
                </setValue>
            </action>
        "#).unwrap(), 
        Action {
            action: BuildableAction::SetValue {
                key: "hello123".to_string(),
                value: BuildableValue::PassedIn
            }
        }
    );

    assert_eq!(
        from_str::<Action>(r#"
            <action>
                <song> <play/> </song>
            </action>
        "#).unwrap(), 
        Action {
            action: BuildableAction::Song {
                action: BuildableSongAction::Play
            }
        }
    );

    

}
