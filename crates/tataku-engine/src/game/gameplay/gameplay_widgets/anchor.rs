use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum GameplayWidgetAnchor {
    /// Anchored to the screen 
    /// 
    /// Position can be absolute with this + UIElementAlign::TopLeft
    #[default]
    Screen,

    /// Anchored to the playfield
    /// 
    /// field is size of screen when saved (if element should scale with playfield)
    Playfield {
        saved_size: Option<Vector2>,

        /// Where should this element be relative to the playfield
        relative: GameplayWidgetAlign,
    },

    /// Anchored to an element, scaling is determined from the parent element
    Element {
        /// What element to anchor to
        element: Cow<'static, str>,

        /// Where should this element be relative to the parent
        relative: GameplayWidgetAlign,
    },
}
impl GameplayWidgetAnchor {
    pub const fn element(
        element: &'static str, 
        relative: GameplayWidgetAlign,
    ) -> Self {
        Self::Element {
            element: Cow::Borrowed(element),
            relative
        }
    }
}
