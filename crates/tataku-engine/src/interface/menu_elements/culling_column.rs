use crate::prelude::*;
use iced::{
    Pixels,
    Length,
    Padding,
    Alignment,

    mouse,
    overlay,
    event::{ self, Event },
    widget::scrollable::{
        Scrollable,
        AbsoluteOffset,
        RelativeOffset,
    },

    advanced::{
        Shell,
        Clipboard,
        layout::{ self, Layout },
        widget::{
            Tree,
            Widget,
            operation,
            Operation,
            Id,
        },
    }
};

pub fn make_scrollable(
    children: Vec<IcedElement>, 
    id: impl Into<Cow<'static, str>>
) -> Scrollable<'static, Message, iced::Theme, IcedRenderer> {
    // Scrollable::new(CullingColumn::with_chilDraggingScrolldren(children).spacing(5.0).into_element()).id(iced::widget::scrollable::Id::new(id))
    Scrollable::new(
        iced_elements::Column::with_children(children)
        .spacing(5.0)
        .clip(true)
        .into_element())
        .id(iced::widget::scrollable::Id::new(id)
    )
}
pub fn make_panel_scroll(children: Vec<IcedElement>, id: impl Into<Cow<'static, str>>) -> DraggingScroll {
    DraggingScroll::with_children(vec![make_scrollable(children, id).into_element()])
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

impl Widget<Message, iced::Theme, IcedRenderer> for CullingColumn {
    fn children(&self) -> Vec<Tree> { self.children.iter().map(Tree::new).collect() }
    fn diff(&self, tree: &mut Tree) { tree.diff_children(&self.children); }
    fn size(&self) -> iced::Size<iced::Length> { iced::Size::new(self.width, self.height) }


    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &IcedRenderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.max_width(self.max_width);

        layout::flex::resolve(
            layout::flex::Axis::Vertical,
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
        // use iced::advanced::Renderer;
        // renderer.start_layer(layout.bounds());

        for ((child, state), layout) in 
            self.children.iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            // if !viewport.intersects(&layout.bounds()) { continue }

            child
                .as_widget()
                .draw(state, renderer, theme, style, layout, cursor, viewport);
        }

        // renderer.end_layer();
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &IcedRenderer,
        offset: iced::Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, IcedRenderer>> {
        iced::advanced::overlay::from_children(&mut self.children, tree, layout, renderer, offset)
    }
}

impl From<CullingColumn> for IcedElement {
    fn from(value: CullingColumn) -> Self {
        Self::new(value)
    }
}

impl Default for CullingColumn {
    fn default() -> Self {
        Self::new()
    }
}



pub fn snap_to<T>(offset: RelativeOffset) -> impl Operation<T> {
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
            _content_bounds: iced::Rectangle,
            _translation: iced::Vector,
        ) {
            state.snap_to(self.offset);
        }
    }

    SnapTo { offset }
}




/// Produces an [`Operation`] that scrolls the widget with the given [`Id`] to
/// the provided [`AbsoluteOffset`].
pub fn scroll_to<T>(offset: AbsoluteOffset, relative: bool) -> impl Operation<T> {
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
            _content_bounds: iced::Rectangle,
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
pub fn scroll_to_id(target: impl Into<Cow<'static, str>>, offset: AbsoluteOffset) -> IcedOperation {
    use iced::{ Vector, Rectangle };
    let id = Id::new(target);

    #[derive(Debug, Clone)]
    struct ScrollTo {
        target: Id,
        offset: AbsoluteOffset,
    }

    impl Operation for ScrollTo {
        fn container(
            &mut self,
            _id: Option<&Id>,
            _bounds: Rectangle,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation),
        ) {
            operate_on_children(self)
        }

        fn scrollable(
            &mut self,
            state: &mut dyn iced::advanced::widget::operation::Scrollable,
            id: Option<&Id>,
            _bounds: Rectangle,
            _content_bounds: iced::Rectangle,
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

    impl Operation for SnapTo {
        fn container(
            &mut self,
            _id: Option<&Id>,
            _bounds: Rectangle,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation),
        ) {
            operate_on_children(self)
        }

        fn scrollable(
            &mut self,
            state: &mut dyn iced::advanced::widget::operation::Scrollable,
            id: Option<&Id>,
            _bounds: Rectangle,
            _content_bounds: iced::Rectangle,
            _translation: Vector,
        ) {
            if Some(&self.target) == id {
                state.snap_to(self.offset);
            }
        }
    }

    Box::new(SnapTo { target: id, offset })
}
