mod build_code;

fn main() {
    #[cfg(all(feature = "bass_audio", feature = "neb_audio"))] 
    panic!("\n\n!!!!!!!!!!!!!!!!!!!!!!!!!!!\nfeatures `bass_audio` and `neb_audio` cannot be used at the same time!\nTo use neb audio, disable default features\n!!!!!!!!!!!!!!!!!!!!!!!!!!!\n\n");

    build_code::build_commit();
    build_code::build_gamemodes();
}
