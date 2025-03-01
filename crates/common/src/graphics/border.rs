use crate::prelude::Color;

#[derive(Clone, Copy, Debug)]
pub struct Border {
    pub color: Color,
    pub radius: f32
}
impl Border {
    pub fn new(color:Color, radius:f32) -> Self {
        Self {
            color, 
            radius
        }
    }
}

mod lua {
    use crate::prelude::*;
    use lua::*;

    impl FromLua for Border {
        fn from_lua(lua_value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
            #[cfg(feature="debug_custom_menus")] info!("Reading Border");
            let LuaValue::Table(table) = lua_value else { return Err(FromLuaConversionError { from: "Not Table", to: "Border".to_owned(), message: Some("Not a table".to_owned()) }) }; 
            
            Ok(Border {
                color: table.get("color")?,
                radius: table.get("radius")?,
            })
        }
    }
}
