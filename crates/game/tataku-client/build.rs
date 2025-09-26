fn main() {
    #[cfg(feature="dynamic_gamemodes")]
    dynamic_gamemodes::run();
}

#[cfg(feature="dynamic_gamemodes")]
mod dynamic_gamemodes {
    use std::path::Path;
    const PATH: &str = "../../../game/gamemodes/";

    fn run() {
        println!("cargo::rerun-if-changed={PATH}");

        let path = Path::new(PATH);
        std::fs::create_dir_all(path).expect("no gamemodes dir");

        let profile = std::env::var("PROFILE").expect("no PROFILE env");

        let a = format!("../../../target/{profile}");
        let build_path = Path::new(&a);

        // taiko
        copy(build_path, path, "taiko");
    }

    fn copy(
        build_path: &Path,
        output_path: &Path,
        name: &str,
    ) {
        std::fs::copy(
            build_path.join(lib_name(&format!("gamemode_{name}"))),
            output_path.join(lib_name(name)),
        ).expect("no copy");
    }

    fn lib_name(name: &str) -> String {
        use std::env::consts::*;
        format!("{DLL_PREFIX}{name}{DLL_SUFFIX}")
    }

}
