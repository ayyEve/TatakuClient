mod build_commit;
mod build_gamemodes;

fn main() {
    // build_commit::build_commit();
    build_gamemodes::build_gamemodes();

    // if this is a ci build
    if option_env!("CI_JOB_NAME").is_some() {
        let ver = env!("CARGO_PKG_VERSION");

        std::fs::write("./.client", ver).unwrap();
    }
}


