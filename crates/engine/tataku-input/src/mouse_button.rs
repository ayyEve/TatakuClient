#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16)
}
