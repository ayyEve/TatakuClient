use crate::prelude::*;
use common::reflect::*;
use engine::{
    shunting_yards::buildable::ShuntingYardResult,
    VariablePathResolver
};

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, Default2, PartialEq)]
pub enum BuildableText {
    #[default]
    #[serde(alias = "$text")]
    Text(ArcStr),

    Locale(ArcStr),
    #[serde(alias="var")]
    Variable {
        #[serde(rename = "$text")] variable: VariablePathResolver
    },
    
    Display {
        #[serde(rename = "$text")] variable: VariablePathResolver,
        #[serde(rename = "@precision", default)] precision: Option<usize>,
    },

    Calc {
        #[serde(rename = "$text")] calc: ArcStr
    },

    
    /// calc but parsed, should not be read into
    #[serde(skip)] CalcParsed(BuildableCalc, ArcStr),

    Iter {
        #[serde(rename = "@variable")] variable: ArcStr,
        #[serde(rename = "@property")] property: Option<ArcStr>,
        #[serde(rename = "@join")] join: ArcStr,
    },
}
impl BuildableText {
    /// Parses Self::Calc into Self::CalcParsed
    pub fn compute(&mut self) -> ShuntingYardResult<()> {
        match self {
            Self::Calc { calc } => {
                let s = calc.clone();
                *self = Self::CalcParsed(BuildableCalc::parse(&s)?, s);
            }
            // because json pointers use '/' and not '.', but '.' is nicer for locale
            // "dialog.confirmation.yes" (us) vs "dialog/confirmation/yes" (json)
            Self::Locale(s) => *s = s.replace('.', "/").into(),

            _ => {}
        }

        Ok(())
    }

    pub fn to_string(&self, values: &dyn Reflect) -> String {
        match self {
            Self::Variable { variable } => {
                let variable = match variable.resolve_path(values) {
                    Ok(v) => v,
                    Err(e) => return format!("error: {e:?}")
                };

                values
                    .reflect_display(&variable, None)
                    .unwrap_or_else(|e| 
                        format!("Invalid property: '{variable}' ({e:?})")
                    )
            },
            
            Self::Text(t) | Self::Locale(t) => t.to_string(),
            
            Self::Display { variable, precision } => {
                let variable = match variable.resolve_path(values) {
                    Ok(v) => v,
                    Err(e) => return format!("error: {e:?}")
                };
                
                if let Ok(number) = values.reflect_as_number(&variable) {
                    let precis = precision.unwrap_or(2);
                    match number {
                        ReflectNumber::F16(n) => tataku::format_float(&n, precis),
                        ReflectNumber::F32(n) => tataku::format_float(&n, precis),
                        ReflectNumber::F64(n) => tataku::format_float(&n, precis),
                        ReflectNumber::BF16(n) => tataku::format_float(&n, precis),
                        other => tataku::format_number(&i128::from(other)),
                    }
                } else {
                    values
                        .reflect_display(&variable, *precision)
                        .unwrap_or_else(|e| 
                            format!("Invalid property: '{variable}' ({e:?})")
                        )
                }
            },

            Self::CalcParsed(calc, calc_str) => {
                match calc.resolve(values) {
                    Ok(val) => val.as_string(),
                    Err(e) => {
                        error!("Error with shunting yard calc. calc: '{calc_str}', error: {e:?}");
                        "Calc error! See console.".to_string()
                    }
                }
            }

            Self::Iter {
                variable, 
                property, 
                join 
            } => {
                match values.reflect_iter(variable) {
                    Ok(iter) => {
                        let mut list = Vec::new();

                        for i in iter {
                            if let Some(property) = &property {
                                let Ok(v) = i
                                    .impl_get(ReflectPath::new(property))
                                    .inspect_err(|e| 
                                        error!("error with text iter prop: {e:?}")
                                    ) 
                                else { return String::new() };

                                if let Some(s) = try_get_string(v.as_ref()) {
                                    list.push(s);
                                }
                            } else if let Some(s) = try_get_string(i.item) {
                                list.push(s);
                            }
                        }

                        list.join(join)
                    }
                    Err(e) => {
                        error!("error with text iter: {e:?}");
                        "ERROR!!!".into()
                    }
                }
            }
            Self::Calc { calc }
                => panic!("Calcs should be built. unbuilt: {calc}"),
        }
    }
}


fn try_get_string(r: &dyn Reflect) -> Option<String> {
    match r.downcast_ref::<String>().cloned() {
        Some(s) => Some(s),
        None => r.downcast_ref::<&str>().map(|s| (*s).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text() {
        let input = r#"<text>hi mom</text>"#;
        let expected = BuildableText::Text("hi mom".into());
        
        assert_eq!(quick_xml::de::from_str::<'_, BuildableText>(input).unwrap(), expected);
    }
    
    #[test]
    fn test_variable() {
        let input = r#" <variable var="hi mom"/> "#;
        let expected = BuildableText::Variable { 
            variable: VariablePathResolver::new("hi mom".to_owned()) 
        };
        
        assert_eq!(quick_xml::de::from_str::<'_, BuildableText>(input).unwrap(), expected);
    }

    #[test]
    fn test_calc() {
        let input = r#" <calc calc="hi mom"/> "#;
        let expected = BuildableText::Calc { 
            calc: "hi mom".into()
        };
        
        assert_eq!(quick_xml::de::from_str::<'_, BuildableText>(input).unwrap(), expected);
    }

    #[test]
    fn test_display() {
        let input = r#" <display var="hi mom"/> "#;
        let expected = BuildableText::Display { 
            variable: VariablePathResolver::new("hi mom".to_owned()),
            precision: None
        };
        
        assert_eq!(quick_xml::de::from_str::<'_, BuildableText>(input).unwrap(), expected);
    }

    #[test]
    fn test_display_precision() {
        let input = r#" <display var="hi mom" precision="4" /> "#;
        let expected = BuildableText::Display { 
            variable: VariablePathResolver::new("hi mom".to_owned()),
            precision: Some(4)
        };
        
        assert_eq!(quick_xml::de::from_str::<'_, BuildableText>(input).unwrap(), expected);
    }

    
    // #[test]
    // fn test_list() {
    //     let input = r#" <list> <text text="hi mom"/> <text text="bye mom"/> </list> "#;
    //     let expected = BuildableText::List {
    //         join: ArcStr::default(),
    //         list: vec![
    //             BuildableText::Text("hi mom".into()),
    //             BuildableText::Text("bye mom".into()),
    //         ]
    //     };
        
    //     assert_eq!(quick_xml::de::from_str::<'_, BuildableText>(input).unwrap(), expected);
    // }

    // #[test]
    // fn test_list_join() {
    //     let input = r#" <list join="uwu"> <text text="hi mom"/> <text text="bye mom"/> </list> "#;
    //     let expected = BuildableText::List {
    //         join: "uwu".into(),
    //         list: vec![
    //             BuildableText::Text("hi mom".into()),
    //             BuildableText::Text("bye mom".into()),
    //         ]
    //     };
        
    //     assert_eq!(quick_xml::de::from_str::<'_, BuildableText>(input).unwrap(), expected);
    // }
}
