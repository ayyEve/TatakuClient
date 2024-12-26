use crate::prelude::*;


/// literally an empty element
pub struct EmptyElement;

impl iced::advanced::Widget<Message, iced::Theme, IcedRenderer> for EmptyElement {
    fn size(&self) -> iced::Size<iced::Length> { 
        iced::Size::new(iced::Length::Fixed(0.0), iced::Length::Fixed(0.0)) 
    }

    fn layout(
        &self,
        _state: &mut iced_core::widget::Tree,
        _renderer: &IcedRenderer,
        _limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        iced_core::layout::Node::new(iced::Size::ZERO)
    }

    fn draw(
        &self,
        _state: &iced_core::widget::Tree,
        _renderer: &mut IcedRenderer,
        _theme: &iced::Theme,
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
