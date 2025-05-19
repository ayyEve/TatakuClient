use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(Widget)]
#[widget(type("container"))]
#[derive(ChainableInitializer)]
pub struct Container {
    #[chain] pub style: Style,
    pub children: Vec<Box<dyn Widget>>,

    #[chain] id: Cow<'static, str>,
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
    pub fn new(items: Vec<Box<dyn Widget>>) -> Self {
        Self {
            style: Style::DEFAULT,
            children: items,
            node_id: EMPTY_NODE,
            scrollable: false,

            drag_scroll: false,
            programmatic: None,
            drag_scroll_data: Default::default(),
            scroll_offset: Vector2::ZERO,
            id: "".into(),
        }
    }

    fn check_scroll(
        &mut self, 
        scroll: ScrollPosition,
        layout: &Layout,
    ) -> bool {
        if !self.scrollable { return false }
        match scroll {
            ScrollPosition::None => false,
            ScrollPosition::Relative(delta) => {
                let new_scroll = (self.scroll_offset.y + delta.y)
                    .clamp(0.0, layout.scroll_height());

                if (self.scroll_offset.y - new_scroll).abs() > f32::EPSILON {
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
                self.scroll_offset = (size * pos).clamp(Vector2::ZERO, size);
                true
            }
        }

    }

    pub fn make_programmatic(mut self, data: ProgrammaticListData) -> Self {
        self.programmatic = Some(data);
        self
    }
}

impl Widget for Container {
    fn name(&self) -> Cow<'static, str> { "container_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn update_styles(
        &mut self,
        shell: &mut StyleShell,
        _display_override: Option<ui::Display>) {
        for i in self.children.iter_mut() {
            i.update_styles(shell, None);  
        }
    }

    fn input(
        &mut self, 
        event: &InputEvent,
        shell: &mut InputShell,
    ) {
        let Some(layout) = shell.tree.get_layout(&*self).cloned() else { return };

        if let InputType::MouseScroll(delta) = &event.event {
            if shell.event_consumed { return }
            if self.check_scroll(ScrollPosition::Relative(Vector2::new(0.0, *delta)), &layout) {
                let context = shell.tree.get_context_mut(&*self).unwrap();
                // TODO: this is backwards, but if its = scroll_offset then it scrolls opposite of the content
                context.local_transform.pos = -self.scroll_offset;
                shell.actions.push(UiAction::new(self.node_id, UiActionType::ContextChanged));

                shell.event_consumed = true;
                return 
            }
        }

        
        let mut captured = false;
        if let Some(data) = &mut self.programmatic {
            let Ok(iter) = shell.values.reflect_iter(&data.list_var) else {
                if !data.error_printed {
                    data.error_printed = true;
                    error!("!!!!!!!!!!!!!!");
                    error!("list variable doesnt exist! {}", data.list_var);
                    error!("!!!!!!!!!!!!!!");
                }
                return 
            };
    
            let values = iter 
                .filter_map(|v| v.duplicate())
                .collect::<Vec<_>>();
    
            let path = ReflectPath::new(&data.variable);
            for (w, value) in self.children.iter_mut().zip(values) {
                shell.values.impl_insert(path.clone(), value).expect("error inserting into values");
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
            let offset = self.drag_scroll_data.check_input(self.node_id, shell, event);
            if self.check_scroll(offset, &layout) {
                shell.event_consumed = true;
                let context = shell.tree.get_context_mut(&*self).unwrap();
                context.local_transform.pos = -self.scroll_offset;
                shell.actions.push(UiAction::new(self.node_id, UiActionType::ContextChanged));
            }
        }
    }

    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        let children = self.children.iter_mut()
            .map(|w| w.layout(shell))
            .collect::<TaffyResult<Vec<_>>>()?;

        self.node_id = shell.tree.new_with_children(
            self.style.clone(), 
            &children
        )?;

        shell.with_context(self.node_id, |ctx| {
            ctx.needs_inverse_transform = true;
        });

        Ok(self.node_id)
    }

    fn draw(&self, shell: &mut DrawShell) {
        let Some(our_bounds) = shell.tree.absolute_bounds(self) else { return };

        let mut list = RenderableCollection::new();
        if self.scrollable {
            std::mem::swap(shell.list, &mut list);
        }

        for i in self.children.iter() {
            // dont attempt to draw items outside our bounds
            let Some(ibounds) = shell.tree.absolute_bounds(i.node_id()) else { continue };
            if our_bounds.intersection(ibounds).is_none() { continue }
            i.draw(shell);
        }
        
        if self.scrollable {
            std::mem::swap(shell.list, &mut list);
            shell.list.push(ScissoredDrawable::new(
                our_bounds.into_scissor(),
                Box::new(TransformGroup::from_collection(Vector2::ZERO, list))
            ));
        }

        // TODO: draw scrollbar if scrollable
        // let Some(l) = shell.tree.get_layout(self).cloned() else { return };

        // // let bounds = Bounds::from(&l);
        // if l.scrollbar_size.has_non_zero_area() {
        // }
    }
    
    fn draw_overlay(&self, shell: &mut DrawShell) {
        let Some(our_bounds) = shell.tree.absolute_bounds(self) else { return };

        let mut list = RenderableCollection::new();
        if self.scrollable {
            std::mem::swap(shell.list, &mut list);
        }

        for i in self.children.iter() {
            // dont attempt to draw items outside our bounds
            let Some(ibounds) = shell.tree.absolute_bounds(i.node_id()) else { continue };
            if our_bounds.intersection(ibounds).is_none() { continue }
            i.draw_overlay(shell)
        }
        
        if self.scrollable {
            std::mem::swap(shell.list, &mut list);
            shell.list.push(ScissoredDrawable::new(
                our_bounds.into_scissor(),
                Box::new(TransformGroup::from_collection(Vector2::ZERO, list))
            ));
        }
    }

    fn update(&mut self, shell: &mut UpdateShell) {
        if let Some(data) = &mut self.programmatic {

            let Ok(iter) = shell.values.reflect_iter(&data.list_var) else {
                if !data.error_printed {
                    error!("!!!!!!!!!!!!!!");
                    error!("list variable doesnt exist! {}", data.list_var);
                    error!("!!!!!!!!!!!!!!");
                }
                return 
            };

            let values = iter  
                .filter_map(|v| v.duplicate())
                .collect::<Vec<_>>();

            // FIXME: need to reload skin for added children
            // make sure our list of children is the same length as the list of values
            match self.children.len() as i64 - values.len() as i64 {
                0 => {} // children and values are the same length, nothing to do
                diff @ (..0) => {
                    // children is too small, need to add elements
                    let mut layout_shell = LayoutShell {
                        tree: shell.tree,
                        values: shell.values,
                        owner: shell.owner,
                        ui_scale: 1.0, // TODO:!
                    };

                    for _ in 0..diff.abs() {
                        // create the new element
                        let mut e = data.template.build();

                        // add it to the tree
                        let child = match e.layout(&mut layout_shell) {
                            Ok(n) => n,
                            Err(e) => panic!("Error laying out new custom list child! {e}"), // TODO: not panic?
                        };

                        // make us its parent
                        layout_shell.tree.add_child(self.node_id, child);

                        // add to our list
                        self.children.push(e);
                    }
                    
                    // mark the tree as dirty
                    shell.actions.push(UiAction::new(self.node_id, UiActionType::MarkDirty));
                    shell.actions.push(UiAction::new(self.node_id, UiActionType::Refresh));
                }
                diff @ (0..) => {
                    // too many elements, remove some
                    for _ in 0..diff.abs() {
                        // remove it from our list
                        let removed = self.children.swap_remove(0);

                        // remove it from the tree
                        shell.tree.remove(removed.node_id()); //.expect("failed to remove child from tree");
                    }
                
                    // mark the tree as dirty
                    shell.actions.push(UiAction::new(self.node_id, UiActionType::MarkDirty));
                    shell.actions.push(UiAction::new(self.node_id, UiActionType::Refresh));
                }
            }
            
            let path = ReflectPath::new(&data.variable);
            for (i, value) in self.children.iter_mut().zip(values) {
                shell.values.impl_insert(path.clone(), value).expect("error inserting into values");
                i.update(shell);
            }

        } else {
            for i in self.children.iter_mut() {
                i.update(shell)
            }
        }
    }

    fn handle_message(
        &mut self, 
        message: &Message, 
        shell: &mut MessageShell,
    ) {
        if let Some(data) = &mut self.programmatic {
            let Ok(iter) = shell.values.reflect_iter(&data.list_var) else {
                if !data.error_printed {
                    data.error_printed = true;
                    error!("!!!!!!!!!!!!!!");
                    error!("list variable doesnt exist! {}", data.list_var);
                    error!("!!!!!!!!!!!!!!");
                }
                return 
            };
    
            let values_ = iter 
                .filter_map(|v| v.duplicate())
                .collect::<Vec<_>>();
    
            let path = ReflectPath::new(&data.variable);
            for (i, value) in self.children.iter_mut().zip(values_) {
                shell.values.impl_insert(path.clone(), value).expect("error inserting into values");
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
        event: TatakuEventType, 
        event_value: Option<TatakuValue>, 
        shell: &mut MessageShell,
    ) {
        if let Some(data) = &mut self.programmatic {
            let Ok(iter) = shell.values.reflect_iter(&data.list_var) else {
                if !data.error_printed {
                    data.error_printed = true;
                    error!("!!!!!!!!!!!!!!");
                    error!("list variable doesnt exist! {}", data.list_var);
                    error!("!!!!!!!!!!!!!!");
                }
                return 
            };
    
            let values_ = iter 
                .filter_map(|v| v.duplicate())
                .collect::<Vec<_>>();
    
            let path = ReflectPath::new(&data.variable);
            for (i, value) in self.children.iter_mut().zip(values_) {
                shell.values.impl_insert(path.clone(), value).expect("error inserting into values");
                i.handle_event(event, event_value.clone(), shell);
            }
        } else {
            for i in self.children.iter_mut() {
                i.handle_event(event, event_value.clone(), shell)
            }
        }
    }

    fn reload_skin(&mut self, shell: &mut UpdateShell) {
        for i in self.children.iter_mut() {
            i.reload_skin(shell);
        }
    }
}

#[derive(ChainableInitializer)]
pub struct ProgrammaticListData {
    /// what element to build for each iteration
    #[chain] pub template: Element,

    /// what variable to iterate over
    #[chain] pub list_var: String,

    /// what var name to store the iter variable in (ie the `i` in `for i in ...`)
    #[chain] pub variable: String,

    error_printed: bool,
}
impl ProgrammaticListData {
    pub fn new(template: Element, list_var: String, variable: String) -> Self {
        Self {
            template,
            list_var,
            variable,
            error_printed: false,
        }
    }
}



mod macros {
    // idk why this says its unused, if i remove it everything cries
    #[allow(unused)]
    use crate::prelude::*;

    #[macro_export]
    macro_rules! row {
        ($($i:expr),*;$($t:ident = $v:expr),*) => {
            Container::new(vec![
                $(
                    $i,
                )*
            ])
            $(
                .$t($v)
            )*
            .flex_direction(FlexDirection::Row)
            .boxed()
        };

        ($vec:expr, $($t:ident = $v:expr),*) => {
            Container::new($vec)
            $(
                .$t($v)
            )*
            .flex_direction(FlexDirection::Row)
            .boxed()
        }
    }

    #[macro_export]
    macro_rules! col {
        ($($i:expr),*;$($t:ident = $v:expr),*) => {
            Container::new(vec![
                $(
                    $i,
                )*
            ])
            $(
                .$t($v)
            )*
            .flex_direction(FlexDirection::Column)
            .boxed()
        };

        ($vec:expr, $($t:ident = $v:expr),*) => {
            Container::new($vec)
            $(
                .$t($v)
            )*
            .flex_direction(FlexDirection::Column)
            .boxed()
        }
    }

}


macro_rules! make_rect_helper {
    ($name: ident, $ty: ty) => {
        pub struct $name(pub taffy::Rect<$ty>);
        impl From<$ty> for $name {
            fn from(value: $ty) -> Self {
                Self(taffy::Rect {
                    top: value,
                    left: value,
                    bottom: value,
                    right: value,
                })
            }
        }
        impl From<[$ty; 2]> for $name {
            fn from(value: [$ty; 2]) -> Self {
                Self(taffy::Rect {
                    top: value[0],
                    bottom: value[0],
                    left: value[1],
                    right: value[1],
                })
            }
        }
        impl From<[$ty; 4]> for $name {
            fn from(value: [$ty; 4]) -> Self {
                Self(taffy::Rect {
                    top: value[0],
                    left: value[1],
                    bottom: value[2],
                    right: value[3],
                })
            }
        }
    }
}
make_rect_helper!(Margin, LengthPercentageAuto);
make_rect_helper!(Padding, LengthPercentage);

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
        shell: &mut InputShell<'_>,
        event: &InputEvent,
    ) -> ScrollPosition {
        let Some(bounds) = shell.tree.absolute_bounds(node_id) else { return ScrollPosition::None }; 
        let hover = bounds.contains(shell.mouse_pos);

        match event.event {
            InputType::MousePress(b) if hover => {
                match b {
                    MouseButton::Left if !self.right_pressed => self.left_pressed = true,
                    MouseButton::Right if !self.left_pressed => self.right_pressed = true,
                    _ => return ScrollPosition::None,
                }

                self.pressed_at = shell.mouse_pos;
            }

            InputType::MouseRelease(b) => {
                match b {
                    MouseButton::Left if self.left_pressed => self.left_pressed = false,
                    MouseButton::Right if self.right_pressed => self.right_pressed = false,
                    _ => return ScrollPosition::None
                }
                // if the mouse moved, we dont want to register the release key, so return that it was consumed
                self.did_move = false;

                // TODO: dont use this hack lmao
                return ScrollPosition::Relative(Vector2::ZERO);
            }
            InputType::MouseMove(position) if hover => {
                if !self.did_move && (self.left_pressed || self.right_pressed) && position.distance(self.pressed_at) > DRAG_THRESHOLD {
                    self.did_move = true;
                }

                // check left click 
                if self.left_pressed && self.did_move {
                    // let diff = AbsoluteOffset {
                    //     x: -(position.x - self.pressed_at.x),
                    //     y: -(position.y - self.pressed_at.y),
                    // };
                    let diff = (position - self.pressed_at) * -1.0;

                    // reset the clicked pos to move the delta
                    self.pressed_at = position;

                    // perform scroll
                    return ScrollPosition::Relative(diff)
                } else 

                // check right click
                if self.right_pressed && self.did_move {
                    let pos = bounds.pos;
                    let size = bounds.size;

                    let move_to = ((position - pos) / size).clamp(Vector2::ZERO, Vector2::ONE);
                    
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
