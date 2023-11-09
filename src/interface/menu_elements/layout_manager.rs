use taffy::Taffy;
use taffy::prelude::Rect;
use taffy::layout::Layout;
use crate::prelude::*;

#[derive(Clone)]
pub struct LayoutManager {
    taffy: Arc<Mutex<Taffy>>,
    parent_nodes: Vec<Node>,
    node: Node,
}
impl LayoutManager {
    pub fn new() -> Self {
        let mut taffy = Taffy::new();
        let parent_node = taffy.new_leaf(Style {
            size: Self::full_size(),
            ..Default::default()
        }).unwrap();

        Self {
            taffy: Arc::new(Mutex::new(taffy)),
            node: parent_node,
            parent_nodes: Vec::new(),
        }
    }
    pub fn with_parent(mut self, parent: Node) -> Self {
        self.parent_nodes.push(self.node);
        self.node = parent;

        self
    }

    pub fn clear(&self) {
        self.taffy.lock().set_children(self.node, &[]).expect("nah");
    }

    pub fn create_node(&self, layout: &Style) -> Node {
        let mut taffy = self.taffy.lock();
        let new_node = taffy.new_leaf(layout.clone()).unwrap();
        taffy.add_child(self.node, new_node).expect("it didnt like that");
        new_node
    }

    pub fn set_style(&self, style: Style) {
        self.taffy.lock().set_style(self.node, style).expect("nope");
    }
    pub fn set_child_style(&self, child: Node, style: Style) {
        self.taffy.lock().set_style(child, style).expect("nope");
    }

    // fn location(&self) -> Vector2 {
    //     let lock = self.taffy.lock();
    //     self.parent_nodes.iter().fold(Vector2::ZERO, |i, n| {
    //         let p: Vector2 = lock.layout(*n).unwrap().location.into();
    //         i + p
    //     })
    // }
    pub fn apply_layout(&mut self, container_size: Vector2) {
        // info!("{:?}", self.parent_nodes);

        let available_space = Size {
            width: taffy::style::AvailableSpace::Definite(container_size.x),
            height: taffy::style::AvailableSpace::Definite(container_size.y),
        };

        let mut taffy = self.taffy.lock();
        taffy.compute_layout(self.node, available_space).expect("failed to perform layout");
    }

    pub fn needs_refresh(&self) -> bool {
        self.taffy.lock().dirty(self.node).unwrap_or_default()
    }
    // fn parent_layout(&self) -> Layout {
    //     self.taffy.lock().layout(self.parent_node).unwrap().clone()
    // }

    pub fn get_layout(&self, node: Node) -> Layout {
        let mut layout = self.taffy.lock().layout(node).unwrap().clone();

        // layout.location.y = layout.location.y - layout.size.y;
        // let parent = self.parent_layout();
        // let location2 = self.location();
        // layout.location.x += parent.location.x + location2.x;
        // layout.location.y += parent.location.y + location2.y;
        layout
    }

    pub fn get_own_layout(&self) -> Layout {
        self.get_layout(self.node)
    }


    pub fn get_pos_size(style: &Style) -> (Vector2, Vector2) {
        let size = style.size.into();
        let pos = Vector2::new(
            style.inset.left.resolve_to_option(0.0).unwrap_or_default(),
            style.inset.top.resolve_to_option(0.0).unwrap_or_default(),
        );

        (pos, size)
    }

    pub fn style_from_size_pos(pos: Vector2, size: Vector2) -> Style {
        Style {
            inset: Rect {
                top: taffy::style::LengthPercentageAuto::Points(pos.y),
                left: taffy::style::LengthPercentageAuto::Points(pos.x),
                right: taffy::style::LengthPercentageAuto::Auto,
                bottom: taffy::style::LengthPercentageAuto::Auto,
            },
            size: taffy::geometry::Size {
                width: taffy::style::Dimension::Points(size.x),
                height: taffy::style::Dimension::Points(size.y),
            },

            ..Default::default()
        }
    }

    pub const fn full_size() -> Size<Dimension> {
        Size {
            width: Dimension::Percent(1.0),
            height: Dimension::Percent(1.0),
        }
    }

    pub const fn full_width() -> Size<Dimension> {
        Size {
            width: Dimension::Percent(1.0),
            height: Dimension::Auto,
        }
    }

    pub const fn small_button() -> Size<Dimension> {
        Size {
            width: Dimension::Percent(0.05),
            height: Dimension::Auto,
        }
    }
    pub const fn long_button() -> Size<Dimension> {
        Size {
            width: Dimension::Percent(0.1),
            height: Dimension::Auto,
        }
    }
}
