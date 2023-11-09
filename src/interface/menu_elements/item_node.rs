use crate::prelude::*;


/// a node that contains elements but doesnt do anything on its own (think of this like a <div> element)
#[derive(ScrollableGettersSetters)]
pub struct ItemNode {
    pos: Vector2,
    size: Vector2,

    style: Style,
    node: Node,

    items: Vec<Box<dyn ScrollableItem>>,
    layout_manager: LayoutManager,

    /// do we want to draw a rectangle for this group? if so, what color and border to use
    pub draw_rect: Option<(Color, Option<Border>)>,
}
impl ItemNode {
    pub fn new(style: Style, layout_manager: &LayoutManager) -> Self {
        let node = layout_manager.create_node(&style);
        let layout_manager = layout_manager.clone().with_parent(node);

        Self {
            pos: Vector2::ZERO,
            size: Vector2::ZERO,
            style, 
            node,

            items: Vec::new(),
            layout_manager,

            draw_rect: None,
        }
    }

    pub fn add_item(&mut self, item: impl ScrollableItem + 'static) {
        let item = Box::new(item);
        self.items.push(item);
    }
}

impl ScrollableItem for ItemNode {
    fn get_style(&self) -> Style { self.style.clone() }
    fn apply_layout(&mut self, layout: &LayoutManager, parent_pos: Vector2) {
        let layout = layout.get_layout(self.node);
        self.pos = layout.location.into();
        self.pos += parent_pos;
        self.size = layout.size.into();

        self.items.iter_mut().for_each(|i|i.apply_layout(&self.layout_manager, self.pos));
    }

    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
        self.items.iter_mut().for_each(|i|i.draw(pos_offset, list));
    }
}