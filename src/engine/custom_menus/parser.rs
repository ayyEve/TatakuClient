use crate::prelude::*;
use rlua::{ Error, Lua, FromLua };
use rlua::Value;

const LUA_INIT:&'static str = include_str!("../../../custom_menus/init.lua");

pub struct CustomMenuParser {
    lua: Lua,
}
impl CustomMenuParser {
    pub fn new() -> Self {
        let lua =  Lua::new();
        let res:Result<(), Error> = lua.context(|lua| {
            #[cfg(feature="debug_custom_menus")] info!("");
            lua.load(LUA_INIT).set_name("lua_init")?.exec()?;
            Ok(())
        });
        if let Err(e) = res { error!("Error parsing init.lua: {e:?}"); }

        Self {
            lua,
        }
    }

    /// read a file
    pub fn load(&mut self, file_path: impl AsRef<Path>) -> TatakuResult {
        let path = file_path.as_ref();
        let file_data = std::fs::read_to_string(path)?;

        let res:Result<(), Error> = self.lua.context(move |lua| {
            let menu_count:usize = lua.globals().get("menu_count")?;
            // println!("got menu count: {menu_count}");

            // run the file
            lua.load(&file_data).set_name(&path.to_string_lossy().to_string())?.exec()?;

            let menu_count2:usize = lua.globals().get("menu_count")?;
            if menu_count2 == menu_count { warn!("No menu was loaded from the file {path:?}") }

            Ok(())
        });

        if let Err(e) = res {
            error!("Error parsing file {path:?}: {e:?}");
        }

        Ok(())
    }

    pub fn get_menus(&mut self) -> Vec<CustomMenu> {
        let menus_maybe:rlua::Result<Vec<CustomMenu>> = self.lua.context(|lua| {
            let menus: Vec<CustomMenu> = lua.globals().get("menus")?;
            Ok(menus)
        });
        
        if let Err(e) = &menus_maybe {
            error!("error converting menus: {e:?}");
        }

        menus_maybe.unwrap_or_default()
    }


    pub fn parse_length(s: Option<String>) -> Option<iced::Length> {
        let s = s?;

        if s.starts_with("fill_portion") {
            let Ok(n) = s.trim_start_matches("fill_portion(").trim_end_matches(")").parse::<u16>() else { warn!("invalid length parameter: {s}"); return None };
            return Some(iced::Length::FillPortion(n))
        }
        if s.starts_with("fixed") {
            let Ok(n) = s.trim_start_matches("fixed(").trim_end_matches(")").parse::<f32>() else { warn!("invalid length parameter: {s}"); return None };
            return Some(iced::Length::Fixed(n))
        }
        match &*s {
            "fill" => Some(iced::Length::Fill),
            "shrink" => Some(iced::Length::Shrink),
            _ => {
                warn!("Invalid length parameter: {s}");
                None
            }
        }

    }

    fn color_handle_value<'lua>(value: Value<'lua>) -> rlua::Result<f32> {
        match value {
            Value::Integer(i) => Ok(i as f32 / 255.0),
            Value::Number(f) => Ok(f as f32),
            other => Err(Error::FromLuaConversionError { from: other.type_name(), to: "Color", message: Some("Not a valid number".to_owned()) })
        }
    }
}


/// color reader
impl<'lua> FromLua<'lua> for Color {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::prelude::LuaContext<'lua>) -> rlua::prelude::LuaResult<Self> {
        match lua_value {
            Value::String(s) => Color::try_from_hex(s.to_str()?).ok_or_else(||Error::FromLuaConversionError { from: "String", to: "Color", message: Some("Not a table".to_owned()) }),
            Value::Table(table) => {

                let mut vals = [None; 4];
                for (n, c) in ["r","g","b","a"].into_iter().enumerate() {
                    let mut v:Option<Value> = table.get(n+1)?;
                    if v.is_none() { v = table.get(c)? }

                    vals[n] = v.map(CustomMenuParser::color_handle_value).transpose()?;
                }

                let [Some(r), Some(g), Some(b), a] = vals else {
                    return Err(Error::FromLuaConversionError { from: "Table", to: "Color", message: Some("Invalid argument count".to_owned()) })
                };

                let a = a.unwrap_or(1.0);
                Ok(Color::new(r,g,b,a))
            }

            other => Err(Error::FromLuaConversionError { from: other.type_name(), to: "Color", message: Some("Not a table".to_owned()) })
        }

    }
}



#[test]
fn test() {
    tataku_logging::init_with_level("logs/", log::Level::Debug).unwrap();

    let mut parser = CustomMenuParser::new();
    if let Err(e) = parser.load("custom_menus/main_menu.lua") {
        error!("error: {e}");
    }

    let menus = parser.get_menus();
    info!("{menus:?}")
}