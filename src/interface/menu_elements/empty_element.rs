use crate::prelude::*;

/// literally an empty element
pub struct EmptyElement;

impl iced::advanced::Widget<Message, IcedRenderer> for EmptyElement {
    fn width(&self) -> iced::Length { iced::Length::Fixed(0.0) }
    fn height(&self) -> iced::Length { iced::Length::Fixed(0.0) }

    fn layout(
        &self,
        _renderer: &IcedRenderer,
        _limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        iced_core::layout::Node::new(iced::Size::ZERO)
    }

    fn draw(
        &self,
        _state: &iced_core::widget::Tree,
        _renderer: &mut IcedRenderer,
        _theme: &<IcedRenderer as iced_core::Renderer>::Theme,
        _style: &iced_core::renderer::Style,
        _layout: iced_core::Layout<'_>,
        _cursor: iced_core::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {}
    
}

impl From<EmptyElement> for IcedElement {
    fn from(value: EmptyElement) -> Self {
        Self::new(value)
    }
}