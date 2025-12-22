use crate::prelude::*;
use common::reflect::*;
use tataku::{
    Vector2,
    TatakuValue,
};
use ui::{
    tree::*,
    style::*,
    widget::*,
    message::*,
};
use input::{ 
    InputType,
    InputEvent, 
};
use super::{
    *,
    drag_scroll_data::DragScrollData
};

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
    children: Vec<Box<dyn Widget<actions::Action>>>,

    #[chain] id: CowStr,
    #[chain] scroll_direction: ScrollDirection,
    programmatic: Option<ProgrammaticListData>,

    // TODO: rename and fix this
    /// should events apply to all elements?
    #[chain] drag_scroll: bool,
    drag_scroll_data: DragScrollData,
    scroll_offset: Vector2,

    node_id: NodeId,
}
impl Container {
    pub fn new(items: Vec<Box<dyn Widget<actions::Action>>>) -> Self {
        Self {
            children: items,
            node_id: ui::EMPTY_NODE,
            scroll_direction: ScrollDirection::None,

            drag_scroll: false,
            programmatic: None,
            drag_scroll_data: DragScrollData::default(),
            scroll_offset: Vector2::ZERO,
            id: "".into(),
        }
    }

    fn check_scroll(
        &mut self,
        scroll: &ScrollPosition,
        layout: &taffy::Layout,
    ) -> bool {
        if matches!(self.scroll_direction, ScrollDirection::None) { 
            return false 
        }

        match scroll {
            ScrollPosition::None => false,
            ScrollPosition::Relative(delta) => {
                let a = self.scroll_offset + *delta;
                let new_scroll = Vector2::new(
                    a.x.clamp(0.0, layout.scroll_width()),
                    a.y.clamp(-layout.scroll_height(), 0.0),
                );
                self.scroll_direction.apply(
                    &mut self.scroll_offset, 
                    new_scroll
                )
            }
            ScrollPosition::Absolute(pos) => {
                let size = Vector2::new(
                    layout.scroll_width(),
                    layout.scroll_height(),
                );
                
                self.scroll_direction.apply(
                    &mut self.scroll_offset, 
                    (size * -*pos).clamp(-size, Vector2::ZERO)
                )
            }
        }

    }

    pub fn make_programmatic(mut self, data: ProgrammaticListData) -> Self {
        self.programmatic = Some(data);
        self
    }


    fn validate_scroll_position(
        &mut self,
        tree: &mut Tree<actions::Action>,
    ) {
        let layout = tree.get_layout(self.node_id).unwrap();
        let size = Vector2::new(
            layout.scroll_width(),
            layout.scroll_height(),
        );

        self.scroll_offset = self.scroll_offset.clamp(-size, Vector2::ZERO);

        let ctx = tree.get_context_mut(self.node_id).unwrap();
        ctx.local_transform.pos = self.scroll_offset;
        tree.mark_dirty(self.node_id);
    }

    fn handle_scroll_operation(
        &mut self,
        scroll: &ScrollOperation,
        tree: &mut Tree<actions::Action>,
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
                let Some((i, _)) = self
                    .children
                    .iter()
                    .map(|c|
                        (c, tree.get_context(c.node_id()).unwrap())
                    )
                    .find(|(_, t)|
                        t.element_data.id.as_deref() == Some(&**id)
                    )
                else { return warn!("scroll: id not found: {id}")};

                return self.handle_scroll_operation(
                    &ScrollOperation {
                        scroll_type: ScrollType::ScrollToNode(i.node_id())
                    },
                    tree
                );
            }

            // scroll to a specific node id
            ScrollType::ScrollToNode(node) => {
                let our_bounds = tree.absolute_bounds(self.node_id).unwrap();
                // let our_layout = tree
                //     .get_layout(self.node_id).unwrap();

                // let node_layout = tree
                //     .get_layout(*node).unwrap();
                let node_bounds = tree.absolute_bounds(*node).unwrap();

                let top = Vector2::new(
                    node_bounds.pos.x,
                    -node_bounds.pos.y,
                );

                let offset = (our_bounds.size - node_bounds.size) / 2.0;
                // .clamp(
                //     -Vector2::from(our_bounds.size),
                //     Vector2::ZERO,
                // );

                self.scroll_offset = top + offset;
            }

            ScrollType::ScrollToActive {
                include_children: false
            } => {
                let Some((i, _)) = self
                    .children
                    .iter()
                    .map(|c|
                        (c, tree.get_context(c.node_id()).unwrap())
                    )
                    .find(|(_, t)|
                        t.element_data.state.contains(ElementState::Active)
                    )
                else { return warn!("scroll: no active?")};

                return self.handle_scroll_operation(
                    &ScrollOperation {
                        scroll_type: ScrollType::ScrollToNode(i.node_id())
                    },
                    tree
                );
            }

            ScrollType::ScrollToActive {
                include_children: true
            } => {
                let Some(active_id) = Self::find_nested_child(
                    tree,
                    self,
                    |tree, child| {
                        let ctx = tree.get_context(child.node_id())?;
                        Some(ctx.element_data.state.contains(ElementState::Active))
                    }
                ) else {
                    warn!("couldnt find nested active element");
                    return
                };

                return self.handle_scroll_operation(
                    &ScrollOperation {
                        scroll_type: ScrollType::ScrollToNode(active_id)
                    },
                    tree
                );
            }
        }

        self.validate_scroll_position(tree);
    }

    fn find_nested_child(
        tree: &Tree<actions::Action>,
        node: &dyn Widget<actions::Action>,
        op: fn(&Tree<actions::Action>, &dyn Widget<actions::Action>) -> Option<bool>,
    ) -> Option<NodeId> {
        for child in node.children() {
            if op(tree, child)? {
                return Some(child.node_id());
            }

            if let Some(res) = Self::find_nested_child(
                tree,
                child,
                op
            ) {
                return Some(res)
            }
        }

        None
    }
}

impl Widget<actions::Action> for Container {
    fn name(&self) -> CowStr { "container_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn children(&self) -> WidgetChildren<'_, actions::Action> {
        WidgetChildren::List(self.children.as_slice())
    }
    fn children_mut(&mut self) -> WidgetChildrenMut<'_, actions::Action> {
        WidgetChildrenMut::List(&mut self.children)
    }
    
    fn layout(&mut self, shell: &mut LayoutShell<actions::Action>) -> taffy::TaffyResult<NodeId> {
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
        shell: &mut InputShell<actions::Action>,
    ) {
        let Some(layout) = shell.tree.get_layout(self.node_id).copied()
        else { return };

        if let InputType::MouseScroll { raw: _, scroll: delta} = &event.event {
            if shell.event_consumed { return }
            if self.check_scroll(
                &ScrollPosition::Relative(*delta),
                &layout
            ) {
                let context = shell
                    .tree
                    .get_context_mut(self.node_id)
                    .unwrap();

                context.local_transform.pos = self.scroll_offset;
                shell.actions.push(actions::ui::UiAction::new(
                    self.node_id,
                    shell.source,
                    actions::ui::UiActionType::ContextChanged
                ).into());

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

        if !captured && self.scroll_direction.is_some() && self.drag_scroll {
            let offset = self
                .drag_scroll_data
                .check_input(self.node_id, shell, event);

            if self.check_scroll(&offset, &layout) {
                shell.event_consumed = true;
                let context = shell
                    .tree
                    .get_context_mut(self.node_id)
                    .unwrap();

                context.local_transform.pos = self.scroll_offset;
                shell.actions.push(actions::ui::UiAction::new(
                    self.node_id,
                    shell.source,
                    actions::ui::UiActionType::ContextChanged
                ).into());
            }
        }
    }

    fn update(&mut self, shell: &mut UpdateShell<actions::Action>) {
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
                    let mut resolver = CssResolver::new(&style, shell.default_css);

                    for _ in 0..diff.abs() {
                        // create the new element
                        let mut e = data.template.build();

                        let mut layout_shell = LayoutShell {
                            tree: shell.tree,
                            values: shell.values,
                            source: shell.source,
                            ui_scale: 1.0, // TODO:!
                            resolver: &mut resolver,
                            text_layout_contexts: shell.text_layout_contexts,
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
                        self.children.push(e.boxed());
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
                (1..) => {
                    // too many elements, remove some
                    for _ in 0..diff.abs() {
                        // remove it from our list
                        let removed = self
                            .children
                            .swap_remove(0);

                        // remove it from the tree
                        shell.tree.remove(removed.node_id());
                    }

                    // mark our node as dirty in the tree
                    shell.tree.mark_dirty(self.node_id);
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

    fn draw(&self, shell: &mut DrawShell<actions::Action>) {
        let Some(our_bounds) = shell.tree.absolute_bounds(self.node_id)
        else { return };

        let mut list = graphics::RenderableCollection::default();
        if self.scroll_direction.is_some() {
            std::mem::swap(shell.list, &mut list);
        }

        for i in self.children.iter() {
            // dont attempt to draw items outside our bounds
            let Some(ibounds) = shell.tree.absolute_bounds(i.node_id())
            else { continue };
            if our_bounds.intersection(ibounds).is_none() { continue }
            i.draw(shell);
        }

        if self.scroll_direction.is_some() {
            std::mem::swap(shell.list, &mut list);

            let elements = list
                .list
                .into_iter()
                .map(|element| graphics::Scissored::new(
                    our_bounds.into_scissor(),
                    element
                ))
                .map(|element| Box::new(element) as Box<dyn graphics::TatakuRenderable>);

            shell.list.list.extend(elements);
        }

        // TODO: draw scrollbar if scrollable
        // let Some(l) = shell.tree.get_layout(self).cloned() else { return };

        // // let bounds = Bounds::from(&l);
        // if l.scrollbar_size.has_non_zero_area() {
        // }
    }

    fn draw_overlay(&self, shell: &mut DrawShell<actions::Action>) {
        let Some(our_bounds) = shell.tree.absolute_bounds(self.node_id)
        else { return };

        let mut list = graphics::RenderableCollection::default();
        if self.scroll_direction.is_some() {
            std::mem::swap(shell.list, &mut list);
        }

        for i in self.children.iter() {
            // dont attempt to draw items outside our bounds
            let Some(ibounds) = shell.tree.absolute_bounds(i.node_id())
            else { continue };

            if our_bounds.intersection(ibounds).is_none() { continue }
            i.draw_overlay(shell);
        }

        if self.scroll_direction.is_some() {
            std::mem::swap(shell.list, &mut list);

            let elements = list.list.into_iter()
                .map(|element| graphics::Scissored::new(
                    our_bounds.into_scissor(),
                    element
                ))
                .map(|element| Box::new(element) as Box<dyn graphics::TatakuRenderable>);

            shell.list.list.extend(elements);
        }
    }

    fn handle_message(
        &mut self,
        message: &Message,
        shell: &mut MessageShell<actions::Action>,
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
        event: &input::TatakuEvent,
        event_value: Option<&TatakuValue>,
        shell: &mut MessageShell<actions::Action>,
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

    fn reload_skin(&mut self, shell: &mut UpdateShell<actions::Action>) {
        for i in self.children.iter_mut() {
            i.reload_skin(shell);
        }
    }

}



pub(super) enum ScrollPosition {
    None,

    /// move a relative amount
    Relative(Vector2),

    /// move to an absolute point (x and y are 0.0..1.0)
    Absolute(Vector2),
}
