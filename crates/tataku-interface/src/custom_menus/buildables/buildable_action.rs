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
    #[serde(alias = "$value", alias = "$text")] 
    Value(TatakuValue),

    /// Get from a variable
    Variable(String),

    /// Get value from a passed in value
    PassedIn,
}
impl BuildableValue {
    pub fn new_value(value: impl Into<TatakuValue>) -> Self {
        Self::Value(value.into())
    }

    /// pre-emptively resolve variables. used when the element's event requires values to be moved
    pub fn resolve_pre(&mut self, values: &dyn Reflect) {
        if let Self::Variable(var) = self {
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

            *self = Self::Value(value)
        }
    }

    pub fn resolve(&self, values: &dyn Reflect, passed_in: Option<TatakuValue>) -> Option<TatakuValue> {
        match self {
            Self::None => None,
            Self::Value(val) => Some(val.clone()),
            Self::Variable(var) => {
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

                Some(value)
            }
            Self::PassedIn => passed_in
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
                value: BuildableValue::Value(TatakuValue::String("hi mom".to_string()))
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
                value: BuildableValue::Value(TatakuValue::U32(100))
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
                value: BuildableValue::Variable("tacos".to_owned())
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
