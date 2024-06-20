use crate::prelude::*;

pub trait SetColor {
    fn color(self, color: Color) -> Self;
}

impl<'a> SetColor for iced::widget::Text<'a, iced::Theme, IcedRenderer> {
    fn color(self, color: Color) -> Self {
        let mut style = iced::widget::text::Style::default();
        style.color = Some(iced::Color::new(color.r, color.g, color.b, color.a));
        self.style(move |_| style)
    }
}