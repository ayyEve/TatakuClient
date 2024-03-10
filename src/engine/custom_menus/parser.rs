use crate::prelude::*;
use rlua::{ Error, Lua };

#[cfg(not(feature="debug_custom_menus"))]
const LUA_INIT:&'static str = include_str!("../../../custom_menus/init.lua");

pub struct CustomMenuParser {
    lua: Lua,
}
impl CustomMenuParser {
    pub fn new() -> Self {
        let lua =  Lua::new();
        let res:Result<(), Error> = lua.context(|lua| {
            #[cfg(feature="debug_custom_menus")] 
            lua.load(LUA_INIT).set_name("lua_init")?.exec()?;
            #[cfg(not(feature="debug_custom_menus"))]
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
            lua
                .load(&file_data)
                .set_name(&path.to_string_lossy().to_string())?
                .exec()?;

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

    // pub fn clear_menus(&mut self) {
    //     let ok:rlua::Result<()> = self.lua.context(|lua| {
    //         let clear = lua.globals().get::<_, rlua::Function>("clear_menus")?;
    //         let _ = clear.call::<(), ()>(())?;
    //         Ok(())
    //     });
    //     if let Err(e) = ok {
    //         error!("error clearing menus: {e}")
    //     }
    // }

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