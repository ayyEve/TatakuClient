use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct ScrollOperation {
    pub scroll_type: ScrollType,
    
    // smoothing?
}

#[derive(Clone, Debug)]
pub enum ScrollType {
    /// Scroll to the (first) selected node
    ScrollToActive {
        /// Should the children's children be included in the search
        include_children: bool,
    },

    /// Scroll to a specific node id
    ScrollToNode(NodeId),

    /// Scroll to an element with the provided id
    ScrollToId(CowStr),

    /// Absolute scroll to pixel
    ScrollToPosition(Vector2),
    /// Relative scroll to pixel
    ScrollByAmount(Vector2),
    
    /// Absolute scroll to percent
    ScrollToPercent(Vector2),
    /// Relative scroll by percent
    ScrollByPercent(Vector2),
}
