use crate::*;
use std::ffi::OsString;

pub struct GamemodeLibrary {
    pub _lib: libloading::Library,
    pub info: engine::gameplay::GamemodeInfo,
}
impl GamemodeLibrary {
    pub fn load_gamemode(path: impl AsRef<Path>) -> tataku::Result<Self> {
        let lib = unsafe {
            libloading::Library::new(lib_path(path.as_ref()))
        }.map_err(|e| tataku::Error::String(e.to_string()))?;

        // let info = **unsafe {
        //     lib.get::<&'static engine::gameplay::GamemodeInfo>(b"GAME_INFO")
        // }.map_err(|e| tataku::Error::String(e.to_string()))?;
        let info = *unsafe {
            lib.get::<fn() -> engine::gameplay::GamemodeInfo>(b"game_info")
        }.map_err(|e| tataku::Error::String(e.to_string()))?;
        let info = info();
        
        Ok(Self {
            _lib: lib,
            info,
        })
    }
}



fn lib_path(path: &Path) -> OsString {
    let filename = path.file_name().expect("fuck");
    let mut path = path.parent().unwrap().to_owned().into_os_string();

    use std::env::consts::*;
    path.push(std::path::MAIN_SEPARATOR_STR);
    path.push(DLL_PREFIX);
    path.push(filename);
    path.push(DLL_SUFFIX);
    path
}

#[test]
fn test() {
    let path = "../../../game/gamemodes/taiko";
    let a = GamemodeLibrary::load_gamemode(path).unwrap();
    println!("{:?}", a.info);
}
