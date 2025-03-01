
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right
}
impl Direction {
    pub fn reverse(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,

            Self::Left => Self::Right,
            Self::Right => Self::Left
        }
    }
}
