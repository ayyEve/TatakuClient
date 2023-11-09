use crate::prelude::*;

#[derive(Clone, ScrollableGettersSetters)]
pub struct KeyButton {
    pos: Vector2,
    size: Vector2,
    style: Style,
    node: Node,


    selected: bool,
    hover: bool,
    tag: String,

    pub key: Key,

    pub prefix: String,
    pub font: Font,
    pub font_size: f32,

    pub on_change: Arc<dyn Fn(&mut Self, Key) + Send + Sync>,
}
impl KeyButton {
    pub fn new(style: Style, key:Key, prefix: impl ToString, layout_manager: &LayoutManager, font:Font) -> Self {
        let (pos, size) = LayoutManager::get_pos_size(&style);
        let node = layout_manager.create_node(&style);

        Self {
            pos, 
            size, 
            style,
            node,
            
            hover: false,
            selected: false,
            tag: String::new(),


            key,
            prefix: prefix.to_string(),

            font,
            font_size: 32.0,
            
            on_change: Arc::new(|_,_|{}),
        }
    }

    fn text(&self) -> String {
        if self.selected {
            "Press a key".to_owned()
        } else {
            format!("{:?}", self.key)
        }
    }
}
impl ScrollableItem for KeyButton {
    fn get_style(&self) -> Style { self.style.clone() }
    fn apply_layout(&mut self, layout: &LayoutManager, parent_pos: Vector2) {
        let layout = layout.get_layout(self.node);
        self.pos = layout.location.into();
        self.pos += parent_pos;
        self.size = layout.size.into();
    }

    fn get_value(&self) -> Box<dyn std::any::Any> {Box::new(self.key.clone())}
    fn draw(&mut self, pos_offset:Vector2, list:&mut RenderableCollection) {
        let border = Rectangle::new(
            self.pos + pos_offset,
            self.size, 
            Color::WHITE,
            Some(Border::new(if self.hover {Color::RED} else if self.selected {Color::BLUE} else {Color::BLACK}, 1.2))
        );
        list.push(border);

        let text = Text::new(
            self.pos + pos_offset,
            self.font_size,
            format!("{}: {}", self.prefix, self.text()),
            Color::BLACK,
            self.font.clone()
        );
        list.push(text);
    }

    fn on_click(&mut self, _pos:Vector2, _btn:MouseButton, _mods:KeyModifiers) -> bool {

        // try to extrapolate where the mouse was clicked, and change the cursor_index to that
        if self.selected {
            if !self.hover {
                self.selected = false;
                return false;
            }
            return true;
        }

        if self.hover {self.selected = true}
        return self.hover
    }

    fn on_key_press(&mut self, key:Key, _mods:KeyModifiers) -> bool {
        if !self.selected {return false}

        // TODO: check exclusion list
        if key == Key::Escape {
            self.selected = false;
            return true;
        }

        self.key = key;
        self.selected = false;
        (self.on_change.clone())(self, key);

        true
    }
}
