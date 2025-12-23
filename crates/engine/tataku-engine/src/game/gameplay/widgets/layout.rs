use crate::*;
use tataku::{ Bounds, Vector2 };
use gameplay::widgets::{
    GameplayWidgetAnchor,
    GameplayWidgetContainer,
    Side,
};

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug, Default)]
pub struct GameplayWidgetLayout {
    /// Where this element is anchored
    pub anchor: GameplayWidgetAnchor,

    /// How to align this element
    pub align: tataku::Alignment,

    /// Local transform of this widget relative to
    /// the anchor and alignment.
    pub transform: graphics::Transform,
}
impl GameplayWidgetLayout {
    pub const fn new(
        anchor: GameplayWidgetAnchor,
        align: tataku::Alignment,
        transform: graphics::Transform,
    ) -> Self {
        Self {
            anchor,
            align,
            transform,
        }
    }
}

#[derive(Debug)]
pub enum GameplayWidgetLayoutError {
    CyclicDependencyDetected,
    InvalidElementReference(String),
}

