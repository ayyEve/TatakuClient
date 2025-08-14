use crate::prelude::*;

macro_rules! get_list {
    ($data: expr, $values: expr) => {{
        let path = $data.list_var.clone();
        let Ok(path) = path.resolve_path($values)
            .inspect_err(|e| $data.print_err(e))
        else { return };

        let Ok(iter) = $values
            .reflect_iter(&*path)
            .inspect_err(|e| $data.print_err(e))
        else { return };

        iter
    }}
}


#[derive(ChainableInitializer)]
pub struct Container {
    children: Vec<Box<dyn Widget<TatakuAction>>>,

    #[chain] id: CowStr,
    #[chain] scrollable: bool,
    programmatic: Option<ProgrammaticListData>,

    // TODO: rename and fix this
    /// should events apply to all elements?
    #[chain] drag_scroll: bool,
    drag_scroll_data: DragScrollData,
    scroll_offset: Vector2,

    node_id: NodeId,
}
impl Container {
    pub fn new(items: Vec<Box<dyn Widget<TatakuAction>>>) -> Self {
        Self {
            children: items,
            node_id: EMPTY_NODE,
            scrollable: false,

            drag_scroll: false,
            programmatic: None,
            drag_scroll_data: DragScrollData::default(),
            scroll_offset: Vector2::ZERO,
            id: "".into(),
        }
    }

    fn check_scroll(
        &mut self,
        scroll: ScrollPosition,
        layout: &taffy::Layout,
    ) -> bool {
        if !self.scrollable { return false }
        match scroll {
            ScrollPosition::None => false,
            ScrollPosition::Relative(delta) => {
                let new_scroll = (self.scroll_offset.y + delta.y)
                    .clamp(-layout.scroll_height(), 0.0);

                if (self.scroll_offset.y + new_scroll).abs() > f32::EPSILON {
                    self.scroll_offset.y = new_scroll;
                    true
                } else {
                    false
                }
            }
            ScrollPosition::Absolute(pos) => {
                let size = Vector2::new(
                    layout.scroll_width(),
                    layout.scroll_height(),
                );
                self.scroll_offset = (size * -pos)
                    .clamp(-size, Vector2::ZERO);
                true
            }
        }

    }

    pub fn make_programmatic(mut self, data: ProgrammaticListData) -> Self {
        self.programmatic = Some(data);
        self
    }


    fn validate_scroll_position(
        &mut self, 
        tree: &mut Tree<TatakuAction>
    ) {
        let layout = tree.get_layout(self.node_id).unwrap();
        let size = Vector2::new(
            layout.scroll_width(),
            layout.scroll_height(),
        );

        self.scroll_offset = self.scroll_offset.clamp(-size, Vector2::ZERO);
    }

    fn handle_scroll_operation(
        &mut self,
        scroll: &ScrollOperation,
        tree: &mut Tree<TatakuAction>,
    ) {
        match &scroll.scroll_type {
            ScrollType::ScrollByAmount(amt) 
                => self.scroll_offset += *amt,

            ScrollType::ScrollToPosition(pos) 
                => self.scroll_offset = -*pos,

            ScrollType::ScrollByPercent(percent) => {
                let layout = tree
                    .get_layout(self.node_id).unwrap();

                let size = Vector2::new(
                    layout.scroll_width(),
                    layout.scroll_height(),
                );
                self.scroll_offset += (size * -*percent)
                    .clamp(-size, Vector2::ZERO);
            }
            ScrollType::ScrollToPercent(percent) => {
                let layout = tree
                    .get_layout(self.node_id).unwrap();

                let size = Vector2::new(
                    layout.scroll_width(),
                    layout.scroll_height(),
                );
                self.scroll_offset = (size * -*percent)
                    .clamp(-size, Vector2::ZERO);
            }

            ScrollType::ScrollToId(id) => {
                let Some((i, _)) = self.children
                    .iter()
                    .map(|c| 
                        (c, tree.get_context(c.node_id()).unwrap())
                    )
                    .find(|(_, t)| 
                        t.element_data.id.as_deref() == Some(&**id)
                    )
                else { return warn!("scroll: id not found: {id}")};
                let node = i.node_id();

                self.handle_scroll_operation(
                    &ScrollOperation { 
                        scroll_type: ScrollType::ScrollToNode(node) 
                    }, 
                    tree
                );
            }

            // scroll to a specific node id
            ScrollType::ScrollToNode(node) => {
                let our_layout = tree
                    .get_layout(self.node_id).unwrap();

                let node_layout = tree
                    .get_layout(*node).unwrap();

                let top = Vector2::from(node_layout.location);

                let offset = (
                    Vector2::from(our_layout.size - node_layout.size)
                    / 2.0
                ).clamp(
                    -Vector2::from(our_layout.size),
                    Vector2::ZERO, 
                );
                
                self.scroll_offset = top + offset;
            }

        }

        self.validate_scroll_position(tree);
    }
}

impl Widget<TatakuAction> for Container {
    fn name(&self) -> CowStr { "container_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn children(&self) -> WidgetChildren<TatakuAction> {
        WidgetChildren::List(&self.children)
    }
    fn children_mut(&mut self) -> WidgetChildrenMut<TatakuAction> {
        WidgetChildrenMut::List(&mut self.children)
    }
    
    fn layout(&mut self, shell: &mut LayoutShell<TatakuAction>) -> taffy::TaffyResult<NodeId> {
        let children = self.children
            .iter_mut()
            .map(|w| w.layout(shell))
            .collect::<taffy::TaffyResult<Vec<_>>>()?;

        self.node_id = shell.tree.new_with_children(&children)?;
        
        shell.with_context(
            self.node_id, 
            |ctx| ctx.needs_inverse_transform = true,
        );

        Ok(self.node_id)
    }

    fn input(
        &mut self,
        event: &InputEvent,
        shell: &mut InputShell<TatakuAction>,
    ) {
        let node_id = self.node_id;
        let Some(layout) = shell.tree.get_layout(node_id).copied()
        else { return };

        if let InputType::MouseScroll(delta) = &event.event {
            if shell.event_consumed { return }
            if self.check_scroll(
                ScrollPosition::Relative(*delta),
                &layout
            ) {
                let context = shell
                    .tree
                    .get_context_mut(node_id)
                    .unwrap();

                context.local_transform.pos = self.scroll_offset;
                shell.actions.push(UiAction::new(
                    self.node_id,
                    UiActionType::ContextChanged
                ));

                shell.event_consumed = true;
                return
            }
        }


        let mut captured = false;
        if let Some(data) = &mut self.programmatic {
            let iter = get_list!(data, shell.values);

            let values = iter
                .filter_map(|v| v.duplicate())
                .collect::<Vec<_>>();

            let path = ReflectPath::new(&data.variable);
            for (w, value) in self
                .children
                .iter_mut()
                .zip(values) 
            {
                shell
                    .values
                    .impl_insert(path.clone(), value)
                    .expect("error inserting into values");
                w.input(event, shell);

                captured |= shell.event_consumed;
                if self.drag_scroll {
                    shell.event_consumed = false;
                } else if captured {
                    break;
                }
            }
        } else {
            for i in self.children.iter_mut() {
                i.input(event, shell);

                captured |= shell.event_consumed;
                if self.drag_scroll {
                    shell.event_consumed = false;
                } else if captured {
                    break;
                }
            }
        }

        shell.event_consumed = captured;

        if !captured && self.scrollable && self.drag_scroll {
            let offset = self
                .drag_scroll_data
                .check_input(self.node_id, shell, event);

            if self.check_scroll(offset, &layout) {
                shell.event_consumed = true;
                let context = shell
                    .tree
                    .get_context_mut(self.node_id)
                    .unwrap();

                context.local_transform.pos = self.scroll_offset;
                shell.actions.push(UiAction::new(
                    self.node_id,
                    UiActionType::ContextChanged
                ));
            }
        }
    }

    fn update(&mut self, shell: &mut UpdateShell<TatakuAction>) {
        if let Some(data) = &mut self.programmatic {
            let iter = get_list!(data, shell.values);

            let values = iter
                .filter_map(|v| v.duplicate())
                .collect::<Vec<_>>();

            // FIXME: need to reload skin for added children
            // make sure our list of children is the same length as the list of values
            let diff = self.children.len() as i64 - values.len() as i64;
            match diff {
                0 => {} // children and values are the same length, nothing to do
                (..0) => {
                    // children is too small, need to add elements
                    let style = shell.tree.node.get_style_str(); // e.get_style_str();
                    let mut resolver = CssResolver::new(&style);

                    for _ in 0..diff.abs() {
                        // create the new element
                        let mut e = data.template.build();

                        let mut layout_shell = LayoutShell {
                            tree: shell.tree,
                            values: shell.values,
                            owner: shell.owner,
                            ui_scale: 1.0, // TODO:!
                            resolver: &mut resolver,
                        };

                        // add it to the tree
                        let child = match e.layout(&mut layout_shell) {
                            Ok(n) => n,
                            Err(e) => panic!("Error laying out new custom list child! {e}"), // FIXME: not panic?
                        };

                        // make us its parent
                        layout_shell.tree.add_child(self.node_id, child);
                        
                        // init it's style
                        e.init_style(&mut layout_shell);

                        // add to our list
                        self.children.push(e);
                    }

                    // // mark the tree as dirty
                    // shell.actions.push(UiAction::new(
                    //     self.node_id,
                    //     UiActionType::MarkDirty
                    // ));
                    // shell.actions.push(UiAction::new(
                    //     self.node_id,
                    //     UiActionType::Refresh
                    // ));
                }
                (0..) => {
                    // too many elements, remove some
                    for _ in 0..diff.abs() {
                        // remove it from our list
                        let removed = self
                            .children
                            .swap_remove(0);

                        // remove it from the tree
                        shell.tree.remove(removed.node_id());
                    }

                    // mark the tree as dirty
                    shell.actions.push(UiAction::new(
                        self.node_id,
                        UiActionType::MarkDirty
                    ));
                    shell.actions.push(UiAction::new(
                        self.node_id,
                        UiActionType::Refresh
                    ));
                }
            }

            let path = ReflectPath::new(&data.variable);
            for (i, value) in self
                .children
                .iter_mut()
                .zip(values)
            {
                shell.values
                    .impl_insert(path.clone(), value)
                    .expect("error inserting into values");
                i.update(shell);
            }

        } else {
            for i in self.children.iter_mut() {
                i.update(shell);
            }
        }
    }

    fn draw(&self, shell: &mut DrawShell<TatakuAction>) {
        let Some(our_bounds) = shell.tree.absolute_bounds(self.node_id)
        else { return };

        let mut list = RenderableCollection::new();
        if self.scrollable {
            std::mem::swap(shell.list, &mut list);
        }

        for i in self.children.iter() {
            // dont attempt to draw items outside our bounds
            let Some(ibounds) = shell.tree.absolute_bounds(i.node_id())
            else { continue };
            if our_bounds.intersection(ibounds).is_none() { continue }
            i.draw(shell);
        }

        if self.scrollable {
            std::mem::swap(shell.list, &mut list);

            let elements = list
                .list
                .into_iter()
                .map(|element| Scissored::new(
                    our_bounds.into_scissor(),
                    element
                ))
                .map(|element| Box::new(element) as Box<dyn TatakuRenderable>);

            shell.list.list.extend(elements);
        }

        // TODO: draw scrollbar if scrollable
        // let Some(l) = shell.tree.get_layout(self).cloned() else { return };

        // // let bounds = Bounds::from(&l);
        // if l.scrollbar_size.has_non_zero_area() {
        // }
    }

    fn draw_overlay(&self, shell: &mut DrawShell<TatakuAction>) {
        let Some(our_bounds) = shell.tree.absolute_bounds(self.node_id)
        else { return };

        let mut list = RenderableCollection::new();
        if self.scrollable {
            std::mem::swap(shell.list, &mut list);
        }

        for i in self.children.iter() {
            // dont attempt to draw items outside our bounds
            let Some(ibounds) = shell.tree.absolute_bounds(i.node_id())
            else { continue };

            if our_bounds.intersection(ibounds).is_none() { continue }
            i.draw_overlay(shell);
        }

        if self.scrollable {
            std::mem::swap(shell.list, &mut list);

            let elements = list.list.into_iter()
                .map(|element| Scissored::new(
                    our_bounds.into_scissor(),
                    element
                ))
                .map(|element| Box::new(element) as Box<dyn TatakuRenderable>);

            shell.list.list.extend(elements);
        }
    }

    fn handle_message(
        &mut self,
        message: &Message,
        shell: &mut MessageShell<TatakuAction>,
    ) {
        if let Some(data) = &mut self.programmatic {
            let iter = get_list!(data, shell.values);

            let values_ = iter
                .filter_map(|v| v.duplicate())
                .collect::<Vec<_>>();

            let path = ReflectPath::new(&data.variable);
            for (i, value) in self
                .children
                .iter_mut()
                .zip(values_)
            {
                shell.values
                    .impl_insert(path.clone(), value)
                    .expect("error inserting into values");
                i.handle_message(message, shell);
            }
        } else {
            for i in self.children.iter_mut() {
                i.handle_message(message, shell);
            }
        }
    }

    fn handle_event(
        &mut self,
        event: &TatakuEventType,
        event_value: Option<&TatakuValue>,
        shell: &mut MessageShell<TatakuAction>,
    ) {
        if let Some(data) = &mut self.programmatic {
            let iter = get_list!(data, shell.values);

            let values = iter
                .filter_map(|v| v.duplicate())
                .collect::<Vec<_>>();

            let path = ReflectPath::new(&data.variable);
            for (i, value) in self
                .children
                .iter_mut()
                .zip(values) 
            {
                shell
                    .values
                    .impl_insert(path.clone(), value)
                    .expect("error inserting into values");

                i.handle_event(event, event_value, shell);
            }
        } else {
            for i in self.children.iter_mut() {
                i.handle_event(event, event_value, shell);
            }
        }
    }

    fn reload_skin(&mut self, shell: &mut UpdateShell<TatakuAction>) {
        for i in self.children.iter_mut() {
            i.reload_skin(shell);
        }
    }

    fn operation(
        &mut self, 
        operation: &UiOperation, 
        tree: &mut Tree<TatakuAction>,
    ) {
        if operation.target.resolve(self, tree) {
            #[allow(clippy::single_match, reason = "expansion")]
            match &operation.operation {
                UiOperationType::Scroll(scroll) 
                    => self.handle_scroll_operation(scroll, tree),

                _ => {}
            }
        } else {
            for i in self.children.iter_mut() {
                i.operation(operation, tree);
            }
        }
    }
}

#[derive(ChainableInitializer)]
pub struct ProgrammaticListData {
    /// What element to build for each iteration
    #[chain] pub template: Element,

    /// What variable to iterate over
    #[chain] pub list_var: VariablePathResolver,

    /// What var name to store the iter variable in (ie the `i` in `for i in ...`)
    #[chain] pub variable: ArcStr,

    error_printed: bool,
}
impl ProgrammaticListData {
    pub fn new(template: Element, list_var: ArcStr, variable: ArcStr) -> Self {
        Self {
            template,
            list_var: VariablePathResolver::new(list_var),
            variable,
            error_printed: false,
        }
    }

    fn print_err(&mut self, error: &dyn std::fmt::Debug) {
        if self.error_printed { return }

        self.error_printed = true;
        error!("!!!!!!!!!!!!!!");
        error!("List variable error! '{:?}' {error:?}", self.list_var);
        error!("!!!!!!!!!!!!!!");
    }

}

/// how far the
const DRAG_THRESHOLD:f32 = 5.0;

#[derive(Default)]
struct DragScrollData {
    left_pressed: bool,
    right_pressed: bool,
    pressed_at: Vector2,
    did_move: bool,
}

impl DragScrollData {
    fn check_input(
        &mut self,
        node_id: NodeId,
        shell: &mut InputShell<TatakuAction>,
        event: &InputEvent,
    ) -> ScrollPosition {
        let Some(bounds) = shell.tree.absolute_bounds(node_id)
        else { return ScrollPosition::None };

        let hover = bounds.contains(shell.mouse_pos);

        match event.event {
            InputType::MousePress(b) if hover => {
                match b {
                    MouseButton::Left if !self.right_pressed 
                        => self.left_pressed = true,
                    MouseButton::Right if !self.left_pressed 
                        => self.right_pressed = true,

                    _ => return ScrollPosition::None,
                }

                self.pressed_at = shell.mouse_pos;
            }

            InputType::MouseRelease(b) => {
                match b {
                    MouseButton::Left if self.left_pressed 
                        => self.left_pressed = false,
                    MouseButton::Right if self.right_pressed 
                        => self.right_pressed = false,
                    _ => return ScrollPosition::None
                }
                // if the mouse moved, we dont want to register the release key, 
                // so return that it was consumed
                self.did_move = false;

                // FIXME: dont use this hack lmao
                return ScrollPosition::Relative(Vector2::ZERO);
            }
            InputType::MouseMove(position) if hover => {
                if !self.did_move 
                    && (self.left_pressed || self.right_pressed) 
                    && position.distance(self.pressed_at) > DRAG_THRESHOLD {
                    self.did_move = true;
                }

                // check left click
                if self.left_pressed && self.did_move {
                    // let diff = AbsoluteOffset {
                    //     x: -(position.x - self.pressed_at.x),
                    //     y: -(position.y - self.pressed_at.y),
                    // };
                    let diff = position - self.pressed_at;

                    // reset the clicked pos to move the delta
                    self.pressed_at = position;

                    // perform scroll
                    return ScrollPosition::Relative(diff)
                } else

                // check right click
                if self.right_pressed && self.did_move {
                    let pos = bounds.pos;
                    let size = bounds.size;

                    let move_to = ((position - pos) / size)
                        .clamp(Vector2::ZERO, Vector2::ONE);

                    return ScrollPosition::Absolute(move_to)
                    // let move_to = Vector2::new(
                    //     move_to.x.clamp(0.0, 1.0),
                    //     ((position.y - pos.y) / size.height).clamp(0.0, 1.0),
                    // );

                    // let mut operation = snap_to(RelativeOffset {
                    //     x: ((position.x - pos.x) / size.width).clamp(0.0, 1.0),
                    //     y: ((position.y - pos.y) / size.height).clamp(0.0, 1.0)
                    // });
                    // self.operate_on_children(tree, layout, renderer, &mut operation);
                }
            }

            _ => {}
        }

        ScrollPosition::None
    }

    // fn should_capture(&self) -> bool {
    //     (self.left_pressed || self.right_pressed) && self.did_move
    // }
}

enum ScrollPosition {
    None,

    /// move a relative amount
    Relative(Vector2),

    /// move to an absolute point (x and y are 0.0..1.0)
    Absolute(Vector2),
}
