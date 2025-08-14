use crate::prelude::*;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct NodeId {
    pub node_id: taffy::NodeId,
    pub owner: MessageOwner,
}
impl NodeId {
    pub fn new(id: taffy::NodeId, owner: MessageOwner) -> Self {
        Self {
            node_id: id,
            owner,
        }
    }
}
impl Default for NodeId {
    fn default() -> Self { 
        Self {
            node_id: u64::MAX.into(),
            owner: MessageOwner::Menu,
        }
    }
}
