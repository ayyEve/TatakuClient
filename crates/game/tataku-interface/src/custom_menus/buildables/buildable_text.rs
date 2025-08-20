use crate::prelude::*;

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BuildableTextTag {
    #[serde(rename="$value")] pub value: BuildableText
}
crate::impl_tag!(BuildableTextTag, BuildableText, value);

#[derive(Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, Default2, PartialEq)]
pub enum BuildableText {
    #[default]
    Text {
        #[serde(rename = "@text")]
        text: ArcStr
    },

    Locale(ArcStr),
    Variable {
        #[serde(rename = "@var")] 
        variable: VariablePathResolver
    },
    
    Display {
        #[serde(rename = "@var")] 
        variable: VariablePathResolver,

        #[serde(rename = "@precision", default)] 
        precision: Option<usize>,
    },

    Calc {
        #[serde(rename = "@calc", default)] calc: Option<ArcStr>,
        #[serde(rename = "@var", default)] var: Option<VariablePathResolver>,
    },

    
    /// calc but parsed, should not be read into
    #[serde(skip)] CalcParsed(BuildableCalc, ArcStr),

    #[serde(alias = "iter")] 
    TextIter {
        #[serde(rename = "@variable")] variable: ArcStr,
        #[serde(rename = "@property")] property: Option<ArcStr>,
        #[serde(rename = "@join")] join: ArcStr,
    },

    List {
        #[serde(rename="$value")]
        list: Vec<Self>,
        
        #[serde(rename="@join", default)]
        join: ArcStr
    }
}
impl BuildableText {
    /// Parses Self::Calc into Self::CalcParsed
    pub fn compute(&mut self) -> ShuntingYardResult<()> {
        match self {
            Self::Calc { calc: Some(calc), .. } => {
                let s = calc.clone();
                *self = Self::CalcParsed(BuildableCalc::parse(&s)?, s);
            }
            // because json pointers use '/' and not '.', but '.' is nicer for locale
            // "dialog.confirmation.yes" (us) vs "dialog/confirmation/yes" (json)
            Self::Locale(s) => *s = s.replace('.', "/").into(),

            Self::List { list, .. } => {
                for i in list {
                    i.compute()?;
                }
            }

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
                    .unwrap_or_else(|e| format!("Invalid property: '{variable}' ({e:?})"))
            },
            
            Self::Text { text: t } | Self::Locale(t) => t.to_string(),
            
            Self::Display { variable, precision } => {
                let variable = match variable.resolve_path(values) {
                    Ok(v) => v,
                    Err(e) => return format!("error: {e:?}")
                };
                
                if let Ok(number) = values.reflect_as_number(&variable) {
                    match number {
                        ReflectNumber::F32(n) => format_float(n, precision.unwrap_or(2)),
                        ReflectNumber::F64(n) => format_float(n, precision.unwrap_or(2)),
                        other => format_number(i128::from(other)),
                    }
                } else {
                    values
                        .reflect_display(&variable, *precision)
                        .unwrap_or_else(|e| format!("Invalid property: '{variable}' ({e:?})"))
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

            Self::List { list, join } => {
                list.iter()
                    .map(|i| i.to_string(values))
                    .collect::<Vec<_>>()
                    .join(join)
            }

            Self::TextIter { 
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
                                
                                let str = match v {
                                    MaybeOwnedReflect::Borrowed(reflect) 
                                        => try_get_string(reflect),
                                    MaybeOwnedReflect::Owned(reflect) 
                                        => try_get_string(&*reflect),
                                };
                                
                                if let Some(s) = str {
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

            Self::Calc { var: Some(var), .. } => {
                let Ok(path) = var
                    .resolve_path(values)
                    .inspect_err(|e| 
                        error!("Error resolving var path: {e:?}")
                    )
                else { return "Error!".to_string() };

                let Ok(calc_str) = values
                    .reflect_get::<String>(&*path)
                    .inspect_err(|e| 
                        error!("Error getting var calc: {e:?}")
                    )
                else { return "Error!".to_string() };

                let Ok(calc) = BuildableCalc::parse(&*calc_str)
                    .inspect_err(|e| 
                        error!("Error parsing var calc: {e:?}")
                    )
                else { return "Error!".to_string() };
                
                match calc.resolve(values) {
                    Ok(val) => val.as_string(),
                    Err(e) => {
                        error!("Error with shunting yard calc. calc: '{}', error: {e:?}", &*calc_str);
                        "Calc error! See console.".to_string()
                    }
                }
            }

            Self::Calc { calc: None, var: None } => "No calc provided!".to_owned(),
            Self::Calc { calc: Some(calc), .. } 
                => unreachable!("Calcs should be built. unbuilt: {calc}"),
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
        let input = r#"<text text="hi mom"/>"#;
        let expected = BuildableText::Text { 
            text: "hi mom".into() 
        };
        
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
            calc: Some("hi mom".into()),
            var: None,
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

    
    #[test]
    fn test_list() {
        let input = r#" <list> <text text="hi mom"/> <text text="bye mom"/> </list> "#;
        let expected = BuildableText::List { 
            join: ArcStr::default(),
            list: vec![
                BuildableText::Text { text: "hi mom".into() },
                BuildableText::Text { text: "bye mom".into() }
            ]  
        };
        
        assert_eq!(quick_xml::de::from_str::<'_, BuildableText>(input).unwrap(), expected);
    }

    #[test]
    fn test_list_join() {
        let input = r#" <list join="uwu"> <text text="hi mom"/> <text text="bye mom"/> </list> "#;
        let expected = BuildableText::List { 
            join: "uwu".into(),
            list: vec![
                BuildableText::Text { text: "hi mom".into() },
                BuildableText::Text { text: "bye mom".into() }
            ]  
        };
        
        assert_eq!(quick_xml::de::from_str::<'_, BuildableText>(input).unwrap(), expected);
    }
}
