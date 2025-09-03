use crate::prelude::*;

#[derive(serde::Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, Default, PartialEq)]
pub enum BuildableValue {
    /// No value
    #[default] None,

    /// Literal value (number, string, bool)
    #[serde(rename = "$text")]
    Value(TatakuValue),

    /// Get from a variable
    #[serde(alias = "var")]
    Variable(VariablePathResolver),

    /// Calculate the value from some calc string
    Calc(ArcStr),

    #[serde(skip)]
    CalcParsed {
        calc: BuildableCalc, 
        calc_str: ArcStr,
    },

    /// The value is passed in from the widget, ie a slider's value when changed
    PassedIn,
}
impl BuildableValue {
    /// pre-emptively resolve variables. used when the element's event requires values to be moved
    pub fn resolve_pre(&mut self, values: &dyn Reflect) {
        match self {
            Self::Variable(var) => {
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

                *self = Self::Value(value);
            }
            Self::Calc(calc_str) => {
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
            Self::Value(value) => Some(Cow::Borrowed(value)),
            Self::Calc(..) => unreachable!("Calc should be built!"),
            Self::CalcParsed { calc, calc_str } => {
                calc
                    .resolve(values)
                    .inspect_err(|e| 
                        error!("error with calc '{calc_str}': {e:?}")
                    )
                    .ok()
            }

            Self::Variable(var) => {
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

// impl<'de> serde::Deserialize<'de> for BuildableValue {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>
//     {
//         struct Visitor;
//         impl<'de> serde::de::Visitor<'de> for Visitor {
//             type Value = BuildableValue;

//             fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//                 write!(formatter, "literal value, variable path, calc string, or passed in")
//             }

//             fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
//             where
//                 E: serde::de::Error,
//             {
//                 Ok(BuildableValue::Value(TatakuValue::Bool(v)))
//             }

//             fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
//             where
//                 E: serde::de::Error,
//             {
//                 Ok(BuildableValue::Value(TatakuValue::F32(v as f32)))
//             }

//             fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
//             where
//                 E: serde::de::Error
//             {
//                 Ok(BuildableValue::Value(TatakuValue::U32(v)))
//             }

//             fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
//             where
//                 E: serde::de::Error,
//             {
//                 Ok(BuildableValue::Value(TatakuValue::U64(v)))
//             }

//             fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
//             where
//                 E: serde::de::Error,
//             {
//                 Ok(BuildableValue::Value(TatakuValue::F32(v)))
//             }

//             fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
//             where
//                 E: serde::de::Error
//             {
//                 Ok(BuildableValue::Value(TatakuValue::String(v)))
//             }

//             fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
//             where
//                 E: serde::de::Error,
//             {
//                 self.visit_string(v.to_owned())
//             }

//             fn visit_none<E>(self) -> Result<Self::Value, E>
//             where
//                 E: serde::de::Error,
//             {
//                 Ok(BuildableValue::None)
//             }

//             fn visit_unit<E>(self) -> Result<Self::Value, E>
//             where
//                 E: serde::de::Error
//             {

//             }

//             fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
//             where
//                 A: serde::de::MapAccess<'de>
//             {
//                 let key: Option<&str> = map.next_key()?;

//                 let key = match key {
//                     Some(key) => key,
//                     None => return Ok(BuildableValue::None),
//                 };

//                 #[derive(Deserialize)]
//                 struct GetText<T> {
//                     #[serde(rename = "$text")]
//                     text: T
//                 }

//                 match key {
//                     "none" => Ok(BuildableValue::None),
//                     "$text" => {
//                         let string: String = map.next_value()?;

//                         println!("got {string}");

//                         self.visit_string(string)
//                     },
//                     "variable" | "var" => Ok(BuildableValue::Variable {
//                         var: map.next_value::<GetText<_>>()?.text,
//                     }),
//                     "calc" => Ok(BuildableValue::Calc {
//                         calc: map.next_value::<GetText<_>>()?.text,
//                     }),
//                     "passedIn" => Ok(BuildableValue::PassedIn),
//                     field => Err(serde::de::Error::unknown_field(field, &[
//                         "none",
//                         "variable",
//                         "calc",
//                         "passedIn",
//                     ])),
//                 }
//             }
//         }

//         let result = deserializer.deserialize_enum(
//             "buildableValue",
//             &[
//                 ""
//             ]
//             Visitor
//         );

//         println!("{result:?}");

//         result
//     }
// }

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
                value: BuildableValue::Value(TatakuValue::String("hi mom".to_string())),
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
                value: BuildableValue::Value(TatakuValue::U32(100)),
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
                value: BuildableValue("tacos".to_owned().into()),
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
            action: BuildableAction::Song(BuildableSongAction::Play)
        }
    );

    

}
