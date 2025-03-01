use crate::prelude::*;
use mlua::Lua;

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
    //     let menus_maybe:LuaResult<Vec<CustomMenu>> = self.lua.context(|lua| {
    //         let menus: Vec<CustomMenu> = lua.globals().get("menus")?;
    //         Ok(menus)
    //     });

    //     if let Err(e) = &menus_maybe {
    //         error!("error converting menus: {e:?}");
    //     }

    //     menus_maybe.unwrap_or_default()
    // }

    // pub fn clear_menus(&mut self) {
    //     let ok:LuaResult<()> = self.lua.context(|lua| {
    //         let clear = lua.globals().get::<_, mlua::Function>("clear_menus")?;
    //         let _ = clear.call::<(), ()>(())?;
    //         Ok(())
    //     });
    //     if let Err(e) = ok {
    //         error!("error clearing menus: {e}")
    //     }
    // }

    pub fn parse_dimension(s: Option<String>) -> Option<taffy::Dimension> {
        let s = s?.to_lowercase();

        if s.starts_with("percent") {
            s.trim_start_matches("percent(")
                .trim_end_matches(")")
                .parse::<f32>()
                .inspect_err(|_| warn!("invalid length parameter: {s}"))
                .ok()
                .map(|n| if n >= 1.0 { n / 100.0 } else {n} )
                .map(taffy::Dimension::Percent)
        }
        else if s.starts_with("fixed") {
            s.trim_start_matches("fixed(")
                .trim_end_matches(")")
                .parse::<f32>()
                .inspect_err(|_| warn!("invalid length parameter: {s}"))
                .ok()
                .map(taffy::Dimension::Length)
        }
        else {
            match &*s {
                "auto" => Some(taffy::Dimension::Auto),
                "fill" => Some(ui::FILL),
                "shrink" => {
                    warn!("menu still using shrink!");
                    Some(ui::SHRINK)
                }
                _ => {
                    warn!("Invalid length parameter: {s}");
                    None
                }
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
