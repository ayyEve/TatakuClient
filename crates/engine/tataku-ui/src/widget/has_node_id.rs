pub trait HasNodeId {
    fn get_id(&self) -> taffy::NodeId;
}
impl HasNodeId for crate::tree::NodeId {
    fn get_id(&self) -> taffy::NodeId {
        self.node_id
    }
}
impl HasNodeId for taffy::NodeId {
    fn get_id(&self) -> taffy::NodeId {
        *self
    }
}
