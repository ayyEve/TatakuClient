use crate::prelude::*;
use crate::prelude::ui::*;


pub struct CurrentInputState {
    pub mouse_pos: Vector2,
    pub mouse_moved: bool,
    pub scroll_delta: f32,

    pub mouse_down: Vec<MouseButton>,
    pub mouse_up: Vec<MouseButton>,

    pub keys_down: KeyCollection,
    pub keys_up: KeyCollection,

    pub controller_down: Vec<(ControllerButton, GamepadId, Arc<String>)>,
    pub controller_up: Vec<(ControllerButton, GamepadId, Arc<String>)>,
    pub controller_axes: Vec<(Axis, f32, GamepadId, Arc<String>)>,

    pub mods: KeyModifiers,
}
impl CurrentInputState {
    fn make_input(&self, event: InputType) -> InputEvent {
        InputEvent {
            event,
            mouse_pos: self.mouse_pos,
            key_mods: self.mods,
        }
    }

    pub fn into_events(self) -> Vec<InputEvent> {
        [
            self.mouse_moved.then_some(InputType::MouseMove(self.mouse_pos)),
            (self.scroll_delta > f32::EPSILON).then_some(InputType::MouseScroll(self.scroll_delta))
        ]
            .into_iter()
            .flatten()
            .chain(self.mouse_down.into_iter().map(InputType::MousePress))
            .chain(self.mouse_up.into_iter().map(InputType::MouseRelease))
            .chain(self.keys_down.0.into_iter().map(InputType::KeyPress))
            .chain(self.keys_up.0.into_iter().map(InputType::KeyRelease))
            
            .chain(self.controller_down.into_iter().map(|(a, b, c)| InputType::ControllerPress(a, b, c)))
            .chain(self.controller_up.into_iter().map(|(a, b, c)| InputType::ControllerRelease(a, b, c)))
            .chain(self.controller_axes.into_iter().map(|(a, b, c, d)| InputType::ControllerAxis(a, b, c, d)))

            .map(|event| InputEvent { event, mouse_pos: self.mouse_pos, key_mods: self.mods })
            .collect()
    }
}


pub struct GeneralUiTheme {
    pub background_color: Color,
    pub default_color: Color,
    pub hover_color: Color,
    pub active_color: Color,
}
impl GeneralUiTheme {
    pub fn get_color(&self, active: bool, hover: bool) -> Color {
        if active {
            self.active_color
        } else if hover {
            self.hover_color
        } else {
            self.default_color
        }
    }
}
impl Default for GeneralUiTheme {
    fn default() -> Self {
        Self {
            background_color: Color::BLACK.alpha(0.8),
            default_color: Color::WHITE,
            hover_color: Color::CYAN,
            active_color: Color::YELLOW,
        }
    }
}


pub struct Tree {
    tree: TaffyTree<TreeData>,
    pub node: Box<dyn Widget>,
    root: NodeId,
    
    pub bounds: Bounds,
    pub owner: MessageOwner,
    should_refresh: bool,

    /// a list of all children in the tree
    all_children: HashSet<TaffyNodeId>,

    selected_node: SelectedNode,
}
impl Tree {
    pub fn with_capacity(
        cap: usize,
        owner: MessageOwner,
    ) -> Self {
        let mut tree = TaffyTree::with_capacity(cap);
        
        let root = tree.new_leaf(
            Style {
                size: Size {
                    width: Dimension::Percent(1.0),
                    height: Dimension::Percent(1.0),
                },
                ..Default::default()
            }
        ).unwrap();
        tree.set_node_context(root, Some(TreeData::default())).unwrap();

        Self {
            tree,
            node: EmptyWidget::new_boxed(),
            root: NodeId::new(root, owner),
            bounds: Bounds::default(),
            should_refresh: false,

            owner,
            selected_node: SelectedNode::default(),
            all_children: HashSet::new(),
        }
    }

    pub fn has_node(&self, node: NodeId) -> bool {
        node.owner == self.owner
    }

    pub fn set_node(
        &mut self, 
        mut node: Box<dyn Widget>,
        values: &mut dyn Reflect,
    ) {
        // clear the tree and all our children
        self.tree.clear();
        self.all_children.clear();

        // TODO: refresh layout when scale changes
        let ui_scale = values.reflect_get::<f32>("settings.ui_scale").map(|i| *i).unwrap_or(1.0);
        
        // layout the new node
        let mut shell = LayoutShell {
            owner: self.owner,
            tree: self,
            values,
            ui_scale,
        };

        let new = node.layout(&mut shell).expect("failed to layout new node?");
        self.node = node;

        self.root = self.new_with_children(
            Style {
                size: Size {
                    width: Dimension::Percent(1.0),
                    height: Dimension::Percent(1.0),
                    // width: Dimension::Length(self.bounds.size.x),
                    // height: Dimension::Length(self.bounds.size.y),
                },
                ..Default::default()
            }, 
            &[ new ]
        ).unwrap();
        self.all_children.insert(self.root.node_id);
        self.tree.set_node_context(self.root.node_id, Some(TreeData::default())).unwrap();
        
        self.update_layout();
    }

    #[allow(clippy::borrowed_box)]
    pub fn get_node(&self) -> &Box<dyn Widget> { &self.node }
    pub fn mark_refresh(&mut self, _s: &str) {
        self.should_refresh = true
    }


    pub fn update_bounds(
        &mut self, 
        bounds: Bounds,
    ) {
        if bounds == self.bounds { return }
        self.bounds = bounds;
        self.update_layout();
    }

    pub fn update_layout(&mut self) {
        // debug!("{:?} doing layout", self.owner);s
        self.should_refresh = false;
        use taffy::AvailableSpace::*;
        let space = Size {
            width: Definite(self.bounds.size.x),
            height: Definite(self.bounds.size.y),
        };

        // i dont think dirty actually does anything taffy-side
        self.tree.mark_dirty(self.root.node_id).expect("failed to mark dirty?");
        self.tree
            .compute_layout(self.root.node_id, space)
            .expect("failed to compute layout?");

        self.update_contexts();
    }


    pub fn update_contexts(&mut self) {
        // update absolute positions and matrices
        let matrix = Matrix::identity().trans(self.bounds.pos);
        self.recurse_update_context(self.root.node_id, matrix);

        // update spatial navigation
        SpatialNagivation::new(self)
            .run(NavigateConfig::default());
    }

    fn recurse_update_context(&mut self, node: TaffyNodeId, mut matrix: Matrix) {
        let layout = self.tree.layout(node).unwrap();
        let bounds = Bounds::new(
            Vector2::new(
                layout.location.x,
                layout.location.y
            ),
            Vector2::new(
                layout.size.width,
                layout.size.height,
            )
        );

        let context = self.tree.get_node_context_mut(node).unwrap();
        context.absolute_bounds = matrix * bounds;
        context.global_transform = matrix;

        if context.needs_inverse_transform {
            context.inverse_global_transform = context
                .global_transform
                .inverse()
                .unwrap_or_else(|| {
                    eprintln!("could not invert transform: {:#?}", context.global_transform); 
                    context.global_transform 
                })
            ;
        }

        matrix = matrix * context.local_transform.matrix() * Matrix::identity().trans(bounds.pos);
        for child in self.tree.children(node).unwrap() {
            self.recurse_update_context(child, matrix);
        }
    }

    pub fn update_context(&mut self, node: impl HasNodeId) {
        let node = node.get_id();

        let our_matrix = self.get_context(node)
            .map(|p| p.global_transform)
            .unwrap_or_else(|| Matrix::identity().trans(self.bounds.pos));

        self.recurse_update_context(node, our_matrix);
    }
    
    pub fn absolute_bounds(&self, node: impl HasNodeId) -> Option<Bounds> {
        self.get_context(node).map(|i| i.absolute_bounds)
    }

    pub fn content_bounds(&self, node: impl HasNodeId) -> Option<Bounds> {
        let layout = self.get_layout(node)?;
        
        Some(Bounds::new(
            Vector2::new(
                layout.content_box_x(),
                layout.content_box_y(),
            ),
            Vector2::new(
                layout.content_box_width(),
                layout.content_box_height(),
            )
        ))
    }
    pub fn bounds(&self, node: impl HasNodeId) -> Option<Bounds> {
        let layout = self.get_layout(node)?;
        
        Some(Bounds::new(
            Vector2::new(
                layout.location.x,
                layout.location.y,
            ),
            Vector2::new(
                layout.size.width,
                layout.size.height,
            )
        ))
    }

    pub fn all_children(&self) -> impl Iterator<Item = TaffyNodeId> {
        self.all_children.iter().copied().filter(|i| i != &EMPTY_NODE.node_id)
    }



    fn with_node<T>(&mut self, mut f: impl FnMut(&mut Tree, &mut Box<dyn Widget>) -> T + Send + Sync) -> T {
        let mut temp: Box<dyn Widget> = Box::new(EmptyWidget(self.node.node_id()));
        std::mem::swap(&mut self.node, &mut temp);

        let t = f(self, &mut temp);

        self.node = temp;
        t
    }

    pub fn handle_inputs(
        &mut self,
        input_state: &mut CurrentInputState,
        values: &mut dyn Reflect,
        actions: &mut ActionQueue,
        messages: &mut Vec<Message>
    ) {
        // let mouse_pos = input_state.mouse_pos;
        self.with_node(|tree, node| {
            let bounds = tree.bounds;
            let mut shell = InputShell {
                owner: tree.owner,
                messages,
                actions,
                tree,
                values,
                mouse_pos: input_state.mouse_pos,
                event_consumed: false,
            };

            if input_state.mouse_moved {
                let pos = input_state.mouse_pos - bounds.pos;

                node.input(
                    &input_state.make_input(InputType::MouseMove(pos)),
                    &mut shell
                );
            }
            if input_state.scroll_delta.abs() > f32::EPSILON {
                node.input(
                    &input_state.make_input(InputType::MouseScroll(input_state.scroll_delta)),
                    &mut shell
                );
            }

            let mouse_pos = input_state.mouse_pos;
            let key_mods = input_state.mods;

            macro_rules! handle_event {
                ($list: expr, $map: ident) => {
                    $list.retain(|a| {
                        node.input(
                            &InputEvent {
                                event: InputType::$map(a.clone()),
                                key_mods,
                                mouse_pos,
                            },
                            &mut shell
                        );
                        !std::mem::take(&mut shell.event_consumed)
                    });
                }
            }

            handle_event!(input_state.keys_down.0, KeyPress);
            handle_event!(input_state.keys_up.0, KeyRelease);
            handle_event!(input_state.mouse_down, MousePress);
            handle_event!(input_state.mouse_up, MouseRelease);

            input_state.controller_down.retain(|(a, id, name)| {
                node.input(
                    &InputEvent {
                        event: InputType::ControllerPress(*a, *id, name.clone()),
                        key_mods,
                        mouse_pos,
                    },
                    &mut shell
                );
                !std::mem::take(&mut shell.event_consumed)
            });

            input_state.controller_up.retain(|(a, id, name)| {
                node.input(
                    &InputEvent {
                        event: InputType::ControllerRelease(*a, *id, name.clone()),
                        key_mods,
                        mouse_pos,
                    },
                    &mut shell
                );
                !std::mem::take(&mut shell.event_consumed)
            });

            input_state.controller_axes.retain(|(a, value, id, name)| {
                node.input(
                    &InputEvent {
                        event: InputType::ControllerAxis(*a, *value, *id, name.clone()),
                        key_mods,
                        mouse_pos,
                    },
                    &mut shell
                );
                !std::mem::take(&mut shell.event_consumed)
            });
        });

        for (key, direction) in [
            (Key::Left, Direction::Left),
            (Key::Right, Direction::Right),
            (Key::Up, Direction::Up),
            (Key::Down, Direction::Down),
            (Key::Tab, Direction::Down),
        ] {
            if !input_state.keys_down.has_key(key) { continue }

            if !self.selected_node.active {
                self.enable_navigation();
                input_state.keys_down.remove_key(key);
                // return since this was just to enable navigation
                // otherwise we'd immediate select the next node, without selecting the current node
                break;
            }

            let Some(current) = self.selected_node.node else { 
                warn!("No active node to navigate from ??");
                break
            };

            if let Some(node) = self.tree.get_node_context(current.node_id).and_then(|i| i.node_direction(direction)) {
                self.context_mut(current).selected = Some(false);
                self.context_mut(node).selected = Some(true);
                input_state.keys_down.remove_key(key);
            }

            break
        }
    }

    fn enable_navigation(&mut self) {
        self.selected_node.active = true;

        // try to make sure we have a selected node to start with
        if self.selected_node.node.is_none() {
            // find the first selectable node
            self.selected_node.node = self.find_child(self.root, Rc::new(|tree, node| {
                tree.context(node).selectable()
            }));

            if let Some(node) = self.selected_node.node {
                self.context_mut(node).selected = Some(true)
            }
        }

    }

    // helpers for when we're certain the node is in the tree
    // private for that reason too
    fn context(&self, node: impl HasNodeId) -> &TreeData {
        self.get_context(node.get_id()).unwrap()
    }
    fn context_mut(&mut self, node: impl HasNodeId) -> &mut TreeData {
        self.get_context_mut(node.get_id()).unwrap()
    }

    /// this isnt the most efficient thing ever but hopefully its not used too often
    fn find_child(&self, parent: impl HasNodeId, f: Rc<dyn Fn(&Self, TaffyNodeId) -> bool>) -> Option<NodeId> {
        let parent = parent.get_id();
        if f(self, parent) { return Some(NodeId::new(parent, self.owner)) }
        for child in self.tree.children(parent).ok()? {
            if let Some(node) = self.find_child(child, f.clone()) { 
                return Some(node) 
            }
        }
        None
    }


    pub fn update(
        &mut self,
        values: &mut dyn Reflect,
        actions: &mut ActionQueue,
        messages: &mut Vec<Message>,
    ) {
        if self.should_refresh {
            self.update_layout();
        }

        let mut node: Box<dyn Widget> = Box::new(EmptyWidget(self.node.node_id()));
        std::mem::swap(&mut self.node, &mut node);

        // update the root widget
        let mut shell = UpdateShell {
            owner: self.owner,
            tree: self,
            values,
            messages,
        };
        node.update(&mut shell, actions);

        self.node = node;
    }


    pub fn draw(&mut self, list: &mut RenderableCollection) {
        self.with_node(|tree, node| {
            let mut shell = DrawShell {
                tree,
                list,
                // TODO: make customizable
                general_theme: GeneralUiTheme::default(),
            };
            node.draw(&mut shell);
        });
    }


    // TaffyTree things
    pub fn new_leaf(&mut self, style: Style) -> TaffyResult<NodeId> {
        let id = self.tree.new_leaf(style)?;

        if self.all_children.insert(id) {
            self.tree.set_node_context(id, Some(TreeData::default()))?;
        }

        let id = NodeId::new(id, self.owner);
        Ok(id)
    }

    pub fn new_with_children(&mut self, style: Style, children: &[NodeId]) -> TaffyResult<NodeId> {
        let id = self.new_leaf(style)?;
        let children = children.iter().map(|i| i.node_id).collect::<Vec<_>>();
        self.tree.set_children(id.node_id, &children)?;
        Ok(id)
    }

    pub fn add_child(&mut self, parent: impl HasNodeId, child: impl HasNodeId) {
        let _ = self.tree.add_child(parent.get_id(), child.get_id());
    }

    pub fn remove(&mut self, node: impl HasNodeId) {
        let id = node.get_id();
        let _ = self.tree.remove(id);
        self.all_children.remove(&id);
    }

    pub fn get_layout(&self, node: impl HasNodeId) -> Option<&Layout> {
        self.tree.layout(node.get_id()).ok()
    }

    pub fn parent(&self, node: impl HasNodeId) -> Option<NodeId> {
        self.tree.parent(node.get_id())
            .map(|i| NodeId::new(i, self.owner))
    }


    pub fn get_context(&self, node: impl HasNodeId) -> Option<&TreeData> {
        self.tree.get_node_context(node.get_id())
    }
    pub fn get_context_mut(&mut self, node: impl HasNodeId) -> Option<&mut TreeData> {
        self.tree.get_node_context_mut(node.get_id())
    }

    pub fn mark_dirty(&mut self, node: impl HasNodeId) {
        let _ = self.tree.mark_dirty(node.get_id());
    }


    pub fn get_style(&self, node: impl HasNodeId) -> Option<&Style> {
        self.tree.style(node.get_id()).ok()
    }
    pub fn set_style(&mut self, node: impl HasNodeId, style: Style) {
        let _ = self.tree.set_style(node.get_id(), style);
    }
}


#[derive(Copy, Clone)]
pub struct TreeData {
    // pub bounds: Bounds,
    pub absolute_bounds: Bounds,
    pub local_transform: Transform,
    pub global_transform: Matrix,
    pub inverse_global_transform: Matrix,
    pub needs_inverse_transform: bool,

    pub selected: Option<bool>,
    pub node_left: Option<TaffyNodeId>,
    pub node_right: Option<TaffyNodeId>,
    pub node_above: Option<TaffyNodeId>,
    pub node_below: Option<TaffyNodeId>,
}
impl Default for TreeData {
    fn default() -> Self {
        Self { 
            // bounds: Bounds::default(),
            absolute_bounds: Bounds::default(), 
            local_transform: Transform::default(), 
            global_transform: Matrix::identity(), 
            inverse_global_transform: Matrix::identity(), 
            needs_inverse_transform: false,

            selected: None, 
            node_left: None, 
            node_right: None, 
            node_above: None, 
            node_below: None, 
        }
    }
}
impl TreeData {
    pub fn selectable(&self) -> bool {
        self.selected.is_some()
    }
    pub fn set_selectable(&mut self, selectable: bool) {
        self.selected = selectable.then_some(false);
    }

    pub fn node_direction(&self, direction: Direction) -> Option<TaffyNodeId> {
        match direction {
            Direction::Up => self.node_above,
            Direction::Down => self.node_below,
            Direction::Left => self.node_left,
            Direction::Right => self.node_right,
        }
    }
}


#[derive(Copy, Clone, Debug, Default)]
struct SelectedNode {
    node: Option<NodeId>,
    active: bool,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct NodeId {
    pub node_id: TaffyNodeId,
    pub owner: MessageOwner,
}
impl NodeId {
    pub fn new(id: TaffyNodeId, owner: MessageOwner) -> Self {
        Self {
            node_id: id,
            owner,
        }
    }
}
impl Default for NodeId {
    fn default() -> Self { EMPTY_NODE }
}


#[derive(Clone, Debug)]
pub enum MenuType {
    Internal(&'static str),
    Custom(String)
}
#[cfg(feature="graphics")]
impl MenuType {
    pub fn from_menu(menu: &dyn crate::prelude::Widget) -> Self {
        match menu.name() {
            Cow::Borrowed(name) => Self::Internal(name),
            Cow::Owned(name) => Self::Custom(name.clone())
        }
    }
}
