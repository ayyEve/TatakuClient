use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct ScrollOperation {
    pub scroll_type: ScrollType,
    
    // smoothing?
}

#[derive(Clone, Debug)]
pub enum ScrollType {
    /// scroll to a specific node id
    ScrollToNode(NodeId),

    /// scroll to an element with the provided id
    ScrollToId(CowStr),

    /// absolute scroll to pixel
    ScrollToPosition(Vector2),
    /// relative scroll to pixel
    ScrollByAmount(Vector2),
    
    /// absolute scroll to percent
    ScrollToPercent(Vector2),
    /// relative scroll by percent
    ScrollByPercent(Vector2),
}