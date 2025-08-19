use crate::prelude::*;

#[derive(Copy, Clone, Debug, Default2, Hash, Eq, PartialEq)]
pub struct NodeId {
    #[default(u64::MAX.into())]
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
