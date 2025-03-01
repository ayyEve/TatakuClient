
/// TODO: somehow merge this with alignment? 
/// my brain just isnt working properly enough to manually calculate this 
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
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
