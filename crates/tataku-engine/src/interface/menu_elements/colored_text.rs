use crate::prelude::*;

pub trait SetColor {
    fn color(self, color: Color) -> Self;
}

impl SetColor for iced::widget::Text<'_, iced::Theme, IcedRenderer> {
    fn color(self, color: Color) -> Self {
        let style = iced::widget::text::Style {
            color: Some(iced::Color::new(color.r, color.g, color.b, color.a)),
        };
        self.style(move |_| style)
    }
}
