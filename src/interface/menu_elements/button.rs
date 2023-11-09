use taffy::prelude::Rect;

// use graphics::Context;
use crate::prelude::*;
const BACK_BUTTON_SIZE:Vector2 = Vector2::new(100.0, 50.0);

#[derive(Clone, ScrollableGettersSetters)]
pub struct MenuButton {
    pos: Vector2,
    size: Vector2,
    style: Style,
    node: Node,

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
    pub fn new(style: Style, text: impl ToString, layout_manager: &LayoutManager, font: Font) -> Self {
        let text = text.to_string();
        let (pos, size) = LayoutManager::get_pos_size(&style);
        let node = layout_manager.create_node(&style);

        Self {
            pos, 
            size, 
            style,
            node,

            keywords: text.split(" ").map(|a|a.to_lowercase().to_owned()).collect(),
            text,

            hover: false,
            selected: false,
            tag: String::new(),
            // context: None,
            font,
            font_size: 12.0,
            on_click: Arc::new(|_|{}),
        }
    }


    pub fn with_on_click(mut self, onclick: impl Fn(&mut Self) + 'static + Send + Sync) -> Self {
        self.on_click = Arc::new(onclick);
        self
    }

    pub fn back_button(font:Font, layout_manager: &LayoutManager) -> Self {
        Self::new(Style {
            size: BACK_BUTTON_SIZE.into(),
            inset: Rect {
                left: taffy::style::LengthPercentageAuto::Points(5.0),
                bottom: taffy::style::LengthPercentageAuto::Points(5.0),
                right: taffy::style::LengthPercentageAuto::Auto,
                top: taffy::style::LengthPercentageAuto::Auto,
            },
            ..Default::default()
        }, "Back", layout_manager, font)
    }
}



impl ScrollableItem for MenuButton {
    fn get_style(&self) -> Style { self.style.clone() }
    fn apply_layout(&mut self, layout: &LayoutManager, parent_pos: Vector2) {
        let layout = layout.get_layout(self.node);
        self.pos = layout.location.into();
        self.pos += parent_pos;
        self.size = layout.size.into();
    }


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
        txt.center_text(&*r);

        list.push(r);
        list.push(txt);
    }
}
