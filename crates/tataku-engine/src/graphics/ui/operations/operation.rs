use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct UiOperation {
    /// The tree this applies to
    pub owner: MessageOwner,
    /// The target element of the operation
    pub target: UiOperationTarget,
    /// The actual operation
    pub operation: UiOperationType,
}

#[derive(Clone, Debug)]
pub enum UiOperationType {
    Scroll(super::ScrollOperation),
    SetTab(String),
}



#[derive(Clone, Debug)]
pub enum UiOperationTarget {
    /// A specific node id
    Node(NodeId),

    /// An element with the provided id
    ElementId(CowStr),

    /// An element with the provided class
    ElementClass(CowStr),
}
impl UiOperationTarget {
    pub fn resolve(
        &self, 
        node: &dyn Widget, 
        tree: &Tree,
    ) -> bool {
        let nid = node.node_id();
        match self {
            Self::Node(node_id) => &nid == node_id,
            Self::ElementId(id) => {
                let Some(ctx) = tree.get_context(nid) else { return false };
                let Some(id2) = ctx.element_data.id.as_ref() else { return false };
                id2 == id
            }
            Self::ElementClass(class) => {
                let Some(ctx) = tree.get_context(nid) else { return false };
                for c in ctx.element_data.class_list.iter() {
                    if c == class { return true }
                }
                false
            }
        }

    }
}