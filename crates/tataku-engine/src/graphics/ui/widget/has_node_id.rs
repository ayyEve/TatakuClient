use crate::prelude::*;

pub trait HasNodeId {
    fn get_id(&self) -> taffy::NodeId;
}
impl<T: Widget> HasNodeId for &T {
    fn get_id(&self) -> taffy::NodeId {
        self.node_id().node_id
    }
}
impl<T: Widget> HasNodeId for &mut T {
    fn get_id(&self) -> taffy::NodeId {
        self.node_id().node_id
    }
}
impl HasNodeId for NodeId {
    fn get_id(&self) -> taffy::NodeId {
        self.node_id
    }
}
impl HasNodeId for taffy::NodeId {
    fn get_id(&self) -> taffy::NodeId {
        *self
    }
}
