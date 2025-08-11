use crate::prelude::*;
use crate::prelude::ui::*;

/// How many pixels of leniency should there be for resizing
const LENIENCY: f32 = 5.0;
#[derive(Copy, Clone, Default)]
struct ResizeHover {
    left: bool,
    right: bool,
    top: bool,
    bottom: bool,
}
impl ResizeHover {
    fn get_drag_origin(
        Self { 
            left, 
            right, 
            top, 
            bottom 
        }: Self
    ) -> Option<DragOrigin> {
        match (top, left, bottom, right) {
            (true, false, false, false) => Some(DragOrigin::Top),
            (false, true, false, false) => Some(DragOrigin::Left),
            (false, false, true, false) => Some(DragOrigin::Bottom),
            (false, false, false, true) => Some(DragOrigin::Right),
            (true, true, false, false) => Some(DragOrigin::TopLeft),
            (true, false, false, true) => Some(DragOrigin::TopRight),
            (false, true, true, false) => Some(DragOrigin::BottomLeft),
            (false, false, true, true) => Some(DragOrigin::BottomRight),
            _ => None,
        }
    }
}


pub struct DialogWidget {
    title: CowStr,
    node: Box<dyn Widget>,
    num: usize,

    draggable: bool,
    resizable: bool,
    resizing: Option<DragData>,

    resize_hover: ResizeHover,


    draw_background: bool,
}
impl DialogWidget {
    pub fn new(
        title: impl Into<CowStr>,
        draggable: bool,
        resizable: bool,
        draw_background: bool,
        inner: Box<dyn Widget>,
    ) -> Self {
        Self {
            title: title.into(),
            num: 0,
            // will get changed in layout
            draw_background,
            node: inner,
            resizable,
            draggable,
            resizing: None,
            resize_hover: ResizeHover::default(),
        }
    }



    fn resize_left(
        delta: f32,
        mut bounds: Bounds,
        node: NodeId,
        actions: &mut ActionQueue,
    ) {
        bounds.pos.x -= delta;
        bounds.size.x += delta;
        actions.push(UiAction::new(
            node, 
            DialogAction::MoveDialog(bounds.pos)
        ));
        actions.push(UiAction::new(
            node, 
            DialogAction::ResizeDialog(bounds.size)
        ));
    }
    fn resize_right(
        delta: f32,
        mut bounds: Bounds,
        node: NodeId,
        actions: &mut ActionQueue,
    ) {
        bounds.size.x -= delta;
        actions.push(UiAction::new(
            node, 
            DialogAction::ResizeDialog(bounds.size)
        ));
    }
    
    fn resize_up(
        delta: f32,
        mut bounds: Bounds,
        node: NodeId,
        actions: &mut ActionQueue,
    ) {
        bounds.pos.y -= delta;
        bounds.size.y += delta;
        actions.push(UiAction::new(
            node, 
            DialogAction::MoveDialog(bounds.pos)
        ));
        actions.push(UiAction::new(
            node, 
            DialogAction::ResizeDialog(bounds.size)
        ));
    }
    fn resize_down(
        delta: f32,
        mut bounds: Bounds,
        node: NodeId,
        actions: &mut ActionQueue,
    ) {
        bounds.size.y -= delta;
        actions.push(UiAction::new(
            node, 
            DialogAction::ResizeDialog(bounds.size)
        ));
    }


    // TODO: cache these maybe?
    fn left_bound(bounds: &Bounds) -> Bounds {
        Bounds::new(
            Vector2::new(
                bounds.pos.x - LENIENCY,
                bounds.pos.y,
            ),
            Vector2::new(
                LENIENCY * 2.0,
                bounds.size.y
            )
        )
    }
    fn right_bound(bounds: &Bounds) -> Bounds {
        Bounds::new(
            Vector2::new(
                bounds.pos.x + bounds.size.x - LENIENCY,
                bounds.pos.y,
            ),
            Vector2::new(
                LENIENCY * 2.0,
                bounds.size.y
            )
        )
    }
    fn top_bound(bounds: &Bounds) -> Bounds {
        Bounds::new(
            Vector2::new(
                bounds.pos.x,
                bounds.pos.y - LENIENCY,
            ),
            Vector2::new(
                bounds.size.x,
                LENIENCY * 2.0,
            )
        )
    }
    fn bottom_bound(bounds: &Bounds) -> Bounds {
        Bounds::new(
            Vector2::new(
                bounds.pos.x,
                bounds.pos.y + bounds.size.y - LENIENCY,
            ),
            Vector2::new(
                bounds.size.x,
                LENIENCY * 2.0,
            )
        )
    }

    fn check_left(
        &mut self,
        bounds: &Bounds,
        mouse_pos: Vector2,
    ) {
        self.resize_hover.left = Self::left_bound(bounds)
            .contains(mouse_pos);
    }

    fn check_right(
        &mut self,
        bounds: &Bounds,
        mouse_pos: Vector2,
    ) {
        self.resize_hover.right = Self::right_bound(bounds)
            .contains(mouse_pos);
    }

    fn check_top(
        &mut self,
        bounds: &Bounds,
        mouse_pos: Vector2,
    ) {
        self.resize_hover.top = Self::top_bound(bounds)
            .contains(mouse_pos);
    }
    fn check_bottom(
        &mut self,
        bounds: &Bounds,
        mouse_pos: Vector2,
    ) {
        self.resize_hover.bottom = Self::bottom_bound(bounds)
            .contains(mouse_pos);
    }
}
impl Widget for DialogWidget {
    fn name(&self) -> CowStr { self.node.name() }
    fn node_id(&self) -> NodeId { self.node.node_id() }

    fn layout(&mut self, shell: &mut LayoutShell) -> TaffyResult<NodeId> {
        let node = std::mem::replace(
            &mut self.node, 
            EmptyWidget::new_boxed()
        );

        let children = if self.draggable || self.resizable {
            vec![
                DialogTitlebar::new(self.title.clone(), self.draggable).boxed(),
                node,
            ]
        } else {
            vec![node]
        };
        self.node = Container::new(children)
            // .flex_direction(FlexDirection::Column)
            // .width(FILL)
            // .height(FILL)
            .boxed();


        self.node.layout(shell)
    }
    
    // fn update_styles(
    //     &mut self, 
    //     shell: &mut StyleShell,
    //     _display_override: Option<DisplayType>
    // ) {
    //     self.node.update_styles(shell, None);
    // }
    
    fn input(
        &mut self, 
        event: &InputEvent, 
        shell: &mut InputShell,
    ) {
        self.node.input(event, shell);
        if shell.event_consumed {
            return;
        }

        let node_id = self.node_id();

        let Some(bounds) = shell.tree.absolute_bounds(node_id) 
        else { return };

        // if this was a mouse input and its inside our bounds
        // we should always consume the event
        if event.is_mouse() && bounds.contains(event.mouse_pos) {
            shell.event_consumed = true;
        }

        // resize 
        if !self.resizable {
            return
        }

        match (&event.event, self.resizing) {
            (InputType::MouseMove(pos), Some(drag)) => {
                let delta = drag.mouse_pos_start - *pos;
                self.resizing = Some(DragData {
                    mouse_pos_start: *pos,
                    ..drag
                });
                    
                match drag.origin {
                    DragOrigin::Left => {
                        self.check_left(&bounds, *pos);
                        Self::resize_left(delta.x, bounds, node_id, shell.actions);
                    }
                    DragOrigin::Right => {
                        self.check_right(&bounds, *pos);
                        Self::resize_right(delta.x, bounds, node_id, shell.actions);
                    }
                    DragOrigin::Top => {
                        self.check_top(&bounds, *pos);
                        Self::resize_up(delta.y, bounds, node_id, shell.actions);
                    }
                    DragOrigin::Bottom => {
                        self.check_bottom(&bounds, *pos);
                        Self::resize_down(delta.y, bounds, node_id, shell.actions);
                    }

                    DragOrigin::BottomLeft => {
                        self.check_left(&bounds, *pos);
                        self.check_bottom(&bounds, *pos);
                        Self::resize_down(delta.y, bounds, node_id, shell.actions);
                        Self::resize_left(delta.x, bounds, node_id, shell.actions);
                    }
                    DragOrigin::BottomRight => {
                        self.check_bottom(&bounds, *pos);
                        self.check_right(&bounds, *pos);
                        Self::resize_down(delta.y, bounds, node_id, shell.actions);
                        Self::resize_right(delta.x, bounds, node_id, shell.actions);
                    }
                    DragOrigin::TopLeft => {
                        self.check_top(&bounds, *pos);
                        self.check_left(&bounds, *pos);
                        Self::resize_up(delta.y, bounds, node_id, shell.actions);
                        Self::resize_left(delta.x, bounds, node_id, shell.actions);
                    }
                    DragOrigin::TopRight => {
                        self.check_top(&bounds, *pos);
                        self.check_right(&bounds, *pos);
                        Self::resize_down(delta.y, bounds, node_id, shell.actions);
                        Self::resize_right(delta.x, bounds, node_id, shell.actions);
                    }
                }
            }
            (InputType::MouseMove(pos), _) => {
                self.check_top(&bounds, *pos);
                self.check_bottom(&bounds, *pos);
                self.check_left(&bounds, *pos);
                self.check_right(&bounds, *pos);
            }

            (InputType::MousePress(MouseButton::Left), None) => {
                if let Some(origin) = ResizeHover::get_drag_origin(
                    self.resize_hover
                ) {
                    shell.event_consumed = true;
                    self.resizing = Some(DragData { 
                        pos_start: Vector2::ZERO, // doesnt matter for resize 
                        mouse_pos_start: event.mouse_pos, 
                        origin,
                    });
                }
            }
            (InputType::MouseRelease(MouseButton::Left), Some(_)) => {
                self.resizing = None;
                shell.event_consumed = true;
            }

            _ => {}
        }
    }
    
    fn operation(
        &mut self, 
        operation: &UiOperation, 
        tree: &mut Tree,
    ) {
        self.node.operation(operation, tree);
    }
    
    fn draw(&self, shell: &mut DrawShell) {
        let Some(bounds) = shell.tree.absolute_bounds(self) 
        else { return };

        if self.draw_background {
            shell.list.push(Blur::new(bounds, BlurType::Box { size: 2 }));

            // black background for visibility
            shell.list.push(Rectangle::new_bounds(
                bounds, 
                Color::BLACK.alpha(0.9), 
            ));
        }

        // FIXME: add scissor!
        self.node.draw(shell);
        if !self.resizable { return }

        let color = Color::CRIMSON;
        if self.resize_hover.left {
            shell.list.push(Rectangle::new_bounds(
                Self::left_bound(&bounds), 
                color, 
            ));
        }
        if self.resize_hover.right {
            shell.list.push(Rectangle::new_bounds(
                Self::right_bound(&bounds), 
                color, 
            ));
        }
        if self.resize_hover.top {
            shell.list.push(Rectangle::new_bounds(
                Self::top_bound(&bounds), 
                color, 
            ));
        }
        if self.resize_hover.bottom {
            shell.list.push(Rectangle::new_bounds(
                Self::bottom_bound(&bounds), 
                color,
            ));
        }
    }
    
    fn draw_overlay(&self, shell: &mut DrawShell) {
        self.node.draw_overlay(shell);
    }
    
    fn update(&mut self, shell: &mut UpdateShell) {
        self.node.update(shell);
    }
    
    fn handle_message(
        &mut self, 
        message: &Message, 
        shell: &mut MessageShell,
    ) {
        match message.owner {
            MessageOwner::Menu => return,
            MessageOwner::Dialog(num) => {
                if &**message.tag == "set_num" {
                    if let MessageValue::Number(n) = message.value {
                        self.num = n;
                        return;
                    }
                }
                
                if num != self.num { return }
            }
        }

        self.node.handle_message(
            message, 
            shell,
        );

        if shell.handled { return }
        match &**message.tag {
            "close" 
            | "force_close"
            => {
                debug!("close request");
                shell.actions.push(UiAction::new(
                    self.node_id(),
                    DialogAction::Close,
                ));
            }

            _ => {}
        }
    }
    
    fn handle_event(
        &mut self, 
        event: &TatakuEventType, 
        event_value: Option<&TatakuValue>, 
        shell: &mut MessageShell,
    ) {
        self.node.handle_event(event, event_value, shell);
    }
    
    fn reload_skin(&mut self, shell: &mut UpdateShell) {
        self.node.reload_skin(shell);
    }
    
}


#[derive(Copy, Clone)]
struct DragData {
    pos_start: Vector2,
    mouse_pos_start: Vector2,
    origin: DragOrigin,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum DragOrigin {
    Left,
    Right,
    Top,
    Bottom,

    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

struct DialogTitlebar {
    title: CowStr,
    draggable: bool,
    node: Box<dyn Widget>,

    drag: Option<DragData>,
}
impl DialogTitlebar {
    fn new(
        title: impl Into<CowStr>,
        draggable: bool,
    ) -> Self {
        Self {
            title: title.into(),
            draggable,
            node: EmptyWidget::new_boxed(),
            drag: None,
        }
    }

    fn get_titlebar(&self) -> Element {
        let title = &self.title;
        let a = format!(r#"
        <row style="width: fill">
            <!-- Title -->
            <text style="font_size: 40.0"> 
                <text text="{title}"/> 
            </text>

            <!-- Close Button -->
            <button>
                <action>
                    <closeDialog />
                </action>
                <element>
                    <text style="font_size: 20.0">
                        <text text="X" />
                    </text>
                </element>
            </button>
        </row>
        "#);
        quick_xml::de::from_str(&a).unwrap()
    }
}
impl Widget for DialogTitlebar {
    fn name(&self) -> CowStr { "titlebar_widget".into() }
    fn node_id(&self) -> NodeId { self.node.node_id() }

    fn layout(&mut self, shell: &mut LayoutShell) -> TaffyResult<NodeId> {
        // self.node = Container::new(vec![
        //     // Title 
        //     TextWidget::new(&*self.title).font_size(40.0).boxed(),
            
        //     // close button
        //     Button::new(Box::new(TextWidget::new("X").font_size(20.0)))
        //         // .padding(MeasurableUnit::Pixels(5.0))
        //         .on_press(Message::new(shell.owner, "close", MessageValue::Click))
        //         .boxed()
        // ])
        // .width(FILL)
        // // .padding(MeasurableUnit::Pixels(5.0))
        // .flex_direction(FlexDirection::Row)
        // // .horizontal_align(AlignContent::SpaceBetween)
        // .boxed();

        self.node = self.get_titlebar().build();
        self.node.layout(shell)
    }

    fn draw(&self, shell: &mut DrawShell) {
        let Some(bounds) = shell.tree.absolute_bounds(self.node_id()) 
        else { return };

        shell.list.push(Rectangle::new_bounds(
            bounds, 
            Color::WHITE.alpha(0.5), 
        ));
        self.node.draw(shell);
    }
    fn draw_overlay(&self, shell: &mut DrawShell) {
        self.node.draw_overlay(shell);
    }

    fn update(&mut self, shell: &mut UpdateShell) {
        self.node.update(shell);
    }

    fn reload_skin(&mut self, shell: &mut UpdateShell) {
        self.node.reload_skin(shell);
    }

    fn input(
        &mut self, 
        event: &InputEvent, 
        shell: &mut InputShell,
    ) {
        self.node.input(event, shell);

        if shell.event_consumed || !self.draggable {
            return
        }

        let node_id = self.node_id();
        match (&event.event, &mut self.drag) {
            (InputType::MouseMove(pos), Some(drag)) => {
                shell.actions.push(UiAction::new(
                    node_id,
                    DialogAction::MoveDialog(
                        drag.pos_start + (*pos - drag.mouse_pos_start)
                    )
                ));
            }

            (InputType::MousePress(MouseButton::Left), _) => {
                let Some(bounds) = shell.tree.absolute_bounds(node_id) 
                else { return };

                if bounds.contains(event.mouse_pos) {
                    shell.event_consumed = true;
                    self.drag = Some(DragData { 
                        pos_start: bounds.pos, 
                        mouse_pos_start: event.mouse_pos, 
                        origin: DragOrigin::Left, // doesnt matter for movement
                    });
                }
            }
            (InputType::MouseRelease(MouseButton::Left), Some(_)) => {
                self.drag = None;
                shell.event_consumed = true;
            }

            _ => {}
        }
    }
}
