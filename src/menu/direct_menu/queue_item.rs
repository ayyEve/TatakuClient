use super::prelude::*;

#[derive(Clone)]
pub struct DirectItem {
    pos: Vector2,
    hover: bool,
    selected: bool,

    pub item: Arc<dyn DirectDownloadable>,
    pub downloading: bool,
}
impl DirectItem {
    pub fn new(item: Arc<dyn DirectDownloadable>) -> DirectItem {
        DirectItem {
            pos: Vector2::zero(), // being set by the scroll area anyways
            item, //DirectMeta::from_str(str.clone()),

            hover: false,
            selected:false,
            downloading: false
        }
    }
}
impl ScrollableItemGettersSetters for DirectItem {
    fn size(&self) -> Vector2 {DIRECT_ITEM_SIZE}
    fn set_pos(&mut self, pos:Vector2) {self.pos = pos}
    fn get_pos(&self) -> Vector2 {self.pos}
    fn get_tag(&self) -> String {self.item.filename()}
    fn set_tag(&mut self, _tag:&str) {}

    fn get_hover(&self) -> bool {self.hover}
    fn set_hover(&mut self, hover:bool) {self.hover = hover}
    fn get_selected(&self) -> bool {self.selected}
    fn set_selected(&mut self, selected:bool) {self.selected = selected}
}

impl ScrollableItem for DirectItem {
    fn get_value(&self) -> Box<dyn std::any::Any> {Box::new(self.item.clone())}

    fn draw(&mut self, _args:piston::RenderArgs, pos_offset:Vector2, parent_depth:f64, list:&mut Vec<Box<dyn Renderable>>) {
        let font = get_font("main");

        list.push(Box::new(Rectangle::new(
            Color::WHITE,
            parent_depth + 10.0,
            self.pos + pos_offset,
            self.size(),
            Some(Border::new(if self.hover {Color::BLUE} else if self.selected {Color::RED} else {Color::BLACK}, 1.5))
        )));

        list.push(Box::new(Text::new(
            Color::BLACK,
            parent_depth + 9.9,
            self.pos+Vector2::new(5.0, 25.0) + pos_offset,
            20,
            format!("{} - {}", self.item.artist(), self.item.title()),
            font.clone()
        )));

        list.push(Box::new(Text::new(
            Color::BLACK,
            parent_depth + 9.9,
            self.pos+Vector2::new(5.0, 50.0) + pos_offset,
            20,
            format!("Mapped by {}", self.item.creator()),
            font.clone()
        )));
    }

    fn on_click(&mut self, _pos:Vector2, _button:piston::MouseButton, _mods:KeyModifiers) -> bool {
        // if self.selected && self.hover {self.item.download()}

        self.selected = self.hover;
        self.hover
    }
}