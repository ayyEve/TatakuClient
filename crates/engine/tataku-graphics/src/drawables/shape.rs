
/// The shape of the rectangle corners
#[derive(Copy, Clone, Debug, PartialEq)]
#[derive(serde::Deserialize)]
pub enum Shape {
    /// Square corners
    Square,

    /// Round corners
    Round(f32),

    /// Round corners with separate vals
    /// tl,tr, bl,br
    RoundSep([f32; 4]),
}
impl From<[f32;4]> for Shape {
    fn from(value: [f32;4]) -> Self {
        Self::RoundSep(value)
    }
}
