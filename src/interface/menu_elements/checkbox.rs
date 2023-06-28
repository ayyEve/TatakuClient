use crate::prelude::*;

const INNER_BOX_PADDING:f32 = 8.0;

#[derive(Clone, ScrollableGettersSetters)]
pub struct Checkbox {
    pos: Vector2,
    size: Vector2,
    hover: bool,
    selected: bool,
    tag: String,
    keywords: Vec<String>,

    pub text: String,
    pub checked: bool,
    pub font: Font,
    pub font_size: f32,

    pub on_change: Arc<dyn Fn(&mut Self, bool) + Send + Sync>,
}
impl Checkbox {
    pub fn new(pos: Vector2, size: Vector2, text:&str, value:bool, font: Font) -> Self {
        Self {
            pos, 
            size, 
            text: text.to_owned(),
            keywords: text.split(" ").map(|a|a.to_lowercase().to_owned()).collect(),

            hover: false,
            selected: false,
            tag: String::new(),
            checked: value,
            font,
            font_size: 12.0,
            
            on_change: Arc::new(|_,_|{}),
        }
    }
}

impl ScrollableItem for Checkbox {
    fn get_value(&self) -> Box<dyn std::any::Any> {Box::new(self.checked)}
    fn get_keywords(&self) -> Vec<String> { self.keywords.clone() }

    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
        // draw bounding box
        list.push(Rectangle::new(
            self.pos + pos_offset,
            self.size,
            Color::new(0.2, 0.2, 0.2, 1.0),
            if self.hover {Some(Border::new(Color::RED, 1.0))} else if self.selected {Some(Border::new(Color::BLUE, 1.0))} else {None}
        ));

        // draw checkbox bounding box
        list.push(Rectangle::new(
            self.pos + pos_offset,
            Vector2::new(self.size.y, self.size.y),
            Color::TRANSPARENT_BLACK,
            if self.hover {Some(Border::new(Color::BLACK, 1.0))} else {None}
        ));
        if self.checked {
            list.push(Rectangle::new(
                self.pos + pos_offset + Vector2::new(INNER_BOX_PADDING, INNER_BOX_PADDING),
                Vector2::new(self.size.y-INNER_BOX_PADDING*2.0, self.size.y-INNER_BOX_PADDING * 2.0),
                Color::YELLOW,
                None
            ));
        }
        
        // draw text
        let mut txt = Text::new(
            self.pos + pos_offset,
            self.font_size,
            self.text.clone(),
            Color::WHITE,
            self.font.clone(),
        );
        txt.center_text(&Rectangle::bounds_only(self.pos + pos_offset + Vector2::new(self.size.y, 0.0), Vector2::new(self.size.x - self.size.y, self.size.y)));

        list.push(txt);
    }

    fn on_click_tagged(&mut self, pos:Vector2, _button:MouseButton, _mods:KeyModifiers) -> Option<String> {
        self.check_hover(pos);
        if self.hover { 
            self.checked = !self.checked;
            (self.on_change.clone())(self, self.checked);

            Some(self.tag.clone())
        } else {
            None
        }
    }
}