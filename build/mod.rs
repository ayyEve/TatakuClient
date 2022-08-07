mod build_commit;
mod build_gamemodes;

fn main() {
    #[cfg(all(feature = "bass_audio", feature = "neb_audio"))] 
    panic!("\n\n!!!!!!!!!!!!!!!!!!!!!!!!!!!\nfeatures `bass_audio` and `neb_audio` cannot be used at the same time!\nTo use neb audio, disable default features\n!!!!!!!!!!!!!!!!!!!!!!!!!!!\n\n");

    build_commit::build_commit();
    build_gamemodes::build_gamemodes();

    // if this is a ci build
    if option_env!("CI_JOB_NAME").is_some() {
        let ver = env!("CARGO_PKG_VERSION");

        std::fs::write("./.client", ver).unwrap();
    }
}


