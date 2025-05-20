use crate::prelude::*;
use crate::prelude::ui::*;

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
    pub fn new(
        cap: usize,
        owner: MessageOwner,
        node: Box<dyn Widget>,
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
            node,
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
        let ui_scale = values.reflect_get::<f32>("settings.ui_scale")
            .map(|i| *i)
            .unwrap_or(1.0);
        
        // layout the new node
        let mut shell = LayoutShell {
            owner: self.owner,
            tree: self,
            values,
            ui_scale,
        };

        let new = node
            .layout(&mut shell)
            .expect("failed to layout new node?");
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
        self.tree.set_node_context(
            self.root.node_id, 
            Some(TreeData::default())
        ).unwrap();
        
        self.update_layout(values);
    }

    #[allow(clippy::borrowed_box)]
    pub fn get_node(&self) -> &Box<dyn Widget> { &self.node }
    pub fn mark_refresh(&mut self, _s: &str) {
        self.should_refresh = true;
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
        let space = Size {
            width: Definite(self.bounds.size.x),
            height: Definite(self.bounds.size.y),
        };

        // before we compute the layout we need to update all the css styles
        // these should save the results in the context
        let style = self.node.get_style_str();
        let mut resolver= CssResolver::new(&style);
        

        self.with_node(|tree, node| {
            let mut shell = StyleShell {
                tree,
                values, 
                resolver: &mut resolver,
            };

            node.update_styles(&mut shell, None);
        });

        self.tree
            .mark_dirty(self.root.node_id)
            .expect("failed to mark dirty?");

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
            .run(&NavigateConfig::default());
    }

    fn recurse_update_context(
        &mut self, 
        node: TaffyNodeId, 
        mut matrix: Matrix
    ) {
        if !self.all_children.contains(&node) {
            eprintln!("node not in tree!!! {node:?}");
            return;
        }

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

        matrix = matrix 
            * context.local_transform.matrix()
            * Matrix::identity().trans(bounds.pos);

        for child in self.tree.children(node).unwrap() {
            self.recurse_update_context(child, matrix);
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

    pub fn all_children(&self) -> impl Iterator<Item = TaffyNodeId> {
        self.all_children.iter().copied().filter(|i| i != &EMPTY_NODE.node_id)
    }



    fn with_node<T>(
        &mut self, 
        f: impl FnOnce(&mut Tree, &mut Box<dyn Widget>) -> T + Send + Sync
    ) -> T {
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
        messages: &mut Vec<Message>,
    ) -> bool {
        // let mouse_pos = input_state.mouse_pos;
        let mut consumed = self.with_node(|tree, node| {
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
            if input_state.scroll_delta.abs() > f32::EPSILON {
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
                !std::mem::take(&mut shell.event_consumed)
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
                !std::mem::take(&mut shell.event_consumed)
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
                !std::mem::take(&mut shell.event_consumed)
            });

            shell.event_consumed
        });

        if !consumed {
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
                    consumed = true;
                    // return since this was just to enable navigation
                    // otherwise we'd immediate select the next node, without selecting the current node
                    break;
                }

                let Some(current) = self.selected_node.node else { 
                    warn!("No active node to navigate from ??");
                    break
                };

                if let Some(node) = self.tree
                    .get_node_context(current.node_id)
                    .and_then(|i| i.node_direction(direction)) {
                    self.context_mut(current).selected = Some(false);
                    self.context_mut(node).selected = Some(true);
                    input_state.keys_down.remove_key(key);
                    consumed = true;
                }

                break
            }
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
        f: &impl Fn(&Self, TaffyNodeId) -> bool
    ) -> Option<NodeId> {
        let parent = parent.get_id();
        if f(self, parent) { return Some(NodeId::new(parent, self.owner)) }
        for child in self.tree.children(parent).ok()? {
            if let Some(node) = self.find_child(child, &f) { 
                return Some(node) 
            }
        }
        None
    }


    // widget things

    pub fn handle_message(
        &mut self,
        message: &Message,
        values: &mut dyn Reflect,
        actions: &mut ActionQueue,
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
        event: TatakuEventType,
        passed_in: Option<TatakuValue>,
        values: &mut dyn Reflect,
        actions: &mut ActionQueue,
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
            node.handle_event(event, passed_in, &mut shell);
        });
    }

    pub fn update(
        &mut self,
        values: &mut dyn Reflect,
        actions: &mut ActionQueue,
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
        actions: &mut ActionQueue,
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
        list: &mut RenderableCollection
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

    



    // TaffyTree things
    pub fn new_leaf(&mut self, style: Style) -> TaffyResult<NodeId> {
        let id = self.tree.new_leaf(style)?;

        if self.all_children.insert(id) {
            self.tree.set_node_context(id, Some(TreeData::default()))?;
        }

        let id = NodeId::new(id, self.owner);
        Ok(id)
    }

    pub fn new_with_children(
        &mut self, 
        style: Style, 
        children: &[NodeId]
    ) -> TaffyResult<NodeId> {
        let id = self.new_leaf(style)?;
        let children = children
            .iter()
            .map(|i| i.node_id)
            .collect::<Vec<_>>();
        
        self.tree.set_children(id.node_id, &children)?;
        Ok(id)
    }

    pub fn add_child(
        &mut self, 
        parent: impl HasNodeId, 
        child: impl HasNodeId,
    ) {
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
    pub fn children(&self, parent: impl HasNodeId) -> Option<Vec<NodeId>> {
        self.tree.children(parent.get_id())
            .map(|list| 
                list.into_iter()
                .map(|i| NodeId::new(i, self.owner))
                .collect()
            ).ok()
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
        if let Err(e) = self.tree.set_style(node.get_id(), style) {
            error!("Error updating style: {e:?}");
        }
    }

    pub fn print(&mut self) {
        self.tree.print_tree(self.root.node_id);
    }
}

#[derive(Copy, Clone, Debug, Default)]
struct SelectedNode {
    node: Option<NodeId>,
    active: bool,
}
