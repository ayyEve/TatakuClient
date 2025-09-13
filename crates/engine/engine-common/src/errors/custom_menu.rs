#[derive(Debug)]
pub enum CustomMenuError {
    /// The custom menu file is empty
    Empty,

    /// There was an issue parsing the custom menu file
    ParseError,
}
