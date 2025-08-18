use std::rc::Rc;
use super::NodeData;
use crate::prelude::*;
use tataku_input::prelude::*;
use tataku_client_common::prelude::*;

use slotmap::SlotMap;
use slotmap::DefaultKey;
use slotmap::SparseSecondaryMap;

pub struct Tree<Action: Send + Sync> {
    // tree: TaffyTree<TreeData>,

    /// The [`NodeData`] for each node stored in this tree
    pub(super) nodes: SlotMap<DefaultKey, NodeData>,

    /// Functions/closures that compute the intrinsic size of leaf nodes
    pub(super) node_context_data: SparseSecondaryMap<DefaultKey, TreeData>,

    /// The children of each node
    ///
    /// The indexes in the outer vector correspond to the position of the parent [`NodeData`]
    pub(super) children: SlotMap<DefaultKey, Vec<taffy::NodeId>>,

    /// The parents of each node
    ///
    /// The indexes in the outer vector correspond to the position of the child [`NodeData`]
    pub(super) parents: SlotMap<DefaultKey, Option<taffy::NodeId>>,

    pub node: Box<dyn Widget<Action>>,
    root: NodeId,
    
    pub bounds: Bounds,
    pub owner: MessageOwner,
    should_refresh: bool,
    selected_node: SelectedNode,

    use_rounding: bool,
}
impl<Action: Send + Sync + 'static> Tree<Action> {
    pub fn new(
        capacity: usize,
        owner: MessageOwner,
        node: Box<dyn Widget<Action>>,
    ) -> Self {
        let mut nodes = SlotMap::with_capacity(capacity);
        let root = nodes.insert(NodeData::new());

        let mut node_context_data = SparseSecondaryMap::with_capacity(capacity);
        node_context_data.insert(root, TreeData::with_style(CssStyle::menu_layout()));

        let mut children = SlotMap::with_capacity(capacity);
        children.insert(Vec::new());

        let mut parents = SlotMap::with_capacity(capacity);
        parents.insert(None);

        Self {
            nodes,
            node_context_data,
            children,
            parents,

            node,
            root: NodeId::new(root.into(), owner),
            bounds: Bounds::default(),
            should_refresh: false,

            owner,
            selected_node: SelectedNode::default(),

            use_rounding: true,
        }
    }

    pub fn has_node(&self, node: NodeId) -> bool {
        node.owner == self.owner
    }

    #[allow(clippy::borrowed_box)]
    pub fn get_node(&self) -> &Box<dyn Widget<Action>> { &self.node }

    pub fn set_node(
        &mut self, 
        mut node: Box<dyn Widget<Action>>,
        values: &mut dyn Reflect,
    ) {
        // clear the tree and all our children
        self.clear();

        // TODO: refresh layout when scale changes
        let ui_scale = values.reflect_get::<f32>("settings.ui_scale")
            .map(|i| *i)
            .unwrap_or(1.0);

        let style = node.get_style_str();
        let mut resolver = CssResolver::new(&style);
        
        // layout the new node
        let mut shell = LayoutShell {
            owner: self.owner,
            tree: self,
            values,
            ui_scale,
            resolver: &mut resolver,
        };

        let root = node
            .layout(&mut shell)
            .expect("failed to layout new node?");
        shell.tree.root = root;

        node.init_style(&mut shell);
        shell.tree.update_style(root, |s| *s = CssStyle::menu_layout());

        self.node = node;
        self.update_layout(values);
    }

    
    pub fn mark_refresh(&mut self, _s: &str) {
        self.should_refresh = true;
    }

    pub fn set_display(
        &mut self, 
        node: impl HasNodeId, 
        display: Option<DisplayType>
    ) {
        let node = node.get_id();
        let Some(data) = self.nodes.get_mut(node.into()) 
        else { return };

        if data.current_display == display { return }
        data.current_display = display;

        self.mark_dirty(node);
    }

    pub fn update_bounds(
        &mut self, 
        bounds: Bounds,
        values: &dyn Reflect,
    ) {
        if bounds == self.bounds { return }
        self.bounds = bounds;
        self.update_layout(values);
    }

    pub fn update_layout(&mut self, values: &dyn Reflect) {
        // debug!("{:?} doing layout", self.owner);
        self.should_refresh = false;
        use taffy::AvailableSpace::*;
        let space = taffy::Size {
            width: Definite(self.bounds.size.x),
            height: Definite(self.bounds.size.y),
        };

        let root: taffy::NodeId = self.root.node_id;
        super::LayoutTree {
            viewport: self.bounds.size,
            use_rounding: self.use_rounding,
            root_font_size: self.root_font_size(values),
            values,
            tree: self,
        }.compute_layout(root, space);

        self.update_contexts();
    }


    pub fn update_contexts(&mut self) {
        // update absolute positions and matrices
        let matrix = Matrix::identity().trans(self.bounds.pos);
        self.recurse_update_context(self.root.node_id, matrix);

        // update spatial navigation
        SpatialNagivation::new(self)
            .run(&NavigateConfig::default());
    }

    fn recurse_update_context(
        &mut self, 
        node: taffy::NodeId, 
        mut matrix: Matrix
    ) {
        if !self.nodes.contains_key(node.into()) {
            error!("node not in tree!!! {node:?}");
            return;
        }

        let layout = self.get_layout(node).unwrap();
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

        let context = self.context_mut(node);
        context.absolute_bounds = matrix * bounds;
        context.global_transform = matrix;

        #[allow(clippy::unnecessary_lazy_evaluations, reason = "comment")]
        if context.needs_inverse_transform {
            context.inverse_global_transform = context
                .global_transform
                .inverse()
                .unwrap_or_else(|| {
                    // eprintln!("could not invert transform: {:#?}", context.global_transform); 
                    context.global_transform 
                })
            ;
        }

        matrix = matrix 
            * context.local_transform.matrix()
            * Matrix::identity().trans(bounds.pos);

        for child in self.children(node) {
            self.recurse_update_context(child.node_id, matrix);
        }
    }

    pub fn update_context(&mut self, node: impl HasNodeId) {
        let node = node.get_id();

        let our_matrix = self
            .get_context(node)
            .map_or_else(
                || Matrix::identity().trans(self.bounds.pos), 
                |p| p.global_transform
            );

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

    pub fn all_children(&self) -> impl Iterator<Item = taffy::NodeId> {
        self.nodes.keys()
            .map(taffy::NodeId::from)
            .filter(|i| i != &EMPTY_NODE.node_id)
    }



    fn with_node<T>(
        &mut self, 
        f: impl FnOnce(&mut Tree<Action>, &mut Box<dyn Widget<Action>>) -> T
    ) -> T {
        let mut temp = SwapTree::new(&*self.node);
        // let mut temp: Box<dyn Widget> = Box::new(EmptyWidget(self.node.node_id()));
        std::mem::swap(&mut self.node, &mut temp);

        let t = f(self, &mut temp);

        self.node = temp;
        t
    }

    pub fn handle_inputs(
        &mut self,
        input_state: &mut CurrentInputState,
        values: &mut dyn Reflect,
        actions: &mut Queue<Action>,
        messages: &mut Vec<Message>,
    ) -> bool {
        let consumed = self.with_node(|tree, node| {
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
                node.input(
                    &input_state
                        .make_input(InputType::MouseMove(input_state.mouse_pos)),
                    &mut shell
                );
            }
            if input_state.scroll_delta.x.abs() > f32::EPSILON || input_state.scroll_delta.y.abs() > f32::EPSILON{
                node.input(
                    &input_state
                        .make_input(InputType::MouseScroll(input_state.scroll_delta)),
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

            input_state.controller_down
                .retain(|(a, id, name)| {
                node.input(
                    &InputEvent {
                        event: InputType::ControllerPress(*a, *id, name.clone()),
                        key_mods,
                        mouse_pos,
                    },
                    &mut shell
                );
                !shell.event_consumed.take()
            });

            input_state.controller_up
                .retain(|(a, id, name)| {
                node.input(
                    &InputEvent {
                        event: InputType::ControllerRelease(*a, *id, name.clone()),
                        key_mods,
                        mouse_pos,
                    },
                    &mut shell
                );
                !shell.event_consumed.take()
            });

            input_state.controller_axes
                .retain(|(a, value, id, name)| {
                node.input(
                    &InputEvent {
                        event: InputType::ControllerAxis(*a, *value, *id, name.clone()),
                        key_mods,
                        mouse_pos,
                    },
                    &mut shell
                );
                shell.event_consumed.take()
            });

            shell.event_consumed
        });

        if consumed { return true }

        // self.handle_spacial_navigation(input_state, values, actions, messages)
        false
    }

    pub fn get_style(&self, node: impl HasNodeId) -> Option<&CssStyle> {
        Some(
            &self.node_context_data
            .get(node.get_id().into())?
            .current_style().0
        )
    }
    pub fn get_text_style(&self, node: impl HasNodeId) -> Option<&TextStyle> {
        Some(
            self.node_context_data
            .get(node.get_id().into())?
            .current_text_style()
        )
    }

    // widget things

    pub fn handle_message(
        &mut self,
        message: &Message,
        values: &mut dyn Reflect,
        actions: &mut Queue<Action>,
        messages: &mut Vec<Message>,
    ) {
        self.with_node(|tree, node| {
            let mut shell = MessageShell {
                values,
                actions,
                messages,
                owner: tree.owner,
                tree,
                handled: false
            };
            node.handle_message(message, &mut shell);
        });
    }

    pub fn handle_event(
        &mut self,
        event: &TatakuEventType,
        passed_in: Option<&TatakuValue>,
        values: &mut dyn Reflect,
        actions: &mut Queue<Action>,
        messages: &mut Vec<Message>,
    ) {
        self.with_node(|tree, node| {
            let mut shell = MessageShell {
                values,
                actions,
                messages,
                owner: tree.owner,
                tree,
                handled: false
            };
            node.handle_event(
                event, 
                passed_in, 
                &mut shell
            );
        });
    }

    pub fn update(
        &mut self,
        values: &mut dyn Reflect,
        actions: &mut Queue<Action>,
        messages: &mut Vec<Message>,
        skin_manager: &mut dyn SkinProvider,
    ) {
        if self.should_refresh {
            self.update_layout(values);
        }

        // update the root widget
        self.with_node(|tree, node| {
            let mut shell = UpdateShell {
                owner: tree.owner,
                tree,
                values,
                actions,
                messages,
                skin_manager,
            };
            node.update(&mut shell);
        });
    }

    pub fn reload_skin(
        &mut self,
        values: &mut dyn Reflect,
        messages: &mut Vec<Message>,
        actions: &mut Queue<Action>,
        skin_manager: &mut dyn SkinProvider,
    ) {
        self.with_node(|tree, node| {
            // update the root widget
            let mut shell = UpdateShell {
                owner: tree.owner,
                tree,
                values,
                actions,
                messages,
                skin_manager,
            };
            node.reload_skin(&mut shell);
        });
    }

    pub fn draw(
        &mut self, 
        values: &dyn Reflect,
        list: &mut RenderableCollection,
    ) {
        self.with_node(|tree, node| {
            let mut shell = DrawShell {
                tree,
                list,
                values,
                // TODO: make customizable
                general_theme: GeneralUiTheme::default(),
            };
            node.draw(&mut shell);
            node.draw_overlay(&mut shell);
        });
    }

}

// spacial navigation things
#[allow(unused, reason = "spacial navigation currently disabled")]
impl<Action: Send + Sync + 'static> Tree<Action> {

    fn handle_spacial_navigation(
        &mut self,
        input_state: &mut CurrentInputState,
        values: &mut dyn Reflect,
        // actions: &mut ActionQueue,
        messages: &mut [Message],
    ) -> bool {
        let mut consumed = false;

        #[derive(Copy, Clone)]
        #[derive(From)]
        enum MenuInputType {
            Key(Key),
            Controller(ControllerButton),
            // Axis()
        }
        impl MenuInputType {
            fn has(
                self, 
                state: &mut CurrentInputState
            ) -> bool {
                match self {
                    Self::Key(key) 
                        => state.keys_down.has_key(key),
                    Self::Controller(btn) 
                        => state.controller_down.iter()
                            .any(|(b, _, _)| b == &btn),
                }
            }
            fn remove_from(
                self,
                state: &mut CurrentInputState,
            ) {
                match self {
                    Self::Key(key) 
                        => state.keys_down.remove_key(key),
                    Self::Controller(btn) 
                        => state.controller_down
                            .retain(|(b, _, _)| b != &btn)
                }
            }
        }

        for (input, direction) in [
            (MenuInputType::Key(Key::Left), Direction::Left),
            (Key::Right.into(), Direction::Right),
            (Key::Up.into(), Direction::Up),
            (Key::Down.into(), Direction::Down),
            (Key::Tab.into(), Direction::Down),

            (ControllerButton::DPadLeft.into(), Direction::Left),
            (ControllerButton::DPadRight.into(), Direction::Right),
            (ControllerButton::DPadUp.into(), Direction::Up),
            (ControllerButton::DPadDown.into(), Direction::Down),
        ] {
            if !input.has(input_state) { continue }
            
            if !self.selected_node.active {
                self.enable_navigation();
                
                input.remove_from(input_state);

                consumed = true;
                error!("Navigation Enabled");
                // return since this was just to enable navigation
                // otherwise we'd immediate select the next node, without selecting the current node
                break;
            }

            let Some(current) = self.selected_node.node else { 
                warn!("No active node to navigate from ??");
                break
            };

            if let Some(node) = self.get_context(current.node_id)
                .and_then(|i| i.node_direction(direction)) 
            {
                self.context_mut(current).selected = Some(false);
                self.context_mut(node).selected = Some(true);
                input.remove_from(input_state);
                consumed = true;
                error!("Navigated!");
            } else {
                error!("No Navigation!!");
            }

            break
        }

        consumed
    }

    fn enable_navigation(&mut self) {
        self.selected_node.active = true;

        // try to make sure we have a selected node to start with
        if self.selected_node.node.is_none() {
            // find the first selectable node
            self.selected_node.node = self.find_child(
                self.root, 
                Rc::new(|tree, node| tree.context(node).selectable())
            );

            if let Some(node) = self.selected_node.node {
                self.context_mut(node).selected = Some(true);
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
    fn find_child(
        &self, 
        parent: impl HasNodeId, 
        f: Rc<dyn Fn(&Self, taffy::NodeId) -> bool>,
    ) -> Option<NodeId> {
        let parent = parent.get_id();
        if f(self, parent) { 
            return Some(NodeId::new(parent, self.owner)) 
        }
        
        for child in self.children(parent) {
            if let Some(node) = self.find_child(child, f.clone()) { 
                return Some(node) 
            }
        }

        None
    }

}

// taffy tree things
impl<Action: Send + Sync + 'static> Tree<Action> {
    pub fn new_leaf(&mut self) -> taffy::TaffyResult<NodeId> {
        let id = self.nodes.insert(NodeData::new());
        self.node_context_data.insert(id, TreeData::default());
        let _ = self.children.insert(Vec::new());
        let _ = self.parents.insert(None);

        Ok(NodeId::new(id.into(), self.owner))
    }

    fn set_children(
        &mut self, 
        parent: taffy::NodeId, 
        children: &[taffy::NodeId]
    ) -> taffy::TaffyResult<()> {
        let parent_key = parent.into();

        // Remove node as parent from all its current children.
        for child in &self.children[parent_key] {
            self.parents[(*child).into()] = None;
        }

        // Build up relation node <-> child
        for &child in children {
            // Remove child from previous parent
            if let Some(previous_parent) = self.parents[child.into()] {
                self.remove_child(previous_parent, child).unwrap();
            }
            self.parents[child.into()] = Some(parent);
        }

        let parent_children = &mut self.children[parent_key];
        parent_children.clear();
        children.iter().for_each(|child| parent_children.push(*child));

        self.mark_dirty(parent);

        Ok(())
    }
    fn remove_child(
        &mut self, 
        parent: taffy::NodeId, 
        child: taffy::NodeId
    ) -> taffy::TaffyResult<taffy::NodeId> {
        let index = self
            .children[parent.into()]
            .iter()
            .position(|n| *n == child)
            .unwrap();
        self.remove_child_at_index(parent, index)
    }
    fn remove_child_at_index(
        &mut self, 
        parent: taffy::NodeId, 
        child_index: usize
    ) -> taffy::TaffyResult<taffy::NodeId> {
        let parent_key = parent.into();
        let child_count = self.children
            .get(parent_key)
            .ok_or(taffy::TaffyError::InvalidParentNode(parent))?
            .len();
        if child_index >= child_count {
            return Err(taffy::TaffyError::ChildIndexOutOfBounds { 
                parent, 
                child_index, 
                child_count 
            });
        }

        let child = self.children[parent_key].remove(child_index);
        self.parents[child.into()] = None;

        self.mark_dirty(parent);

        Ok(child)
    }

    pub fn new_with_children(
        &mut self, 
        children: &[NodeId]
    ) -> taffy::TaffyResult<NodeId> {
        let id = self.new_leaf()?;
        let children = children
            .iter()
            .map(|i| i.node_id)
            .collect::<Vec<_>>();
        
        self.set_children(id.node_id, &children)?;
        Ok(id)
    }

    pub fn add_child(
        &mut self, 
        parent: impl HasNodeId, 
        child: impl HasNodeId,
    ) {
        let parent = parent.get_id();
        let child = child.get_id();

        let parent_key = parent.into();
        let child_key = child.into();
        self.parents[child_key] = Some(parent);
        self.children[parent_key].push(child);
        self.mark_dirty(parent);
        self.mark_refresh("add_child");
    }

    pub fn remove(&mut self, node: impl HasNodeId) {
        let id = node.get_id();
        let key = id.into();
        if let Some(parent) = self.parents[key] {
            if let Some(children) = self.children.get_mut(parent.into()) {
                children.retain(|f| *f != id);
            }
        }

        // Remove "parent" references to a node when removing that node
        if let Some(children) = self.children.get(key) {
            for child in children.iter().copied() {
                self.parents[child.into()] = None;
            }
        }

        let _ = self.children.remove(key);
        let _ = self.parents.remove(key);
        let _ = self.nodes.remove(key);
    }

    fn clear(&mut self) {
        self.nodes.clear();
        self.children.clear();
        self.parents.clear();
        self.node_context_data.clear();
    }

    pub fn get_layout(&self, node: impl HasNodeId) -> Option<&taffy::Layout> {
        let node = node.get_id();

        if self.use_rounding {
            Some(&self.nodes.get(node.into())?.final_layout)
        } else {
            Some(&self.nodes.get(node.into())?.unrounded_layout)
        }
    }

    pub fn parent(&self, node: impl HasNodeId) -> Option<NodeId> {
        self.parents
            .get(node.get_id().into())?
            .map(|i| NodeId::new(i, self.owner))
    }
    pub fn children(&self, parent: impl HasNodeId) -> Vec<NodeId> {
        let Some(children) = self.children
            .get(parent.get_id().into())
        else { return Vec::new() };

        children.iter()
            .map(|i| NodeId::new(*i, self.owner))
            .collect()
    }


    pub fn get_context(&self, node: impl HasNodeId) -> Option<&TreeData> {
        self.node_context_data.get(node.get_id().into())
    }
    pub fn get_context_mut(&mut self, node: impl HasNodeId) -> Option<&mut TreeData> {
        self.node_context_data.get_mut(node.get_id().into())
    }

    pub fn mark_dirty(&mut self, node: impl HasNodeId) {
        fn mark_dirty_recursive(
            nodes: &mut SlotMap<DefaultKey, NodeData>,
            parents: &SlotMap<DefaultKey, Option<taffy::NodeId>>,
            node_key: DefaultKey,
        ) {
            match nodes[node_key].cache.clear() {
                taffy::ClearState::AlreadyEmpty => {
                    // Node was already marked as dirty.
                    // No need to visit ancestors
                    // as they should be marked as dirty already.
                }
                taffy::ClearState::Cleared => {
                    if let Some(Some(node)) = parents.get(node_key) {
                        mark_dirty_recursive(nodes, parents, (*node).into());
                    }
                }
            }
        }

        mark_dirty_recursive(
            &mut self.nodes, 
            &self.parents, 
            node.get_id().into()
        );
    }


    pub fn update_style(&mut self, node: impl HasNodeId, f: impl Fn(&mut CssStyle)) {
        let node = node.get_id();

        let Some(data) = self
            .node_context_data
            .get_mut(node.into())
        else { return };

        data.element_data.styles
            .all_mut()
            .into_iter()
            .for_each(|(i,_)| f(i));

        self.mark_dirty(node);
        self.mark_refresh("update_style");
    }

    fn root_font_size(&self, values: &dyn Reflect) -> f32 {
        self.node_context_data
            .get(self.root.node_id.into())
            .unwrap()
            .current_style().0
            .font_size
            .resolve_copied(values)
            .unwrap_or(32.0)
    }
    pub fn print(&mut self, values: &dyn Reflect) {
        let root = self.root.node_id;
        super::LayoutTree {
            values,
            viewport: self.bounds.size,
            root_font_size: self.root_font_size(values),
            use_rounding: self.use_rounding,
            tree: self,
        }.print(root);
    }
}

#[derive(Copy, Clone, Debug, Default)]
struct SelectedNode {
    node: Option<NodeId>,
    active: bool,
}



struct SwapTree<Action> {
    node: NodeId,
    style: ArcStr,
    _a: std::marker::PhantomData<Action>,
}
impl<Action: Send + Sync + 'static> SwapTree<Action> {
    #[allow(clippy::new_ret_no_self, reason = "its the only time its used")]
    fn new(root: &dyn Widget<Action>) -> Box<dyn Widget<Action>> {
        Box::new(Self {
            node: root.node_id(),
            style: root.get_style_str(),
            _a: std::marker::PhantomData,
        })
    }
}
impl<Action: Send + Sync + 'static> Widget<Action> for SwapTree<Action> {
    fn name(&self) -> CowStr { "SWAP TEMP".into() }
    fn node_id(&self) -> NodeId { self.node }
    fn get_style_str(&self) -> ArcStr { self.style.clone() }
    fn layout(&mut self, _: &mut LayoutShell<Action>) -> taffy::TaffyResult<NodeId> {
        unimplemented!()
    }
}
