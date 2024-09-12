use super::prelude::*;

/// color reader
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct LuaColor(pub Color);
impl<'lua> FromLua<'lua> for LuaColor {
    fn from_lua(lua_value: Value<'lua>, _lua: LuaContext<'lua>) -> LuaResult<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading Color");
        match lua_value {
            Value::String(s) => Ok(Self(Color::try_from_hex(s.to_str()?).ok_or(FromLuaConversionError { from: "String", to: "Color", message: Some("Not a table".to_owned()) })?)),
            Value::Table(table) => {
                let mut vals = [None; 4];
                for (n, c) in ["r","g","b","a"].into_iter().enumerate() {
                    let mut v:Option<Value> = table.get(n+1)?;
                    if v.is_none() { v = table.get(c)? }

                    vals[n] = v.map(color_handle_value).transpose()?;
                }

                let [Some(r), Some(g), Some(b), a] = vals else {
                    return Err(FromLuaConversionError { from: "Table", to: "Color", message: Some("Invalid argument count".to_owned()) })
                };

                let a = a.unwrap_or(1.0);
                Ok(Self(Color::new(r,g,b,a)))
            }

            other => Err(FromLuaConversionError { from: other.type_name(), to: "Color", message: Some("Not a table".to_owned()) })
        }

    }
}


fn color_handle_value<'lua>(value: Value<'lua>) -> LuaResult<f32> {
    match value {
        Value::Integer(i) => Ok(i as f32 / 255.0),
        Value::Number(f) => Ok(f as f32),
        other => Err(FromLuaConversionError { from: other.type_name(), to: "Color", message: Some("Not a valid number".to_owned()) })
    }
}