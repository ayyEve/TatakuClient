use crate::prelude::*;

use iced::Theme;
use iced_runtime::core::{
    layout,
    mouse,
    overlay,
    renderer,
    alignment,
    event::{self, Event},
    widget::operation::Operation,
    widget::tree::Tree,
    
    Clipboard, Layout, Length, Point,
    Rectangle, Shell, Padding, Pixels,
    Alignment
};

#[derive(ChainableInitializer)]
pub struct ContentBackground {
    content: IcedElement,
    image: Option<Image>,

    #[chain]
    border: Option<Border>,
    #[chain]
    color: Option<Color>,
    #[chain]
    shape: Shape,

    #[chain]
    width: Length,
    #[chain]
    height: Length,

    max_width: f32,
    max_height: f32,

    #[chain]
    padding: Padding,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
}

impl ContentBackground {
    /// Creates a new [`ContentWithImage`] with the given content
    pub fn new(content: impl IntoElement) -> Self {
        Self {
            content: content.into_element(),
            image: None,
            color: None,
            border: None,
            shape: Shape::Square,

            width: Length::Shrink,
            height: Length::Shrink,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,

            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            padding: Padding::new(0.0),
        }
    }
    
    /// set this content's background image
    pub fn image(mut self, mut image: Option<Image>) -> Self {
        image.ok_do_mut(|i|i.origin = Vector2::ZERO);
        self.image = image;
        self
    }


    /// Sets the maximum width of the [`Container`].
    pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
        self.max_width = max_width.into().0;
        self
    }

    /// Sets the maximum height of the [`Container`].
    pub fn max_height(mut self, max_height: impl Into<Pixels>) -> Self {
        self.max_height = max_height.into().0;
        self
    }

    // /// Sets the [`Padding`] of the [`Button`].
    // pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
    //     self.padding = padding.into();
    //     self
    // }

    /// Sets the content alignment for the horizontal axis of the [`Container`].
    pub fn align_x(mut self, alignment: alignment::Horizontal) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    /// Sets the content alignment for the vertical axis of the [`Container`].
    pub fn align_y(mut self, alignment: alignment::Vertical) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    /// Centers the contents in the horizontal axis of the [`Container`].
    pub fn center_x(mut self) -> Self {
        self.horizontal_alignment = alignment::Horizontal::Center;
        self
    }

    /// Centers the contents in the vertical axis of the [`Container`].
    pub fn center_y(mut self) -> Self {
        self.vertical_alignment = alignment::Vertical::Center;
        self
    }

}


impl iced::advanced::Widget<Message, IcedRenderer> for ContentBackground {
    fn width(&self) -> Length { self.width }
    fn height(&self) -> Length { self.height }
    
    fn children(&self) -> Vec<Tree> { vec![Tree::new(&self.content)] }
    fn diff(&self, tree: &mut Tree) { tree.diff_children(std::slice::from_ref(&self.content)) }


    fn layout(
        &self,
        renderer: &IcedRenderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits
            .loose()
            .max_width(self.max_width)
            .max_height(self.max_height)
            .width(self.width)
            .height(self.height);

        let mut content = self.content.as_widget().layout(renderer, &limits.pad(self.padding).loose());
        let padding = self.padding.fit(content.size(), limits.max());
        let size = limits.pad(padding).resolve(content.size());

        content.move_to(Point::new(padding.left, padding.top));
        content.align(
            Alignment::from(self.horizontal_alignment),
            Alignment::from(self.vertical_alignment),
            size,
        );

        layout::Node::with_children(size.pad(padding), vec![content])
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &IcedRenderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
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
        viewport: &Rectangle,
    ) -> event::Status {
        self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut IcedRenderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();

        if let Some(mut image) = self.image.clone() {
            image.pos = bounds.position().into();
            image.set_size(bounds.size().into());

            renderer.draw_primitive(iced::advanced::graphics::Primitive::Custom(Arc::new(image)));
        } else if self.color.is_some() || self.border.is_some() {
            let rect = crate::prelude::Rectangle::new(
                bounds.position().into(), 
                bounds.size().into(),
                self.color.unwrap_or(Color::TRANSPARENT_WHITE),
                self.border
            ).shape(self.shape);
            renderer.draw_primitive(iced::advanced::graphics::Primitive::Custom(Arc::new(rect)));
        }

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            content_layout,
            cursor,
            &bounds,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &IcedRenderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &IcedRenderer,
    ) -> Option<overlay::Element<'b, Message, IcedRenderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
        )
    }
    
}

impl From<ContentBackground> for IcedElement {
    fn from(value: ContentBackground) -> Self {
        IcedElement::new(value)
    }
}
