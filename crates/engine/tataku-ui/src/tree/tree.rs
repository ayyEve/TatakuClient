use crate::*;
use crate::tree::*;
use crate::style::*;
use crate::style::css::*;
use super::LayoutData;
use crate::widget::*;
use crate::message::*;
use common::reflect::*;
use crate::spatial_navigation::*;
use crate::current_input_state::CurrentInputState;

use NodeId;
use slotmap::SlotMap;
use slotmap::DefaultKey;
use slotmap::SparseSecondaryMap;

pub struct Tree<Action: Send + Sync> {
    // tree: TaffyTree<TreeData>,

    /// The [`NodeData`] for each node stored in this tree
    pub(super) nodes: SlotMap<DefaultKey, LayoutData>,

    /// Functions/closures that compute the intrinsic size of leaf nodes
    pub(super) node_context_data: SparseSecondaryMap<DefaultKey, TreeData>,

    /// The children of each node
    ///
    /// The indexes in the outer vector correspond to the position of the parent [`NodeData`]
    pub(super) children: SlotMap<DefaultKey, Vec<NodeId>>,

    /// The parents of each node
    ///
    /// The indexes in the outer vector correspond to the position of the child [`NodeData`]
    pub(super) parents: SlotMap<DefaultKey, Option<NodeId>>,

    pub node: Box<dyn Widget<Action>>,
    root: NodeId,

    pub bounds: Bounds,
    pub source: MessageSource,
    needs_relayout: bool,
    selected_node: SelectedNode,

    use_rounding: bool,
}
impl<Action: Send + Sync + 'static> Tree<Action> {
    pub fn new(
        capacity: usize,
        source: MessageSource,
        node: Box<dyn Widget<Action>>,
    ) -> Self {
        let mut nodes = SlotMap::with_capacity(capacity);
        let root = nodes.insert(LayoutData::new(StyleStack::menu_layout()));

        let mut node_context_data = SparseSecondaryMap::with_capacity(capacity);
        node_context_data.insert(root, TreeData::default());

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
            root: root.into(),
            bounds: Bounds::default(),
            needs_relayout: false,

            source,
            selected_node: SelectedNode::default(),

            use_rounding: true,
        }
    }

    #[allow(clippy::borrowed_box)]
    pub fn get_node(&self) -> &Box<dyn Widget<Action>> { &self.node }

    pub fn set_node(
        &mut self,
        mut node: Box<dyn Widget<Action>>,
        values: &mut dyn Reflect,
        
        default_css: &str,
        text_layout_contexts: &mut TextLayoutContexts,
    ) {
        // clear the tree and all our children
        self.clear();

        // TODO: refresh layout when scale changes
        let ui_scale = values.reflect_get::<f32>("settings.ui_scale")
            .map(|i| *i)
            .unwrap_or(1.0);

        let style = node.get_style_str();
        let mut resolver = CssResolver::new(&style, default_css);

        // layout the new node
        let mut shell = LayoutShell {
            source: self.source,
            tree: self,
            values,
            ui_scale,
            resolver: &mut resolver,
            text_layout_contexts,
        };

        let root = node
            .layout(&mut shell)
            .expect("failed to layout new node?");
        shell.tree.root = root;
        
        node.init_style(&mut shell);
        shell.tree
            .nodes[root.into()]
            .style = StyleStack::menu_layout();

        self.node = node;
        self.update_layout(values);
    }


    pub fn mark_for_relayout(&mut self) {
        self.needs_relayout = true;
    }

    /// Overrides a node's display without affecting the underlying style
    /// 
    /// Also marks dirty and for relayout
    pub fn set_overrides(
        &mut self,
        node: NodeId,
        f: impl FnOnce(&mut Box<dyn StylePropertyGroup>),
    ) {
        let Some(data) = self.nodes.get_mut(node.into())
        else { return };

        f(&mut data.style.get_group(StyleId::Overrides).unwrap());

        // if data.current_display == display { return }
        // data.current_display = display;

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
        self.needs_relayout = false;
        use taffy::AvailableSpace::*;
        let space = taffy::Size {
            width: Definite(self.bounds.size.x),
            height: Definite(self.bounds.size.y),
        };


        let root = self.root;
        super::LayoutTree {
            viewport: self.bounds.size,
            use_rounding: self.use_rounding,
            root_font_size: self.root_font_size(values),
            values,
            tree: self,
        }.compute_layout(root, space);

        self.update_contexts();
    }

    fn update_styles_inner(&mut self, node: NodeId, values: &dyn Reflect) {
        if node != self.root {
            let node = node.into();
            let a = &mut self.nodes[node];
            
            if a.cache.is_empty() {
                let parent = self.parents[node].unwrap().into();
                assert_ne!(node, parent);

                let parent_style = self.nodes[parent].style.clone();

                let a = &mut self.nodes[node];
                a.style = parent_style;

                let b = &self.node_context_data[node];
                // FIXME: !!!!!!!!!!!!!!!!!!!!!!
                // let style = &b.element_data.style().0;
                // a.current_style.merge_with_collection(style);
            }
        }
        let children = self.children.keys().collect::<Vec<_>>();
        for i in children {
            self.update_styles_inner(i.into(), values);
        }
    }


    pub fn update_contexts(&mut self) {
        // update absolute positions and matrices
        let matrix = Matrix::identity().trans(self.bounds.pos);
        self.recurse_update_context(self.root, matrix);

        // update spatial navigation
        SpatialNagivation::new(self)
            .run(&NavigateConfig::default());
    }

    fn recurse_update_context(
        &mut self,
        node: NodeId,
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
                layout.location.y,
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

        for child in self.children(node).into_owned() {
            self.recurse_update_context(child, matrix);
        }
    }

    pub fn update_context(&mut self, node: NodeId) {
        let our_matrix = self
            .get_context(node)
            .map_or_else(
                || Matrix::identity().trans(self.bounds.pos),
                |p| p.global_transform
            );
        self.recurse_update_context(node, our_matrix);
    }

    pub fn absolute_bounds(&self, node: NodeId) -> Option<Bounds> {
        self.get_context(node).map(|i| i.absolute_bounds)
    }

    pub fn content_bounds(&self, node: NodeId) -> Option<Bounds> {
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
    pub fn bounds(&self, node: NodeId) -> Option<Bounds> {
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


    pub fn has_child(
        &self,
        parent: NodeId,
        child: NodeId,
    ) -> bool {
        let Some(Some(parent_id)) = self.parents
            .get(child.into())
        else { return false };
        parent_id == &parent
    }
    pub fn all_children(&self) -> impl Iterator<Item = NodeId> {
        self.nodes.keys()
            .map(NodeId::from)
            .filter(|i| i != &EMPTY_NODE)
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



    // pub fn set_styles<_T>(
    //     &mut self, 
    //     node: NodeId,
    //     styles: ElementStateStyles<StylePropertyCollection, _T>, 
    // ) {
    //     let Some(ctx) = self.get_context_mut(node)
    //     else { return };
    //     ctx.element_data.styles = styles.transpose();
    //     self.mark_dirty(node);
    // }
    pub fn get_style(&self, node: NodeId) -> Option<&StyleStack> {
        Some(
            &self.nodes
            .get(node.into())?
            .style
        )
    }
    pub fn get_text_style(&self, node: NodeId) -> Option<&TextStyle> {
        Some(
            &self.nodes
                .get(node.into())?
                .text_style
        )
    }


    
    // widget things

    pub fn handle_inputs(
        &mut self,
        input_state: &mut CurrentInputState,
        values: &mut dyn Reflect,
        actions: &mut Vec<Action>,
        messages: &mut Vec<Message>,
    ) -> bool {
        let consumed = self.with_node(|tree, node| {
            let mut shell = InputShell {
                source: tree.source,
                messages,
                actions,
                tree,
                values,
                mouse_pos: input_state.mouse_pos,
                event_consumed: false,
            };

            // if input_state.mouse_moved {
            //     node.input(
            //         &input_state
            //             .make_input(InputType::MouseMove(input_state.mouse_pos)),
            //         &mut shell
            //     );
            // }
            // if input_state.scroll_delta.x.abs() > f32::EPSILON || input_state.scroll_delta.y.abs() > f32::EPSILON{
            //     node.input(
            //         &input_state
            //             .make_input(InputType::MouseScroll(input_state.scroll_delta)),
            //         &mut shell
            //     );
            // }

            let mouse_pos = input_state.mouse_pos;
            let key_mods = input_state.mods;

            input_state.events.retain(|event| {
                let event = input::InputEvent {
                    event: event.clone(),
                    mouse_pos,
                    key_mods
                };
                node.input(&event, &mut shell);

                !shell.event_consumed.take()
            });

            shell.event_consumed
        });

        if consumed { return true }

        // self.handle_spacial_navigation(input_state, values, actions, messages)
        false
    }

    pub fn handle_message(
        &mut self,
        message: &Message,
        values: &mut dyn Reflect,
        actions: &mut Vec<Action>,
        messages: &mut Vec<Message>,
    ) {
        self.with_node(|tree, node| {
            let mut shell = MessageShell {
                values,
                actions,
                messages,
                source: tree.source,
                tree,
                handled: false
            };
            node.handle_message(message, &mut shell);
        });
    }

    pub fn handle_event(
        &mut self,
        event: &input::TatakuEvent,
        passed_in: Option<&TatakuValue>,
        values: &mut dyn Reflect,
        actions: &mut Vec<Action>,
        messages: &mut Vec<Message>,
    ) {
        self.with_node(|tree, node| {
            let mut shell = MessageShell {
                values,
                actions,
                messages,
                source: tree.source,
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
        actions: &mut Vec<Action>,
        messages: &mut Vec<Message>,
        skin_manager: &mut dyn graphics::SkinProvider,

        default_css: &str,
        text_layout_contexts: &mut TextLayoutContexts,
    ) {
        if self.needs_relayout {
            self.update_layout(values);
        }

        // update the root widget
        self.with_node(|tree, node| {
            let mut shell = UpdateShell {
                source: tree.source,
                tree,
                values,
                actions,
                messages,
                skin_manager,
                
                default_css,
                text_layout_contexts,
            };
            node.update(&mut shell);
        });
    }

    pub fn reload_skin(
        &mut self,
        values: &mut dyn Reflect,
        messages: &mut Vec<Message>,
        actions: &mut Vec<Action>,
        skin_manager: &mut dyn graphics::SkinProvider,

        default_css: &str,
        text_layout_contexts: &mut TextLayoutContexts,
    ) {
        self.with_node(|tree, node| {
            // update the root widget
            let mut shell = UpdateShell {
                source: tree.source,
                tree,
                values,
                actions,
                messages,
                skin_manager,

                default_css,
                text_layout_contexts,
            };
            node.reload_skin(&mut shell);
        });
    }

    pub fn draw(
        &mut self,
        values: &dyn Reflect,
        list: &mut graphics::RenderableCollection,
        text_layout_contexts: &mut TextLayoutContexts,
    ) {
        self.with_node(|tree, node| {
            let mut shell = DrawShell {
                tree,
                list,
                values,
                // TODO: make customizable
                general_theme: GeneralUiTheme::default(),
                text_layout_contexts,
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
        use input::{
            Key,
            GamepadButton,
            InputType,
        };

        let mut consumed = false;

        #[derive(From)]
        #[derive(Copy, Clone)]
        enum MenuInputType {
            Key(Key),
            Controller(GamepadButton),
            // Axis()
        }
        impl MenuInputType {
            fn has(
                self,
                state: &mut CurrentInputState
            ) -> bool {
                match self {
                    Self::Key(key)
                        => state.keys_down().any(|k| k == key),
                    Self::Controller(btn)
                        => state.controller_down().any(|b| b == &btn),
                }
            }
            fn remove_from(
                self,
                state: &mut CurrentInputState,
            ) {
                match self {
                    Self::Key(key) => {
                        state.events.retain(|e| {
                            let InputType::KeyPress(k) = e
                            else { return true };

                            k.key != Some(key)
                        });
                    },
                    Self::Controller(btn) => {
                        state.events.retain(|e| {
                            let InputType::ControllerPress(cb, _, _) = e
                            else { return true };

                            cb != &btn
                        });
                    }
                }
            }
        }

        for (input, direction) in [
            (MenuInputType::Key(Key::Left), Direction::Left),
            (Key::Right.into(), Direction::Right),
            (Key::Up.into(), Direction::Up),
            (Key::Down.into(), Direction::Down),
            (Key::Tab.into(), Direction::Down),

            (GamepadButton::DPadLeft.into(), Direction::Left),
            (GamepadButton::DPadRight.into(), Direction::Right),
            (GamepadButton::DPadUp.into(), Direction::Up),
            (GamepadButton::DPadDown.into(), Direction::Down),
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

            if let Some(node) = self.get_context(current)
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
                &|tree, node| tree.context(node).selectable()
            );

            if let Some(node) = self.selected_node.node {
                self.context_mut(node).selected = Some(true);
            }
        }

    }

    // helpers for when we're certain the node is in the tree
    // private for that reason too
    fn context(&self, node: NodeId) -> &TreeData {
        self.get_context(node).unwrap()
    }
    fn context_mut(&mut self, node: NodeId) -> &mut TreeData {
        self.get_context_mut(node).unwrap()
    }

    /// this isnt the most efficient thing ever but hopefully its not used too often
    fn find_child(
        &self,
        parent: NodeId,
        f: &dyn Fn(&Self, NodeId) -> bool,
    ) -> Option<NodeId> {
        if f(self, parent) {
            return Some(parent)
        }

        for child in self.children(parent).iter() {
            if let Some(node) = self.find_child(*child, f) {
                return Some(node)
            }
        }

        None
    }

}

// taffy tree things
impl<Action: Send + Sync + 'static> Tree<Action> {
    pub fn new_leaf(&mut self) -> taffy::TaffyResult<NodeId> {
        let id = self.nodes.insert(LayoutData::default());
        self.node_context_data.insert(id, TreeData::default());
        let _ = self.children.insert(Vec::new());
        let _ = self.parents.insert(None);

        Ok(id.into())
    }

    fn set_children(
        &mut self,
        parent: NodeId,
        children: &[NodeId]
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
        parent_children.extend(children.iter().copied());

        self.mark_dirty(parent);

        Ok(())
    }
    fn remove_child(
        &mut self,
        parent: NodeId,
        child: NodeId
    ) -> taffy::TaffyResult<NodeId> {
        let index = self
            .children[parent.into()]
            .iter()
            .position(|n| *n == child)
            .unwrap();
        self.remove_child_at_index(parent, index)
    }
    fn remove_child_at_index(
        &mut self,
        parent: NodeId,
        child_index: usize
    ) -> taffy::TaffyResult<NodeId> {
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

        self.set_children(id, children)?;
        Ok(id)
    }

    pub fn add_child(
        &mut self,
        parent: NodeId,
        child: NodeId,
    ) {
        let parent_key = parent.into();
        let child_key = child.into();
        self.parents[child_key] = Some(parent);
        self.children[parent_key].push(child);
        self.mark_dirty(parent);
        self.mark_for_relayout();
    }

    pub fn remove(&mut self, node: NodeId) {
        let key = node.into();
        if let Some(parent) = self.parents[key]
        && let Some(children) = self.children.get_mut(parent.into()) {
            children.retain(|f| *f != node);
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

    pub fn get_layout(&self, node: NodeId) -> Option<&taffy::Layout> {
        let a = self.nodes.get(node.into())?;

        if self.use_rounding {
            Some(&a.final_layout)
        } else {
            Some(&a.unrounded_layout)
        }
    }

    pub fn parent(&self, node: NodeId) -> Option<NodeId> {
        *self.parents
            .get(node.into())?
    }
    pub fn children<'a>(&'a self, parent: NodeId) -> Cow<'a, [NodeId]> {
        let Some(children) = self.children
            .get(parent.into())
        else { return Vec::new().into() };

        children.into()
    }


    pub fn get_context(&self, node: NodeId) -> Option<&TreeData> {
        self.node_context_data.get(node.into())
    }
    pub fn get_context_mut(&mut self, node: NodeId) -> Option<&mut TreeData> {
        self.node_context_data.get_mut(node.into())
    }

    /// Mark a node as dirty, also marks the tree for re-layout
    pub fn mark_dirty(&mut self, node: NodeId) {
        fn mark_dirty_recursive(
            nodes: &mut SlotMap<DefaultKey, LayoutData>,
            parents: &SlotMap<DefaultKey, Option<NodeId>>,
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
            node.into()
        );
        self.mark_for_relayout();
    }


    // // FIXME: implement new api
    // pub fn update_style(&mut self, node: NodeId, f: impl Fn(&mut CssStyle)) {
    //     let Some(data) = self
    //         .node_context_data
    //         .get_mut(node.into())
    //     else { return };

    //     // data.element_data.styles
    //     //     .all_mut()
    //     //     .into_iter()
    //     //     .for_each(|(i,_)| f(i));

    //     self.mark_dirty(node);
    //     self.mark_for_relayout();
    // }

    fn root_font_size(&self, values: &dyn Reflect) -> f32 {
        self.nodes
            .get(self.root.into())
            .unwrap()
            .style
            .get(CssProperty::FontSize)
            .and_then(|p| p.value().resolve_copied(values))
            .unwrap_or(32.0)
    }
    pub fn print(&mut self, values: &dyn Reflect) {
        let root = self.root;
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


mod export_tree {
    use super::LayoutTree;
    use tataku_engine_common::prelude::*;
    use taffy::{
        NodeId,
        TraversePartialTree,
    };

    impl<A: Send + Sync + 'static> super::Tree<A> {
        pub fn export_xml(&mut self, values: &dyn super::Reflect) -> String {
            let mut lines = Vec::new();
            let id = self.root;

            let tree = LayoutTree {
                values,
                tree: self,
                viewport: Vector2::ZERO,
                root_font_size: 0.0,
                use_rounding: false,
            };

            Self::export_node_xml(
                &tree, 
                &mut lines, 
                0,
                id,
            );

            lines.join("\n")
        }

        fn export_node_xml(
            tree: &LayoutTree<A>, 
            lines: &mut Vec<String>,
            indent: usize,
            id: NodeId,
        ) {
            let spacing = "  ".repeat(indent);
            let ctx = tree.tree.context(id);
            let data = &ctx.element_data;
            let style = tree.tree.get_style(id).unwrap();

            let ele = &data.element_name;
            let ele_id = data.id.as_ref()
                .map(|i| format!("id='{i}'"))
                .unwrap_or_default();

            let class_list = if !data.class_list.is_empty() {
                format!("class='{}'", super::ArcStr::join(&data.class_list, " "))
            } else { String::new() };

            lines.push(format!("{spacing}<{ele} {ele_id} {class_list}>"));
            // // style
            // style.export_xml(
            //     lines,
            //     indent + 1,
            //     tree.values,
            // );
            // lines.push(String::new());
            
            let children = tree.child_ids(id);
            for i in children {
                Self::export_node_xml(
                    tree, 
                    lines, 
                    indent + 1, 
                    i
                );
            }

            lines.push(format!("{spacing}</{ele}>"));
        }

    }
}
