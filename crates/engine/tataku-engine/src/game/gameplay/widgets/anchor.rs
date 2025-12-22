use crate::*;

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
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
        /// Relative orientation of the x axis
        horizontal_side: Side,
        /// Relative orientation of the y axis
        vertical_side: Side,
    },

    /// Anchored to an element, scaling is determined from the parent element
    Element {
        /// What element to anchor to
        element: CowStr,

        /// Relative orientation of the x axis
        horizontal_side: Side,
        /// Relative orientation of the y axis
        vertical_side: Side,
    },
}

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Side {
    /// Inside the parent
    Inside,

    /// Outside the parent,
    Outside,
}
