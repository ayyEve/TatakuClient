mod build_commit;
mod build_gamemodes;

fn main() {
    build_commit::build_commit();
    build_gamemodes::build_gamemodes();
}
