use crate::prelude::*;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BuildableValueTag {
    #[serde(rename="$value", alias="$text")] pub value: BuildableValue,
}
crate::impl_tag!(BuildableValueTag, BuildableValue, value);

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, Default, PartialEq)]
pub enum BuildableValue {
    /// No value
    #[default] None,

    /// Literal value (number, string, bool)
    Value {
        #[serde(rename = "$value", default)] value: Option<TatakuValue>,
        #[serde(rename = "@val", default)] value_attribute: Option<TatakuValue>,
    },

    /// Get from a variable
    Variable {
        #[serde(rename="@var", alias="$value")] var: VariablePathResolver,
    },

    /// Calculate the value from some calc string
    Calc {
        #[serde(rename="@calc", default)] calc: Option<ArcStr>,
        #[serde(rename="@var", default)] var: Option<VariablePathResolver>,
    },

    #[serde(skip)]
    CalcParsed {
        calc: BuildableCalc, 
        calc_str: ArcStr,
    },

    /// The value is passed in from the widget, ie a slider's value when changed
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
                let Ok(path) = var.resolve_path(values) else { return };

                let Ok(val) = values.impl_get(ReflectPath::new(&path)) else {
                    error!("custom event value is none! {path}");
                    *self = Self::None;
                    return;
                };
                let value = match TatakuValue::from_reflection(val) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("custom event value error: {path}, {e:?}");
                        *self = Self::None;
                        return
                    }
                };

                *self = Self::Value { value: Some(value), value_attribute: None };
            }
            Self::Calc { 
                calc, 
                var,
            } => {
                if let Some(path) = var {
                    let Ok(path) = path
                        .resolve_path(values)
                        .inspect_err(|e| 
                            error!("error with calc var {path}: {e:?}")
                        )
                    else {
                        *self = Self::None;
                        return
                    };

                    let Ok(calc_str) = values
                        .reflect_display(&*path, None)
                        .inspect_err(|e| 
                            error!("error with calc var {path}: {e:?}")
                        )
                    else {
                        *self = Self::None;
                        return
                    };
                    let calc_str = ArcStr::from(calc_str);

                    match BuildableCalc::parse(&calc_str) {
                        Ok(calc) => {
                            *self = Self::CalcParsed { calc, calc_str }
                        }
                        Err(e) => {
                            error!("Error with calc '{calc_str}': {e:?}");
                            *self = Self::None;
                        }
                    }
                } else if let Some(calc_str) = calc {
                    match BuildableCalc::parse(&calc_str) {
                        Ok(calc) => {
                            *self = Self::CalcParsed { 
                                calc, 
                                calc_str: calc_str.clone()
                            }
                        }
                        Err(e) => {
                            error!("Error with calc '{calc_str}': {e:?}");
                            *self = Self::None;
                        }
                    }
                } else {
                    error!("No calc!");
                    *self = Self::None;
                }
            }

            _ => {}
        }

    }

    pub fn resolve<'a:'b, 'b>(
        &'a self, 
        values: &'b dyn Reflect, 
        passed_in: Option<&'b TatakuValue>
    ) -> Option<Cow<'b, TatakuValue>> {
        match self {
            Self::None => None,
            Self::Value { 
                value, 
                value_attribute
            } => Some(Cow::Borrowed(value.as_ref().or(value_attribute.as_ref())?)),
            Self::Calc { .. } => unreachable!("Calc should be built!"),
            Self::CalcParsed { calc, calc_str } => {
                calc
                    .resolve(values)
                    .inspect_err(|e| 
                        error!("error with calc '{calc_str}': {e:?}")
                    )
                    .ok()
            }

            Self::Variable { var } => {
                let path = var.resolve_path(values).ok()?;


                let Ok(val) = values.impl_get(ReflectPath::new(&path)) 
                else {
                    error!("custom event value is none! {path}");
                    return None;
                };
                let value = match TatakuValue::from_reflection(val) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("custom event value error: {path}, {e:?}");
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
                key: "hi".into(),
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
                key: "hi2".into(),
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
                key: "hello".into(),
                value: BuildableValue::Variable {
                    var: "tacos".to_owned().into()
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
                key: "hello123".into(),
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
