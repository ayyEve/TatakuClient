use crate::prelude::*;
use iced::{
    Pixels,
    Length,
    Padding,
    Alignment,

    mouse,
    event::{ self, Event },
    widget::scrollable::{
        AbsoluteOffset,
        RelativeOffset,
    }
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


/// runs all events for all items in the list
/// also enables left and right-click drag scrolling
pub struct DraggingScroll {
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
impl DraggingScroll {
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
    pub fn push(mut self, child: IcedElement) -> Self {
        self.children.push(child);
        self
    }

    fn operate_on_children(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &IcedRenderer,
        operation: &mut dyn Operation
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

impl Default for DraggingScroll {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget<Message, iced::Theme, IcedRenderer> for DraggingScroll {
    fn size(&self) -> iced::Size<iced::Length> { iced::Size::new(self.width, self.height) }

    fn children(&self) -> Vec<Tree> { self.children.iter().map(Tree::new).collect() }
    fn diff(&self, tree: &mut Tree) { tree.diff_children(&self.children) }
    fn tag(&self) -> iced_core::widget::tree::Tag { iced_core::widget::tree::Tag::of::<PanelState>() }
    fn state(&self) -> iced_core::widget::tree::State { iced_core::widget::tree::State::new(PanelState::default()) }

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

        // if we have a cursor pos, and the event is a mouse event, do our checks first
        if let (Some(cursor_pos), Event::Mouse(e)) = (cursor.position(), &event) {
            if self.handle_mouse_event(tree, cursor_pos, e, layout, renderer) { 
                return event::Status::Captured;
            }
        }

        // if the cursor isnt over us, ignore the event
        if !cursor.position().map(|p| layout.bounds().contains(p)).unwrap_or_default() {
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
        tree: &mut Tree,
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
            self.width,
            self.height,
            self.padding,
            self.spacing,
            self.align_items,
            &self.children,
            &mut tree.children
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &IcedRenderer,
        operation: &mut dyn Operation,
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
        theme: &iced::Theme,
        style: &iced_core::renderer::Style,
        layout: Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        use iced::advanced::Renderer;
        renderer.start_layer(layout.bounds());

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

        renderer.end_layer();
    }
}

impl From<DraggingScroll> for IcedElement {
    fn from(value: DraggingScroll) -> Self {
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
