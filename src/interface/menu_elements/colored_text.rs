use crate::prelude::*;

pub trait SetColor {
    fn color(self, color: Color) -> Self;
}

impl<'a> SetColor for iced::widget::Text<'a, IcedRenderer> {
    fn color(self, color: Color) -> Self {
        let color = iced::Color::new(color.r, color.g, color.b, color.a);
        self.style(color)
    }
}