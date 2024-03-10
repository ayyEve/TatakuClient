use crate::prelude::*;
use rlua::{ Value, FromLua, Error::FromLuaConversionError };

#[derive(Clone, Debug)]
pub enum CustomElementText {
    Value(String),
    Calc(String),
    /// calc but parsed, should not be read into
    CalcParsed(Arc<CustomElementCalc>, String),
    Text(String),
    Locale(String),

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
            // "dialog.confirmation.yes" vs "dialog/confirmation/yes"
            Self::Locale(s) => *s = s.replace('.', "/"),

            Self::List(items, _) => {
                for i in items { i.parse()? }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn to_string(
        &self, 
        values: &ShuntingYardValues,
    ) -> String {
        match self {
            Self::Value(t) => values.get_string(t).unwrap_or_else(|_| format!("Invalid property: '{t}'")),
            Self::Text(t) | Self::Locale(t) => t.clone(),
            
            Self::CalcParsed(calc, calc_str) => {
                match calc.resolve(values) {
                    Ok(n) => format!("{n:.2}"),
                    Err(e) => {
                        error!("Error with shunting yard calc. calc: '{calc_str}', error: {e:?}");
                        format!("Calc error! See console.")
                    }
                }
            }

            Self::Calc(_t) => panic!("You forgot to parse a calc!"),

            Self::List(items, join_str) => {
                items
                    .iter()
                    .map(|i|i.to_string(values))
                    .collect::<Vec<_>>()
                    .join(&join_str)
            }
        }
    }
}
impl<'lua> FromLua<'lua> for CustomElementText {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::prelude::LuaContext<'lua>) -> rlua::Result<Self> {

        match lua_value {
            Value::String(s) => Ok(Self::Text(s.to_str()?.to_owned())),
            Value::Table(table) => {
                if let Some(calc) = table.get::<_, Option<String>>("calc")? {
                    Ok(Self::Calc(calc))
                } else if let Some(locale) = table.get::<_, Option<String>>("locale")? {
                    Ok(Self::Locale(locale))
                } else if let Some(text) = table.get::<_, Option<String>>("text")? {
                    Ok(Self::Text(text))
                } else if let Some(value) = table.get::<_, Option<String>>("value")? {
                    Ok(Self::Value(value))
                } else if let Some(value) = table.get::<_, Option<Vec<Self>>>("list")? {
                    Ok(Self::List(value, String::new()))
                } else {
                    Err(FromLuaConversionError { from: "Table", to: "CustomElementText", message: Some("No property to get type".to_owned()) })
                }
            }
            Value::Integer(n) => {
                let Some(char) = char::from_u32(n as u32) else {
                    return Err(FromLuaConversionError { 
                        from: "Integer", 
                        to: "CustomElementText", 
                        message: Some("Failed to cast int to char".to_owned()) 
                    })
                };

                Ok(Self::Text(char.to_string()))
            }

            other => Err(FromLuaConversionError { from: other.type_name(), to: "CustomElementText", message: Some("Invalid type".to_owned()) })
        }
    }
}
