
/// TODO: somehow merge this with alignment? 
/// my brain just isnt working properly enough to manually calculate this 
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum GameplayWidgetAlign {
    /// Inside the parent
    Inside, 

    /// Above the parent
    Above, 

    /// Below the parent
    Below,

    /// To the left of the parent
    Left, 

    /// To the right of the parent
    Right
}
impl GameplayWidgetAlign {
    pub fn inside(&self) -> bool {
        matches!(self, Self::Inside)
    }
    pub fn vertical(&self) -> bool {
        matches!(self, Self::Above | Self::Below)
    }
    pub fn horizontal(&self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
}
