#[derive(Debug)]
pub enum AudioError {
    /// audio is empty
    Empty,

    /// an api error, converted to string
    ApiError(String),

    /// specified audio file doesnt exist
    FileDoesntExist,
}
