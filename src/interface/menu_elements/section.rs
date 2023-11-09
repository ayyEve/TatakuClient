use crate::prelude::*;

const BOTTOM_PAD:f32 = 5.0;
const UNDERLINE_PAD:f32 = 10.0;

/// basically a spacer with some text
#[derive(ScrollableGettersSetters)]
pub struct MenuSection {
    size: Vector2,
    pos: Vector2,
    style: Style,
    node: Node,

    pub text: String,
    pub font: Font,
    pub font_size: f32,
    keywords: Vec<String>,
    color: Color
}

impl MenuSection {
    pub fn new(style: Style, text:&str, color: Color, layout_manager: &LayoutManager, font:Font) -> Self {
        let (pos, size) = LayoutManager::get_pos_size(&style);
        let node = layout_manager.create_node(&style);
        
        Self {
            pos, 
            size,
            style, 
            node,

            text: text.to_owned(),
            color,
            font,
            font_size: 32.0,
            keywords: text.split(" ").map(|a|a.to_lowercase().to_owned()).collect(),
        }
    }
}

impl ScrollableItem for MenuSection {
    fn get_style(&self) -> Style { self.style.clone() }
    fn apply_layout(&mut self, layout: &LayoutManager, parent_pos: Vector2) {
        let layout = layout.get_layout(self.node);
        self.pos = layout.location.into();
        self.pos += parent_pos;
        self.size = layout.size.into();
    }

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
