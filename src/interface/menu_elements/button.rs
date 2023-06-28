// use graphics::Context;
use crate::prelude::*;
const BACK_BUTTON_SIZE:Vector2 = Vector2::new(100.0, 50.0);

#[derive(Clone, ScrollableGettersSetters)]
pub struct MenuButton {
    pos: Vector2,
    size: Vector2,
    hover: bool,
    selected: bool,
    tag: String,
    keywords: Vec<String>,

    pub text: String,
    pub font: Font,
    pub font_size: f32,

    pub on_click:Arc<dyn Fn(&mut Self) + 'static + Send + Sync>, 
}
impl MenuButton {
    pub fn new(pos: Vector2, size: Vector2, text:&str, font:Font) -> Self {
        Self {
            pos, 
            size, 
            text: text.to_owned(),
            keywords: text.split(" ").map(|a|a.to_lowercase().to_owned()).collect(),

            hover: false,
            selected: false,
            tag: String::new(),
            // context: None,
            font,
            font_size: 12.0,
            on_click: Arc::new(|_|{}),
        }
    }

    pub fn back_button(window_size:Vector2, font:Font) -> Self {
        Self::new(Vector2::new(10.0, window_size.y - (BACK_BUTTON_SIZE.y + 10.0)), BACK_BUTTON_SIZE, "Back", font)
    }
}



impl ScrollableItem for MenuButton {
    fn get_keywords(&self) -> Vec<String> { self.keywords.clone() }

    fn on_click_tagged(&mut self, pos:Vector2, _button:MouseButton, _mods:KeyModifiers) -> Option<String> {
        self.check_hover(pos);

        if self.get_hover() {
            (self.on_click.clone())(self);
            Some(self.tag.clone())
        } else {
            None
        }
    }

    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {

        // draw box
        let r = Rectangle::new(
            self.pos + pos_offset,
            self.size,
            Color::new(0.2, 0.2, 0.2, 1.0),
            if self.hover {Some(Border::new(Color::RED, 1.0))} else if self.selected {Some(Border::new(Color::BLUE, 1.0))} else {None}
        );
        
        // draw text
        let mut txt = Text::new(
            Vector2::ZERO,
            self.font_size,
            self.text.clone(),
            Color::WHITE,
            self.font.clone()
        );
        txt.center_text(&r);

        list.push(r);
        list.push(txt);
    }
}
