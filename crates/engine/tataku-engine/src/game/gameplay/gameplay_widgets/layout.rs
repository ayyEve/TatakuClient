use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct GameplayWidgetLayout {
    /// Where this element is anchored
    pub anchor: GameplayWidgetAnchor,

    /// How to align this element
    pub align: Alignment,

    /// How to align the inner element (if needed)
    pub inner_align: Option<Alignment>,

    /// Offset in pixels
    pub offset: Vector2,

    /// What screen size the offset was saved with
    pub screen_size: Vector2,

    /// What scale is this element set to?
    pub scale: Vector2,

    /// Is this element visible?
    pub visible: bool
}
impl GameplayWidgetLayout {
    pub const fn new_default(
        anchor: GameplayWidgetAnchor,
        align: Alignment,
        inner_align: Option<Alignment>,
        offset: Option<Vector2>,
    ) -> Self {
        let offset = match offset {
            Some(o) => o,
            None => Vector2::ZERO,
        };

        Self {
            anchor,
            align,
            inner_align,
            offset,
            screen_size: Vector2::new(1920.0, 1080.0),
            scale: Vector2::ONE,
            visible: true
        }
    }
}

#[derive(Debug)]
pub enum GameplayWidgetLayoutError {
    CyclicDependencyDetected,
    InvalidElementReference(String),
}
