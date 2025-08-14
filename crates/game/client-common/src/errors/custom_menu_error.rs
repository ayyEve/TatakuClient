#[derive(Debug)]
pub enum CustomMenuError {
    /// The custom menu file is empty
    Empty,

    /// There was an issue parsing the custom menu file
    ParseError,
}
impl From<CustomMenuError> for super::TatakuError {
    fn from(value: CustomMenuError) -> Self {
        Self::CustomMenuError(value)
    }
}
