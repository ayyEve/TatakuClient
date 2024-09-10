//TODO: probably want a build script for this lol

pub mod shaders {
    pub const FLASHLIGHT: &str = include_str!("../shaders/flashlight.wgsl");
    pub const PARTICLES: &str = include_str!("../shaders/particles.wgsl");
    pub const SHADER_TEX_ARRAY: &str = include_str!("../shaders/shader_with_tex_array.wgsl");
    pub const SHADER: &str = include_str!("../shaders/shader.wgsl");
    pub const SLIDER: &str = include_str!("../shaders/slider.wgsl");
}

// pub mod locales {}

pub mod menus {
    pub const BEATMAP_SELECT: &[u8] = include_bytes!("../menus/beatmap_select_menu.lua");

    pub const LUA_INIT: &[u8] = include_bytes!("../menus/init.lua");
    pub const LOBBY_MENU: &[u8] = include_bytes!("../menus/lobby_menu.lua");

    pub const LOBBY_SELECT: &[u8] = include_bytes!("../menus/lobby_select.lua");
    pub const MAIN_MENU: &[u8] = include_bytes!("../menus/main_menu.lua");
    pub const MENU_LIST: &[u8] = include_bytes!("../menus/menu_list.lua");
}
