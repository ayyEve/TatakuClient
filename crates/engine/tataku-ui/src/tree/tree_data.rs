use crate::*;
use crate::tree::*;
use crate::spatial_navigation::Direction;

#[derive(Clone, Default2)]
pub struct TreeData {
    // pub bounds: Bounds,
    pub absolute_bounds: Bounds,
    pub local_transform: graphics::Transform,

    #[default(Matrix::identity())]
    pub global_transform: Matrix,
    
    #[default(Matrix::identity())]
    pub inverse_global_transform: Matrix,

    pub needs_inverse_transform: bool,

    pub selected: Option<bool>,
    adjacent_nodes: [Option<NodeId>; 4],

    pub element_data: ElementData,
}
impl TreeData {
    pub fn selectable(&self) -> bool {
        self.selected.is_some()
    }
    pub fn set_selectable(&mut self, selectable: bool) {
        self.selected = selectable.then_some(false);
    }

    pub fn node_direction(&self, direction: Direction) -> Option<NodeId> {
        self.adjacent_nodes[direction as usize]
    }
    pub fn set_node_direction(&mut self, direction: Direction, node: Option<NodeId>) {
        self.adjacent_nodes[direction as usize] = node;
    }
}
