use crate::prelude::*;

#[derive(ScrollableGettersSetters)]
pub struct SimpleText {
    pos: Vector2,
    size: Vector2,
    style: Style,
    node: Node,

    font_size: f32,
    text: String,

    color: Color,
    font: Font,
}
impl SimpleText {
    pub fn new(style: Style, font_size: f32, text: impl ToString, layout_manager: &LayoutManager) -> Self {
        let (pos, size) = LayoutManager::get_pos_size(&style);
        let node = layout_manager.create_node(&style);

        Self {
            pos, 
            size,
            style, 
            node,
            
            font_size,
            text: text.to_string(),

            color: Color::BLACK,
            font: Font::Main,
        }
    }
    pub fn with_font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl ScrollableItem for SimpleText {
    fn get_style(&self) -> Style { self.style.clone() }
    fn apply_layout(&mut self, layout: &LayoutManager, parent_pos: Vector2) {
        let layout = layout.get_layout(self.node);
        self.pos = layout.location.into();
        self.pos += parent_pos;
        self.size = layout.size.into();
    }

    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
        list.push(Text::new(
            self.pos + pos_offset,
            self.font_size,
            &self.text,
            self.color,
            self.font.clone()
        ))
    }
}