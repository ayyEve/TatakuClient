
use crate::prelude::*;
use std::ops::RangeInclusive;
use crate::prelude::iced_elements::{*, Rectangle};
use iced::advanced::graphics::Primitive;

macro_rules! chain {
    ($name:ident, $type: ty) => {
        pub fn $name(mut self, $name: $type) -> Self {
            self.$name = $name;
            self
        }
    }
}

pub struct ProgressBarWidget {
    width: Length,
    height: Length,

    background_color: Color,
    fill_color: Color,
    shape: Shape,
    border: Option<(f32, Color)>,

    /// 0..max
    value: f32,
    range: RangeInclusive<f32>,

    on_click: Option<(MessageOwner, MessageTag)>
}
impl ProgressBarWidget {
    pub fn new(range: RangeInclusive<f32>, value: f32) -> Self {
        Self {
            width: Fill,
            height: Shrink,

            background_color: Color::WHITE.alpha(0.8),
            fill_color: Color::CYAN,

            shape: Shape::Round(4.0),
            border: None,
            on_click: None,

            value,
            range,
        }
    }
    
    chain!(width, Length);
    chain!(height, Length);
    chain!(background_color, Color);
    chain!(fill_color, Color);
    chain!(shape, Shape);
    chain!(border, Option<(f32, Color)>);
    // chain!(on_click, Option<(MessageOwner, MessageTag)>);

    pub fn on_click(mut self, on_click: Option<(MessageOwner, impl Into<MessageTag>)>) -> Self {
        self.on_click = on_click.map(|(owner, tag)| (owner,tag.into()));
        self
    }

    fn value_percent(&self) -> f32 {
        self.range.end() / (self.value - self.range.start())
    }
}


impl iced::advanced::Widget<Message, IcedRenderer> for ProgressBarWidget {
    fn width(&self) -> Length { self.width }
    fn height(&self) -> Length { self.height }

    fn layout(
        &self,
        _renderer: &IcedRenderer,
        limits: &iced_runtime::core::layout::Limits,
    ) -> iced_runtime::core::layout::Node {
        iced_runtime::core::layout::Node::new(
            limits
            .width(self.width)
            .height(self.height)
            .fill()
        )
    }

    fn on_event(
        &mut self,
        _state: &mut iced_runtime::core::widget::Tree,
        event: iced::Event,
        layout: iced_runtime::core::Layout<'_>,
        cursor: iced_runtime::core::mouse::Cursor,
        _renderer: &IcedRenderer,
        _clipboard: &mut dyn iced_runtime::core::Clipboard,
        shell: &mut iced_runtime::core::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> iced::event::Status {
        let iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) = event else { return iced::event::Status::Ignored };
        let Some((owner, tag)) = &self.on_click else { return iced::event::Status::Ignored };

        let bounds = layout.bounds();
        let Some(pos) = cursor.position_in(bounds) else { return iced::event::Status::Ignored };

        let x = pos.x / bounds.width;
        let amount = self.range.start() + x * self.range.end();
        let message = Message::new(owner.clone(), tag.clone(), MessageType::Float(amount));
        shell.publish(message);

        iced::event::Status::Captured
    }

    fn draw(
        &self,
        _state: &iced_runtime::core::widget::Tree,
        renderer: &mut IcedRenderer,
        _theme: &<IcedRenderer as iced_runtime::core::Renderer>::Theme,
        _style: &iced_runtime::core::renderer::Style,
        layout: iced_runtime::core::Layout<'_>,
        _cursor: iced_runtime::core::mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let border_width = 0.0;
        let border_color = Color::TRANSPARENT_WHITE.into();
        let border_radius = match self.shape {
            Shape::Square => [0.0; 4],
            Shape::Round(r) => [r; 4],
            Shape::RoundSep(a) => a,
        };
        
        // bg
        renderer.draw_primitive(Primitive::Quad { 
            bounds, 
            background: iced::Background::Color(self.background_color.into()), 
            border_radius, 
            border_width, 
            border_color
        });

        // fill
        let mut fill_bounds = bounds;
        fill_bounds.width *= self.value_percent();
        renderer.draw_primitive(Primitive::Quad { 
            bounds: fill_bounds, 
            background: iced::Background::Color(self.fill_color.into()), 
            border_radius, 
            border_width, 
            border_color
        });

        // border
        if let Some((border_width, border_color)) = self.border {
            renderer.draw_primitive(Primitive::Quad { 
                bounds, 
                background: iced::Background::Color(Color::TRANSPARENT_WHITE.into()), 
                border_radius, 
                border_width, 
                border_color: border_color.into()
            });
        }
    }
}

impl From<ProgressBarWidget> for IcedElement {
    fn from(value: ProgressBarWidget) -> Self {
        Self::new(value)
    }
}