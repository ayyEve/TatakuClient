use super::prelude::*;

impl<'lua> FromLua<'lua> for Shape {
    fn from_lua(lua_value: Value<'lua>, _lua: LuaContext<'lua>) -> LuaResult<Self> {
        match lua_value {
            Value::Integer(i) => Ok(Self::Round(i as f32)),
            Value::Number(n) => Ok(Self::Round(n as f32)),
            Value::Table(table) => {
                if let Some(round) = table.get("round")? {
                    Ok(Self::Round(round))
                } else {
                    todo!("i got lazy")
                }

            }

            other => Err(FromLuaConversionError { from: other.type_name(), to: "Shape", message: Some("Invalid type".to_owned()) })
        }
    }
}