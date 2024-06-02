use crate::prelude::*;
use iced::widget::Scrollable;
use iced::{
    Pixels,
    Length,
    Padding,
    Alignment,

    mouse,
    overlay,
    event::{ self, Event },
};

use iced::advanced::{
    Shell,
    Clipboard,
    layout::{ self, Layout },
    widget::{
        Tree,
        Widget,
        Operation,
    },
};

/// how far the
const DRAG_THRESHOLD:f32 = 5.0;

pub fn make_scrollable(children: Vec<IcedElement>, id: &'static str) -> Scrollable<'static, Message, IcedRenderer> {
    Scrollable::new(CullingColumn::with_children(children).spacing(5.0).into_element()).id(iced::widget::scrollable::Id::new(id))
}
pub fn make_panel_scroll(children: Vec<IcedElement>, id: &'static str) -> PanelScroll {
    PanelScroll::with_children(vec![make_scrollable(children, id).into_element()])
        // .id(iced::advanced::widget::Id::new(id))
        .spacing(5.0)
}

/// a scrollable area which culls items outside of bounds (i dont know why iced doesnt do this already :///)
pub struct CullingColumn {
    width: Length,
    height: Length,
    max_width: f32,
    spacing: f32,
    padding: Padding,
    align_items: Alignment,
    children: Vec<IcedElement>,
}

#[allow(unused)]
impl CullingColumn {
    /// Creates an empty [`CullingColumn`].
    pub fn new() -> Self {
        Self::with_children(Vec::new())
    }

    /// Creates a [`CullingColumn`] with the given elements.
    pub fn with_children(children: Vec<IcedElement>) -> Self {
        Self {
            spacing: 0.0,
            padding: Padding::ZERO,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: f32::INFINITY,
            align_items: Alignment::Start,
            children,
        }
    }

    /// Sets the vertical spacing _between_ elements.
    ///
    /// Custom margins per element do not exist in iced. You should use this
    /// method instead! While less flexible, it helps you keep spacing between
    /// elements consistent.
    pub fn spacing(mut self, amount: impl Into<Pixels>) -> Self {
        self.spacing = amount.into().0;
        self
    }

    /// Sets the [`Padding`] of the [`ActualScroll`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the [`ActualScroll`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`ActualScroll`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the maximum width of the [`ActualScroll`].
    pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
        self.max_width = max_width.into().0;
        self
    }

    /// Sets the horizontal alignment of the contents of the [`ActualScroll`] .
    pub fn align_items(mut self, align: Alignment) -> Self {
        self.align_items = align;
        self
    }

    /// Adds an element to the [`ActualScroll`].
    pub fn push(
        mut self,
        child: IcedElement,
    ) -> Self {
        self.children.push(child);
        self
    }
}

impl Default for CullingColumn {
    fn default() -> Self {
        Self::new()
    }
}


impl Widget<Message, IcedRenderer> for CullingColumn {
    fn children(&self) -> Vec<Tree> { self.children.iter().map(Tree::new).collect() }
    fn diff(&self, tree: &mut Tree) { tree.diff_children(&self.children); }
    fn width(&self) -> Length { self.width }
    fn height(&self) -> Length { self.height }


    fn layout(
        &self,
        renderer: &IcedRenderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits
            .max_width(self.max_width)
            .width(self.width)
            .height(self.height);

        layout::flex::resolve(
            layout::flex::Axis::Vertical,
            renderer,
            &limits,
            self.padding,
            self.spacing,
            self.align_items,
            &self.children,
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &IcedRenderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.children
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget()
                        .operate(state, layout, renderer, operation);
                })
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &IcedRenderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) -> event::Status {
        self.children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &IcedRenderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget().mouse_interaction(
                    state, layout, cursor, viewport, renderer,
                )
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut IcedRenderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style, //renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let old = renderer.start_layer();

        for ((child, state), layout) in 
            self.children.iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            if !viewport.intersects(&layout.bounds()) { continue }

            child
                .as_widget()
                .draw(state, renderer, theme, style, layout, cursor, viewport);
        }

        renderer.end_layer(old, layout.bounds());
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &IcedRenderer,
    ) -> Option<overlay::Element<'b, Message, IcedRenderer>> {
        iced::advanced::overlay::from_children(&mut self.children, tree, layout, renderer)
    }
}

impl From<CullingColumn> for IcedElement {
    fn from(value: CullingColumn) -> Self {
        Self::new(value)
    }
}


/// runs all events for all items in the list
/// also enabled left and right-click drag scrolling
pub struct PanelScroll {
    children: Vec<IcedElement>,
    width: Length,
    height: Length,
    id: Option<iced::advanced::widget::Id>,
    axis: layout::flex::Axis,

    max_height: f32,
    spacing: f32,
    padding: Padding,
    align_items: Alignment,
}

#[allow(unused)]

/// this lays out like a row
impl PanelScroll {
    /// Creates an empty [`PanelScroll`].
    pub fn new() -> Self {
        Self::with_children(Vec::new())
    }

    /// Creates a [`PanelScroll`] with the given elements.
    pub fn with_children(
        children: Vec<IcedElement>,
    ) -> Self {
        Self {
            width: Length::Fill,
            height: Length::Fill,
            max_height: f32::INFINITY,
            axis: layout::flex::Axis::Horizontal,
            spacing: 0.0,
            padding: Padding::ZERO,
            align_items: Alignment::Start,
            id: None,
            children,
        }
    }

    /// Sets the horizontal spacing _between_ elements.
    ///
    /// Custom margins per element do not exist in iced. You should use this
    /// method instead! While less flexible, it helps you keep spacing between
    /// elements consistent.
    pub fn spacing(mut self, amount: impl Into<Pixels>) -> Self {
        self.spacing = amount.into().0;
        self
    }

    /// Sets the [`Padding`] of the [`PanelScroll`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the [`PanelScroll`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`PanelScroll`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the maximum width of the [`PanelScroll`].
    pub fn max_height(mut self, max_height: impl Into<Pixels>) -> Self {
        self.max_height = max_height.into().0;
        self
    }

    /// Sets the vertical alignment of the contents of the [`PanelScroll`] .
    pub fn align_items(mut self, align: Alignment) -> Self {
        self.align_items = align;
        self
    }

    pub fn id(mut self, id: iced::advanced::widget::Id) -> Self {
        self.id = Some(id);
        self
    }
    pub fn axis(mut self, axis: layout::flex::Axis) -> Self {
        self.axis = axis;
        self
    }

    /// Adds an element to the [`PanelScroll`].
    pub fn push(
        mut self,
        child: IcedElement,
    ) -> Self {
        self.children.push(child);
        self
    }

    fn operate_on_children(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &IcedRenderer,
        operation: &mut dyn Operation<Message>
    ) {
        self.children
            .iter()
            .zip(&mut tree.children)
            .zip(layout.children())
            .for_each(|((child, state), layout)| 
                child.as_widget().operate(
                    state, 
                    layout,
                    renderer,
                    operation
                )
        )
    }


    fn handle_mouse_event(
        &self,
        tree: &mut Tree,
        cursor_pos: iced::Point,
        event: &mouse::Event,
        layout: Layout<'_>,
        renderer: &IcedRenderer,
    ) -> bool {
        let state = tree.state.downcast_mut::<PanelState>();
        let hover = layout.bounds().contains(cursor_pos);

        match event {
            mouse::Event::ButtonPressed(b) if hover => {
                match b {
                    mouse::Button::Left if !state.right_pressed => state.left_pressed = true,
                    mouse::Button::Right if !state.left_pressed => state.right_pressed = true,
                    _ => return false
                }

                state.pressed_at = cursor_pos;
            }

            mouse::Event::ButtonReleased(b) => {
                match b {
                    mouse::Button::Left if state.left_pressed => state.left_pressed = false,
                    mouse::Button::Right if state.right_pressed => state.right_pressed = false,
                    _ => return false
                }
                // if the mouse moved, we dont want to register the release key, so return that it was consumed
                return std::mem::take(&mut state.did_move)
            }
            mouse::Event::CursorMoved { position } if hover => {
                if !state.did_move && (state.left_pressed || state.right_pressed) && position.distance(state.pressed_at) > DRAG_THRESHOLD {
                    state.did_move = true;
                }

                // check left click 
                if state.left_pressed && state.did_move {
                    let diff = AbsoluteOffset {
                        x: -(position.x - state.pressed_at.x),
                        y: -(position.y - state.pressed_at.y),
                    };

                    // reset the clicked pos to move the delta
                    state.pressed_at = *position;

                    // perform scroll
                    let mut operation = scroll_to(diff, true);
                    self.operate_on_children(tree, layout, renderer, &mut operation);
                } else 

                // check right click
                if state.right_pressed && state.did_move {
                    let bounds = layout.bounds();
                    let pos = bounds.position();
                    let size = bounds.size();

                    let mut operation = snap_to(RelativeOffset {
                        x: ((position.x - pos.x) / size.width).clamp(0.0, 1.0),
                        y: ((position.y - pos.y) / size.height).clamp(0.0, 1.0)
                    });
                    self.operate_on_children(tree, layout, renderer, &mut operation);
                }
            }

            _ => {}
        }

        false
    }


    /// why is Axis not copy and clone ?????
    fn get_axis(&self) -> layout::flex::Axis {
        match &self.axis {
            layout::flex::Axis::Horizontal => layout::flex::Axis::Horizontal,
            layout::flex::Axis::Vertical => layout::flex::Axis::Vertical,
        }
    }
}

impl Widget<Message, IcedRenderer> for PanelScroll {
    fn width(&self) -> Length { self.width }
    fn height(&self) -> Length { self.height }

    fn children(&self) -> Vec<Tree> { self.children.iter().map(Tree::new).collect() }
    fn diff(&self, tree: &mut Tree) { tree.diff_children(&self.children) }
    fn state(&self) -> iced_core::widget::tree::State {
        println!("making panel scroll state");
        iced_core::widget::tree::State::new(PanelState::default())
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        renderer: &IcedRenderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) -> event::Status {
        use iced_core::widget::tree::State;
        if let &State::None = &tree.state {
            tree.state = State::new(PanelState::default())
        }

        // if we have a cursor pos, and the event is a mouse event, do our checks first
        if let (Some(cursor_pos), Event::Mouse(e)) = (cursor.position(), &event) {
            if self.handle_mouse_event(tree, cursor_pos, &e, layout, renderer) { 
                return event::Status::Captured;
            }
        }

        // if the cursor isnt over us, ignore the event
        if !cursor.position().map(|p|layout.bounds().contains(p)).unwrap_or_default() {
            return event::Status::Ignored;
        }

        // send event to all children
        self.children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }

    
    fn layout(
        &self,
        renderer: &IcedRenderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits
            .width(self.width)
            .height(self.height)
            .max_height(self.max_height);

        layout::flex::resolve(
            self.get_axis(),
            renderer,
            &limits,
            self.padding,
            self.spacing,
            self.align_items,
            &self.children,
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &IcedRenderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(self.id.as_ref(), layout.bounds(), &mut |operation| {
            self.children
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget()
                        .operate(state, layout, renderer, operation);
                })
        });
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut IcedRenderer,
        theme: &<IcedRenderer as iced_core::Renderer>::Theme,
        style: &iced_core::renderer::Style,
        layout: Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let old = renderer.start_layer();

        for ((child, state), layout) in 
            self.children.iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            if !viewport.intersects(&layout.bounds()) { continue }

            child
                .as_widget()
                .draw(state, renderer, theme, style, layout, cursor, viewport);
        }

        renderer.end_layer(old, layout.bounds());
    }
}

impl From<PanelScroll> for IcedElement {
    fn from(value: PanelScroll) -> Self {
        IcedElement::new(value)
    }
}

#[derive(Default)]
struct PanelState {
    left_pressed: bool,
    right_pressed: bool,

    pressed_at: iced::Point,
    did_move: bool,

    // left_pressed_at: Option<iced::Point>,
    // left_did_move: bool,

    // right_pressed_at: Option<iced::Point>,
    // right_did_move: bool,
}


use iced::widget::scrollable::{
    AbsoluteOffset,
    RelativeOffset,
};
use iced::advanced::widget::{operation, Id};

fn snap_to<T>(offset: RelativeOffset) -> impl Operation<T> {
    struct SnapTo {
        offset: RelativeOffset,
    }

    impl<T> Operation<T> for SnapTo {
        fn container(
            &mut self,
            _id: Option<&Id>,
            _bounds: iced::Rectangle,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
        ) {
            operate_on_children(self)
        }

        fn scrollable(
            &mut self,
            state: &mut dyn operation::Scrollable,
            _id: Option<&Id>,
            _bounds: iced::Rectangle,
            _translation: iced::Vector,
        ) {
            state.snap_to(self.offset);
        }
    }

    SnapTo { offset }
}




/// Produces an [`Operation`] that scrolls the widget with the given [`Id`] to
/// the provided [`AbsoluteOffset`].
fn scroll_to<T>(offset: AbsoluteOffset, relative: bool) -> impl Operation<T> {
    struct ScrollTo {
        relative: bool,
        offset: AbsoluteOffset,
    }

    impl<T> Operation<T> for ScrollTo {
        fn container(
            &mut self,
            _id: Option<&Id>,
            _bounds: iced::Rectangle,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
        ) {
            operate_on_children(self)
        }

        fn scrollable(
            &mut self,
            state: &mut dyn operation::Scrollable,
            _id: Option<&Id>,
            _bounds: iced::Rectangle,
            translation: iced::Vector,
        ) {
            let mut offset = self.offset;
            if self.relative {
                offset.x += translation.x;
                offset.y += translation.y;
            }
            state.scroll_to(offset);
        }
    }

    ScrollTo { relative, offset }
}




/// Produces an [`Operation`] that scrolls the widget with the given [`Id`] to
/// the provided [`AbsoluteOffset`].
pub fn scroll_to_id(target: &'static str, offset: AbsoluteOffset) -> IcedOperation {
    use iced::{ Vector, Rectangle };
    let id = Id::new(target);

    #[derive(Debug, Clone)]
    struct ScrollTo {
        target: Id,
        offset: AbsoluteOffset,
    }

    impl Operation<Message> for ScrollTo {
        fn container(
            &mut self,
            _id: Option<&Id>,
            _bounds: Rectangle,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<Message>),
        ) {
            operate_on_children(self)
        }

        fn scrollable(
            &mut self,
            state: &mut dyn iced::advanced::widget::operation::Scrollable,
            id: Option<&Id>,
            _bounds: Rectangle,
            _translation: Vector,
        ) {
            if Some(&self.target) == id {
                state.scroll_to(self.offset);
            }
        }
    }

    Box::new(ScrollTo { target: id, offset })
}

/// Produces an [`Operation`] that snaps the widget with the given [`Id`] to
/// the provided `percentage`.
pub fn snap_to_id(target: &'static str, offset: RelativeOffset) -> IcedOperation {
    use iced::{ Vector, Rectangle };
    let id = Id::new(target);

    #[derive(Debug, Clone)]
    struct SnapTo {
        target: Id,
        offset: RelativeOffset,
    }

    impl Operation<Message> for SnapTo {
        fn container(
            &mut self,
            _id: Option<&Id>,
            _bounds: Rectangle,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<Message>),
        ) {
            operate_on_children(self)
        }

        fn scrollable(
            &mut self,
            state: &mut dyn iced::advanced::widget::operation::Scrollable,
            id: Option<&Id>,
            _bounds: Rectangle,
            _translation: Vector,
        ) {
            if Some(&self.target) == id {
                state.snap_to(self.offset);
            }
        }
    }

    Box::new(SnapTo { target: id, offset })
}