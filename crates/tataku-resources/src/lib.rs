//TODO: probably want a build script for this lol

// pub mod locales {}

pub mod menus {
    pub const BEATMAP_SELECT: &[u8] = include_bytes!("../menus/beatmap_select_menu.xml");
    pub const LOBBY_MENU: &[u8] = include_bytes!("../menus/lobby_menu.xml");
    pub const LOBBY_SELECT: &[u8] = include_bytes!("../menus/lobby_select.xml");
    pub const MAIN_MENU: &[u8] = include_bytes!("../menus/main_menu.xml");
    pub const MENU_LIST: &[u8] = include_bytes!("../menus/menu_list.xml");

    pub const FAIL_MENU: &[u8] = include_bytes!("../menus/fail_menu.xml");
    pub const PAUSE_MENU: &[u8] = include_bytes!("../menus/pause_menu.xml");
}
