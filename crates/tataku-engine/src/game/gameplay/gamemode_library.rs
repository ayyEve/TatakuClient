use crate::prelude::*;
use std::ffi::OsString;

pub struct GamemodeLibrary {
    pub _lib: libloading::Library,
    pub info: GameModeInfo,
}
impl GamemodeLibrary {
    pub fn load_gamemode(path: impl AsRef<Path>) -> TatakuResult<Self> {
        let lib = unsafe {
            libloading::Library::new(lib_path(path.as_ref()))
        }.map_err(|e| TatakuError::String(e.to_string()))?;

        let info = **unsafe {
            lib.get::<&'static GameModeInfo>(b"GAME_INFO")
        }.map_err(|e| TatakuError::String(e.to_string()))?;
        
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
    let path = "/home/ayyeve/Desktop/projects/tataku/tataku-client/target/release/gamemode_taiko";
    let a = GamemodeLibrary::load_gamemode(Path::new(path).to_path_buf()).unwrap();
    println!("{:?}", a.info)
}
