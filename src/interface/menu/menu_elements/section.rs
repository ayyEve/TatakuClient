use crate::prelude::*;


/// basically a spacer with some text
#[derive(ScrollableGettersSetters)]
pub struct MenuSection {
    size: Vector2,
    pos: Vector2,
    pub text: String,
    pub font: Font,
    pub font_size: f32,

    // hover: bool
}
impl MenuSection {
    pub fn new(pos:Vector2, height:f32, text:&str, font:Font) -> Self {
        Self {
            pos, 
            size: Vector2::new(300.0, height),
            text: text.to_owned(),
            font,
            font_size: 32.0,
            // hover: false,
        }
    }
}

impl ScrollableItem for MenuSection {
    fn on_click(&mut self, _pos:Vector2, _button:MouseButton, _mods:KeyModifiers) -> bool {false} //{self.hover}

    fn draw(&mut self, pos_offset:Vector2, parent_depth:f32, list:&mut RenderableCollection) {
        // text
        let t = Text::new(
            Color::BLACK,
            parent_depth,
            self.pos + pos_offset,
            self.font_size,
            self.text.clone(),
            self.font.clone()
        );

        // underline
        let r = Rectangle::new(
            Color::BLACK,
            parent_depth,
            self.pos + pos_offset + Vector2::new(0.0, self.font_size + 10.0),
            Vector2::new(self.size().x, 4.0),
            None
        );

        list.push(t);
        list.push(r);
    }

}