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
    pub fn build(&mut self) {
        let Self::Calc(calc_str) = self else { return };
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

// #[test]
// fn test() {
//     use quick_xml::de::from_str;

//     #[derive(Deserialize, PartialEq, Debug)]
//     struct Action { #[serde(rename="$value")] action: BuildableAction }
    
//     assert_eq!(
//         from_str::<Action>(r#"
//             <action>
//                 <setValue key='hi'>
//                     hi mom
//                 </setValue>
//             </action>
//         "#).unwrap(), 
//         Action {
//             action: BuildableAction::SetValue {
//                 key: "hi".into(),
//                 value: BuildableValue::Value(TatakuValue::String("hi mom".to_string())),
//             }
//         }
//     );

//     assert_eq!(
//         from_str::<Action>(r#"
//             <action>
//                 <setValue key='hi2'>
//                     100
//                 </setValue>
//             </action>
//         "#).unwrap(), 
//         Action {
//             action: BuildableAction::SetValue {
//                 key: "hi2".into(),
//                 value: BuildableValue::Value(TatakuValue::U32(100)),
//             }
//         }
//     );

//     assert_eq!(
//         from_str::<Action>(r#"
//             <action>
//                 <setValue key='hello'>
//                     <variable>tacos</variable>
//                 </setValue>
//             </action>
//         "#).unwrap(), 
//         Action {
//             action: BuildableAction::SetValue {
//                 key: "hello".into(),
//                 value: BuildableValue("tacos".to_owned().into()),
//             }
//         }
//     );

//     assert_eq!(
//         from_str::<Action>(r#"
//             <action>
//                 <setValue key='hello123'>
//                     <passedIn/>
//                 </setValue>
//             </action>
//         "#).unwrap(), 
//         Action {
//             action: BuildableAction::SetValue {
//                 key: "hello123".into(),
//                 value: BuildableValue::PassedIn
//             }
//         }
//     );

//     assert_eq!(
//         from_str::<Action>(r#"
//             <action>
//                 <song> <play/> </song>
//             </action>
//         "#).unwrap(), 
//         Action {
//             action: BuildableAction::Song(BuildableSongAction::Play)
//         }
//     );
// }
