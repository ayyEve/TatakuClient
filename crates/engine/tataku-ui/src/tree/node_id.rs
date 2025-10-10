use crate::*;
use crate::message::MessageSource;

#[derive(Default2)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct NodeId {
    #[default(u64::MAX.into())]
    pub node_id: taffy::NodeId,
    pub source: MessageSource,
}
impl NodeId {
    pub fn new(id: taffy::NodeId, source: MessageSource) -> Self {
        Self {
            node_id: id,
            source,
        }
    }
}
