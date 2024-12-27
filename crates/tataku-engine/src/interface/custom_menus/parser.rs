use crate::prelude::*;
use rlua::Lua;

pub struct CustomMenuParser {
    lua: Lua,
}
impl CustomMenuParser {
    pub fn new() -> TatakuResult<Self> {
        let lua =  Lua::new();

        // #[cfg(feature="debug_custom_menus")] {
        //     let bytes = std::fs::read("../menus/init.lua").unwrap();
        //     lua.load(&bytes).set_name("lua_init").exec()?;
        // }
        // #[cfg(not(feature="debug_custom_menus"))]
        lua.load(tataku_resources::menus::LUA_INIT).set_name("lua_init").exec()?;

        Ok(Self {
            lua,
        })
    }

    /// read a file
    pub fn load_menu(&mut self, file_path: impl AsRef<Path>) -> TatakuResult<CustomMenu> {
        let path = file_path.as_ref();
        let file_data = std::fs::read(path)?;
        self.load_menu_from_bytes(&file_data, &path.to_string_lossy().to_string())
    }

    pub fn load_menu_from_bytes(&mut self, data: &[u8], name: &String) -> TatakuResult<CustomMenu> {
        // let menu_count:usize = lua.globals().get("menu_count")?;
        // println!("got menu count: {menu_count}");

        // run the file
        self.lua
            .load(data)
            .set_name(name)
            .exec()?;

        // let menu_count2:usize = lua.globals().get("menu_count")?;
        // if menu_count2 == menu_count { warn!("No menu was loaded from the file {path:?}") }

        Ok(self.lua
            .globals()
            .get("new_menu")?)
    }

    // pub fn get_menus(&mut self) -> Vec<CustomMenu> {
    //     let menus_maybe:rlua::Result<Vec<CustomMenu>> = self.lua.context(|lua| {
    //         let menus: Vec<CustomMenu> = lua.globals().get("menus")?;
    //         Ok(menus)
    //     });

    //     if let Err(e) = &menus_maybe {
    //         error!("error converting menus: {e:?}");
    //     }

    //     menus_maybe.unwrap_or_default()
    // }

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




// #[test]
// fn test() {
//     tataku_logging::init_with_level("logs/", log::Level::Debug).unwrap();

//     let mut parser = CustomMenuParser::new();
//     if let Err(e) = parser.load_menu("custom_menus/main_menu.lua") {
//         error!("error: {e}");
//     }

//     let menus = parser.get_menus();
//     info!("{menus:?}")
// }
