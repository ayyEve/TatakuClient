#[derive(Debug)]
pub enum AudioError {
    Empty,

    ApiError(String),

    FileDoesntExist,
    DifferentSong,
}
