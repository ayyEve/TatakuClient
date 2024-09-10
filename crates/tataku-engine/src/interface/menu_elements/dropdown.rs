use crate::prelude::*;



// pub trait Dropdownable: Sized + Clone + Send + Sync {
//     fn variants() -> Vec<Self>;
//     fn display_text(&self) -> String;
//     fn from_string(s:String) -> Self;
// }

// const Y_PADDING:f32 = 5.0;
// const ITEM_Y_PADDING:f32 = 5.0;

// #[derive(Clone)]
// pub struct Dropdown<E:Dropdownable> {
//     pos: Vector2,
//     size: Vector2,
//     hover: bool,
//     selected: bool,
//     tag: String,
//     keywords: Vec<String>,

//     text: String,
//     pub value: Option<E>,
//     expanded: bool,

//     pub font: Font,
//     pub font_size: f32,
//     hover_index: usize,
//     item_height: f32,

//     pub text_color: Color,
//     pub bg_color: Color,

//     pub on_change: Arc<dyn Fn(&mut Self, Option<E>) + Send + Sync>,
// }
// impl<E:Dropdownable> Dropdown<E> {
//     pub fn new(pos: Vector2, width: f32, font_size: f32, text:&str, value:Option<E>, font: Font) -> Self {
//         let item_height = font_size + ITEM_Y_PADDING * 2.0;
//         let size = Vector2::new(width, font_size + Y_PADDING * 2.0);
//         let text = text.to_owned() + ": ";

//         Self {
//             pos,
//             size,
//             hover: false,
//             selected: false,
//             tag: String::new(),
//             keywords: text.split(" ").map(|a|a.to_lowercase().to_owned()).collect(),

//             text,
//             value,
//             expanded: false,

//             font,
//             font_size,

//             hover_index: 0,
//             item_height,

//             text_color: Color::BLACK,
//             bg_color: Color::WHITE,
            
//             on_change: Arc::new(|_,_|{}),
//         }
//     }
// }

// impl<E:Dropdownable> ScrollableItemGettersSetters for Dropdown<E> {
//     fn size(&self) -> Vector2 {
//         if self.expanded {
//             self.size + Vector2::new(0.0, self.item_height * E::variants().len() as f32 - ITEM_Y_PADDING)
//         } else {
//             self.size
//         }
//     }
//     fn get_pos(&self) -> Vector2 {self.pos}
//     fn set_pos(&mut self, pos:Vector2) {self.pos = pos}
//     fn get_tag(&self) -> String {self.tag.clone()}
//     fn set_tag(&mut self, tag:&str) {self.tag = tag.to_owned()}
//     fn get_hover(&self) -> bool {self.hover}
//     fn set_hover(&mut self, hover:bool) {self.hover = hover}
//     fn get_selected(&self) -> bool {self.selected}
//     fn set_selected(&mut self, selected:bool) {self.selected = selected}
// }

// impl<E:'static+Dropdownable> ScrollableItem for Dropdown<E> {
//     fn window_size_changed(&mut self, _new_window_size: Vector2) {}

//     fn get_value(&self) -> Box<dyn std::any::Any> { Box::new(self.value.clone()) }
//     fn get_keywords(&self) -> Vec<String> { self.keywords.clone() }

//     fn on_click_release(&mut self, _pos:Vector2, _button:MouseButton) {}
//     fn on_click(&mut self, _pos:Vector2, _button:MouseButton, _mods:KeyModifiers) -> bool {
//         if !self.hover && self.selected {
//             self.expanded = false;
//             self.selected = false;
//             return false;
//         }

//         if self.selected {
//             if self.expanded {
//                 // we were clicked
//                 self.expanded = false;
//                 // get the clicked item
//                 self.value = E::variants().get(self.hover_index).cloned();
//                 (self.on_change.clone())(self, self.value.clone());
//             } else {
//                 // expand self so the user can select the item
//                 self.expanded = true;
//             }

//             true
//         } else if self.hover {
//             self.selected = true;
//             true
//         } else {
//             false
//         }
//     }

//     fn on_mouse_move(&mut self, p:Vector2) {
//         self.check_hover(p);
        
//         if self.hover {
//             if p.y < self.pos.y + self.item_height {
//                 self.hover_index = 999;
//                 return;
//             }

//             let rel_y2 = (p.y - self.pos.y).abs() - self.size.y;
//             self.hover_index = ((rel_y2 + ITEM_Y_PADDING/2.0) / (self.item_height)).floor() as usize;
//         } else {
//             self.hover_index = 999;
//         }
//     }
//     fn on_text(&mut self, _text:String) {}

//     fn on_key_press(&mut self, _key:Key, _mods:KeyModifiers) -> bool {false}
//     fn on_key_release(&mut self, _key:Key) {}

//     fn on_scroll(&mut self, _delta:f32) -> bool {false}

//     fn update(&mut self) {}

//     fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
//         let pos = self.pos + pos_offset + Vector2::with_y(Y_PADDING);

//         // draw bounding box
//         list.push(Rectangle::new(
//             pos - Vector2::with_y(Y_PADDING),
//             self.size,
//             self.bg_color,
//             self.get_border_black(1.0)
//         ));

//         // draw item text
//         let item_text = Text::new(
//             pos,
//             self.font_size,
//             self.text.to_owned(),
//             self.text_color,
//             self.font.clone(),
//         );
//         let offset = Vector2::with_x(item_text.measure_text().x);
//         list.push(item_text);


//         let item_offset = Vector2::new(offset.x, 0.0);
//         let item_size = Vector2::new(self.size.x - offset.x, self.item_height - ITEM_Y_PADDING);

//         // draw selected option
//         let selected_text = self.value
//             .as_ref()
//             .and_then(|e|Some(e.display_text()))
//             .unwrap_or("--Select--".to_owned());
//         list.push(Text::new(
//             pos + item_offset,
//             self.font_size,
//             selected_text,
//             self.text_color,
//             self.font.clone(),
//         ));

//         // draw options
//         if self.expanded {
//             let y_size = Vector2::with_y(self.item_height);

//             for (mut index, text) in E::variants().iter().map(|e|e.display_text()).enumerate() {
//                 index += 1;

//                 // draw border 
//                 list.push(Rectangle::new(
//                     pos + item_offset + y_size * index as f32 - Vector2::with_y(ITEM_Y_PADDING),
//                     item_size,
//                     self.bg_color,
//                     Some(Border::new(
//                         if index-1 == self.hover_index {Color::BLUE} else {Color::BLACK}, 
//                         1.0
//                     ))
//                 ));
                
//                 // draw text
//                 list.push(Text::new(
//                     pos + item_offset + y_size * index as f32 - Vector2::with_y(ITEM_Y_PADDING),
//                     self.font_size,
//                     text,
//                     self.text_color,
//                     self.font.clone(),
//                 ));
//             }
//         }
//     }
// }



// mod test {
//     use crate::prelude::*;

//     #[derive(Debug, Clone, Copy, Dropdown)]
//     pub enum Test {
//         A,
//         B,
//         C
//     }

//     #[test]
//     fn test() {
//         let iters = Test::variants();
//         for i in iters {
//             let s = i.display_text();
//             println!("{}", s)
//         }
//     }
// }
