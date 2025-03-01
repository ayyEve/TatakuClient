use crate::prelude::*;
use lua::*;

#[derive(Clone, Debug)]
pub enum CustomElementText {
    Variable(String),
    Calc(String),
    /// calc but parsed, should not be read into
    CalcParsed(Arc<CustomElementCalc>, String),
    Text(String),
    Locale(String),

    TextIter {
        variable: String,
        property: Option<String>,
        join: String,
    },

    List(Vec<Self>, String),
}
impl CustomElementText {
    /// Parses Self::Calc into Self::CalcParsed
    pub fn parse(&mut self) -> ShuntingYardResult<()> {
        match self {
            Self::Calc(s) => {
                let s = s.clone();
                *self = Self::CalcParsed(Arc::new(CustomElementCalc::parse(&s)?), s)
            }
            // because json pointers use '/' and not '.', but '.' is nicer for locale
            // "dialog.confirmation.yes" (us) vs "dialog/confirmation/yes" (json)
            Self::Locale(s) => *s = s.replace('.', "/"),

            Self::List(items, _) => {
                for i in items { i.parse()? }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn to_string(&self, values: &dyn Reflect) -> String {
        match self {
            Self::Variable(t) => values.reflect_get::<String>(t).as_deref().cloned().unwrap_or_else(|_| format!("Invalid property: '{t}'")),
            Self::Text(t) | Self::Locale(t) => t.clone(),

            Self::CalcParsed(calc, calc_str) => {
                match calc.resolve(values) {
                    Ok(val) => val.as_string(),
                    Err(e) => {
                        error!("Error with shunting yard calc. calc: '{calc_str}', error: {e:?}");
                        "Calc error! See console.".to_string()
                    }
                }
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
                                let Ok(v) = i.impl_get(ReflectPath::new(property))
                                    .inspect_err(|e| error!("error with text iter prop: {e:?}")) 
                                    else { return String::new() };
                                
                                let str = match v {
                                    MaybeOwnedReflect::Borrowed(reflect) => try_get_string(reflect),
                                    MaybeOwnedReflect::Owned(reflect) => try_get_string(&*reflect),
                                };
                                
                                if let Some(s) = str {
                                    list.push(s);
                                }
                            } else if let Some(s) = try_get_string(i) {
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

            Self::Calc(_t) => panic!("You forgot to parse a calc!"),

            Self::List(items, join_str) => {
                items
                    .iter()
                    .map(|i| i.to_string(values))
                    .collect::<Vec<_>>()
                    .join(join_str)
            }
        }
    }
}

impl FromLua for CustomElementText {
    fn from_lua(lua_value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading text");

        match lua_value {
            LuaValue::String(s) => Ok(Self::Text(s.to_str()?.to_owned())),
            LuaValue::Table(table) => {
                #[cfg(feature="debug_custom_menus")] info!("Is table");
                if let Some(calc) = table.get::<Option<String>>("calc")? {
                    Ok(Self::Calc(calc))
                } else if let Some(locale) = table.get::<Option<String>>("locale")? {
                    Ok(Self::Locale(locale))
                } else if let Some(text) = table.get::<Option<String>>("text")? {
                    Ok(Self::Text(text))
                } else if let Some(variable) = table.get::<Option<String>>("variable")? {
                    if let Some(join) = table.get::<Option<String>>("join")? {
                        Ok(Self::TextIter { 
                            property: table.get("property")?, 
                            variable, 
                            join,
                        })
                    } else {
                        Ok(Self::Variable(variable))
                    }
                } else if let Some(value) = table.get::<Option<Vec<Self>>>("list")? {
                    Ok(Self::List(value, String::new()))
                } else if let Some(first) = table.get::<Option<Self>>(0)? {
                    #[cfg(feature="debug_custom_menus")] info!("Is table/array");

                    let mut list = vec![first];
                    for i in 1.. {
                        let Some(entry) = table.get(i)? else { break };
                        list.push(entry);
                    }

                    Ok(Self::List(list, String::new()))
                } else {
                    Err(FromLuaConversionError { 
                        from: "Table", 
                        to: "CustomElementText".to_owned(), 
                        message: Some("No property to get type".to_owned()) 
                    })
                }
            }
            LuaValue::Integer(n) => {
                #[cfg(feature="debug_custom_menus")] info!("Is Integer");
                let Some(char) = char::from_u32(n as u32) else {
                    return Err(FromLuaConversionError {
                        from: "Integer",
                        to: "CustomElementText".to_owned(),
                        message: Some("Failed to cast int to char".to_owned())
                    })
                };

                Ok(Self::Text(char.to_string()))
            }

            other => Err(FromLuaConversionError { 
                from: other.type_name(), 
                to: "CustomElementText".to_owned(), 
                message: Some("Invalid type".to_owned()) 
            })
        }
    }
}


fn try_get_string(r: &dyn Reflect) -> Option<String> {
    match r.downcast_ref::<String>().cloned() {
        Some(s) => Some(s),
        None => r.downcast_ref::<&str>().map(|s| s.to_string())
    }
}
