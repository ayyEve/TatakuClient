use crate::prelude::*;

const BOTTOM_PAD:f32 = 5.0;
const UNDERLINE_PAD:f32 = 10.0;

/// basically a spacer with some text
#[derive(ScrollableGettersSetters)]
pub struct MenuSection {
    size: Vector2,
    pos: Vector2,
    pub text: String,
    pub font: Font,
    pub font_size: f32,
    keywords: Vec<String>,
    color: Color
}

impl MenuSection {
    pub fn new(pos:Vector2, height:f32, text:&str, color: Color, font:Font) -> Self {
        Self {
            pos, 
            size: Vector2::new(300.0, height),
            text: text.to_owned(),
            color,
            font,
            font_size: 32.0,
            keywords: text.split(" ").map(|a|a.to_lowercase().to_owned()).collect(),
        }
    }
}

impl ScrollableItem for MenuSection {
    fn get_keywords(&self) -> Vec<String> { self.keywords.clone() }
    fn on_click(&mut self, _pos:Vector2, _button:MouseButton, _mods:KeyModifiers) -> bool {false} //{self.hover}

    fn draw(&mut self, pos_offset:Vector2, list:&mut RenderableCollection) {
        let base_pos = self.pos + Vector2::with_y(self.size.y - (self.font_size + UNDERLINE_PAD + BOTTOM_PAD));

        // text
        list.push(Text::new(
            base_pos + pos_offset,
            self.font_size,
            self.text.clone(),
            self.color,
            self.font.clone()
        ));

        // underline
        list.push(Rectangle::new(
            base_pos + pos_offset + Vector2::with_y(self.font_size + UNDERLINE_PAD),
            Vector2::new(self.size().x, 4.0),
            self.color,
            None
        ));
    }

}
