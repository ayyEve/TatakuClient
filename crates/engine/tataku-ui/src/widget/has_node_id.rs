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

impl<Action: Send + Sync> HasNodeId for Box<dyn crate::widget::Widget<Action>> {
    fn get_id(&self) -> taffy::NodeId {
        self.node_id().get_id()
    }
} 
