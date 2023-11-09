use crate::prelude::*;

#[derive(Clone, ScrollableGettersSetters)]
pub struct GenericNode {
    pos: Vector2,
    size: Vector2,

    style: Style,
    node: Node,
}
impl GenericNode {
    pub fn new(style: Style, layout_manager: &LayoutManager) -> Self {
        let node = layout_manager.create_node(&style);

        Self {
            style, 
            node,
            pos: Vector2::ZERO,
            size: Vector2::ZERO,
        }
    }
}

impl ScrollableItem for GenericNode {
    fn get_style(&self) -> Style { self.style.clone() }
    fn apply_layout(&mut self, layout: &LayoutManager, parent_pos: Vector2) {
        let layout = layout.get_layout(self.node);
        self.pos = layout.location.into();
        self.pos += parent_pos;
        self.size = layout.size.into();
    }

    fn draw(&mut self, _pos_offset:Vector2, _list: &mut RenderableCollection) {}
}