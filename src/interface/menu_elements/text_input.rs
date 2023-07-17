use crate::prelude::*;
use std::error::Error;
use clipboard::{ClipboardProvider, ClipboardContext};

const KEY_REPEAT_DELAY:f32 = 500.0;
const KEY_REPEAT_INTERVAL:f32 = 1000.0 / 20.0; // x per second

const SELECTION_COLOR:Color = Color::BLUE_DIAMOND;
const SELECTION_COLOR_ALPHA:f32 = 0.8;

#[derive(Clone)]
pub struct TextInput {
    pos: Vector2,
    size: Vector2,
    hover: bool,
    selected: bool,
    tag: String,

    placeholder: String,
    text: String,
    cursor_index: usize,
    keywords: Vec<String>,

    pub is_password: bool,
    pub show_password_on_hover: bool,
    show_password: bool,

    pub font: Font,
    pub font_size: f32,


    selection_end: usize,

    /// key being held, when it started being held, when the last repeat was performed
    key_hold: Option<(Key, KeyModifiers, Instant, f32)>,

    hold_pos: Option<Vector2>,
    
    pub on_change: Arc<dyn Fn(&mut Self, String) + Send + Sync>,
}
impl TextInput { 
    pub fn new(pos:Vector2, size: Vector2, placeholder:&str, value:&str, font:Font) -> Self {
        Self {
            pos, 
            size, 
            keywords: placeholder.split(" ").map(|a|a.to_lowercase().to_owned()).collect(),
            placeholder: placeholder.to_owned(),

            hover: false,
            selected: false,
            text: value.to_owned(),
            cursor_index: 0,
            tag: String::new(),

            is_password: false,
            show_password_on_hover: false,
            show_password: false,

            font,
            font_size: size.y * 0.8,
            selection_end: 0,
            key_hold: None,
            hold_pos: None,
            
            on_change: Arc::new(|_,_|{}),
        }
    }

    pub fn get_text(&self) -> String { self.text.clone() }
    pub fn set_text(&mut self, text:String) {
        self.text = text.clone();
        self.cursor_index = text.len();
        self.selection_end = 0;

        (self.on_change.clone())(self, text.clone());
    }

    fn add_letter(&mut self, c:char) {
        if self.selection_end > 0 {
            self.remove_selected_text()
        }

        if self.cursor_index == self.text.len() {
            self.text.push(c);
        } else {
            self.text.insert(self.cursor_index, c);
        }

        self.cursor_index += 1;
    }

    fn remove_selected_text(&mut self) {
        // 
        let (start, end) = self.text.split_at(self.cursor_index);
        let (_snipped, end) = end.split_at(self.selection_end - self.cursor_index);

        self.text = start.to_owned() + end;
        self.selection_end = 0;
    }

    fn index_at_x(&self, x: f32) -> usize {
        if self.pos.x > x { return 0; }
        let rel_x = x - self.pos.x;

        let text;
        if self.is_password && !self.show_password {
            text = "*".repeat(self.text.len());
        } else {
            text = self.text.clone();
        }

        // cumulative width
        let mut width = 0.0;

        for (counter, char) in text.chars().enumerate() {
            // get the font character
            let Some(c) = self.font.get_character(self.font_size, char) else { continue };
            
            width += c.advance_width() / 2.0;
            
            if rel_x < width { return counter; }

            width += c.advance_width() / 2.0;
        }

        self.text.len()
    }
}
impl ScrollableItemGettersSetters for TextInput {
    fn size(&self) -> Vector2 {self.size}
    fn get_pos(&self) -> Vector2 {self.pos}
    fn set_pos(&mut self, pos:Vector2) {self.pos = pos}
    fn get_tag(&self) -> String {self.tag.clone()}
    fn set_tag(&mut self, tag:&str) {self.tag = tag.to_owned()}
    fn get_hover(&self) -> bool {self.hover}
    fn set_hover(&mut self, hover:bool) {self.hover = hover; self.show_password = hover && self.show_password_on_hover}
    fn get_selected(&self) -> bool {self.selected}
    fn set_selected(&mut self, selected:bool) {self.selected = selected}
}
impl ScrollableItem for TextInput {
    fn get_value(&self) -> Box<dyn std::any::Any> { Box::new(self.text.clone()) }
    fn get_keywords(&self) -> Vec<String> { self.keywords.clone() }

    fn draw(&mut self, mut pos_offset:Vector2, list:&mut RenderableCollection) {
        list.push(Rectangle::new(
            self.pos + pos_offset,
            self.size, 
            Color::WHITE,
            Some(Border::new(if self.hover {Color::RED} else if self.selected {Color::BLUE} else {Color::BLACK}, 1.2))
        ));

        // offset text pos to be y centered
        pos_offset.y += (self.size.y-self.size.y) / 2.0;

        let text = if self.is_password && !self.show_password {
            "*".repeat(self.text.len())
        } else {
            self.text.clone()
        };

        if text.len() > 0 {
            list.push(Text::new(
                self.pos + pos_offset,
                self.font_size,
                text.clone(),
                Color::BLACK,
                self.font.clone()
            ));
        } else {
            list.push(Text::new(
                self.pos + pos_offset,
                self.font_size,
                self.placeholder.clone(),
                Color::new(0.2,0.2,0.2,1.0),
                self.font.clone()
            ));
        }

        let width = Text::new(
            self.pos + pos_offset,
            self.font_size,
            text.split_at(self.cursor_index).0.to_owned(),
            Color::BLACK,
            self.font.clone()
        ).measure_text().x;

        // cursor if no text is selected
        if self.selected && self.selection_end == 0 {
            list.push(Rectangle::new(
                self.pos + pos_offset + Vector2::new(width, 0.0),
                Vector2::new(0.7, self.font_size), 
                Color::RED,
                Some(Border::new(Color::RED, 1.2))
            ));
        }

        // draw rectangle around selected items
        if self.selection_end > self.cursor_index {
            let (start, draw_this) = text.split_at(self.cursor_index);
            let (draw_this, _end) = draw_this.split_at(self.selection_end - self.cursor_index);
            
            let start_offset = Text::new(
                self.pos,
                self.font_size,
                start.to_owned(),
                Color::BLACK,
                self.font.clone()
            ).measure_text().x;

            let width = Text::new(
                self.pos,
                self.font_size,
                draw_this.to_owned(),
                Color::BLACK,
                self.font.clone()
            ).measure_text().x;


            list.push(Rectangle::new(
                self.pos + pos_offset + Vector2::with_x(start_offset), 
                Vector2::new(width, self.size.y), 
                SELECTION_COLOR.alpha(SELECTION_COLOR_ALPHA), 
                Some(Border::new(SELECTION_COLOR, 1.0))
            ))
        }

    }
    
    fn on_click(&mut self, pos:Vector2, _btn:MouseButton, _mods:KeyModifiers) -> bool {
        self.show_password = false;

        // try to extrapolate where the mouse was clicked, and change the cursor_index to that
        if self.selected {
            if !self.hover {
                self.selected = false;
                return false;
            }

            self.hold_pos = Some(pos);
            self.cursor_index = self.index_at_x(pos.x);
            self.selection_end = 0;
            
            return true;
        }

        if self.hover {
            self.selected = true;
            self.hold_pos = Some(pos);
        }

        return self.hover
    }
    fn on_mouse_move(&mut self, p:Vector2) {
        self.check_hover(p);

        // if the mouse is being held, we want to do a selection
        if let Some(hold_pos) = self.hold_pos {
            let i = self.index_at_x(p.x);
            // if the mouse-pos-index is greater than the start of the selection
            // it becomes the end of the selection
            if i > self.cursor_index {
                self.selection_end = i;
            } else {
                // otherwise, it becomes the start of the selection

                // set the end of the selection to where the hold started
                self.selection_end = self.index_at_x(hold_pos.x);

                // set the start of the selection to the mouse-pos-index
                self.cursor_index = i;
            }
            
        }
    }

    fn on_click_release(&mut self, _pos:Vector2, _button:MouseButton) {
        if self.hold_pos.is_some() {
            self.hold_pos = None;
        }
    }


    fn on_key_press(&mut self, key:Key, mods:KeyModifiers) -> bool {
        self.show_password = false;
        if !self.selected { return false }

        if mods.alt {
            self.show_password = true;
            return true;
        }

        if let Some((k, _,_,_)) = &self.key_hold {
            if k != &key {
                self.key_hold = Some((key, mods, Instant::now(), 0.0));
            }
        } else {
            self.key_hold = Some((key, mods, Instant::now(), 0.0));
        }

        // reset selection if its not a selection modifier
        if (key == Key::Left || key == Key::Right) && !mods.shift {
            // println!("before: {}-{}", self.cursor_index, self.selection_end);
            self.selection_end = 0;
        }

        match key {
            Key::Left if mods.shift => {
                if self.selection_end > self.cursor_index {
                    self.selection_end -= 1
                } 
                // else if self.cursor_index > 0 {
                //     // selection will be were cursor index was, and cursor index will reduce (selection expands to the left)
                //     if self.selection_end == 0 {
                //         self.selection_end = self.cursor_index;
                //     }
                //     self.cursor_index -= 1;
                // }
            }
            Key::Right if mods.shift => {
                if self.selection_end < self.cursor_index {self.selection_end = self.cursor_index}
                if self.selection_end < self.text.len() {self.selection_end += 1}
            }

            Key::Left if self.cursor_index > 0 => self.cursor_index -= 1,
            Key::Right if self.cursor_index < self.text.len() => self.cursor_index += 1,
            
            Key::Back if self.cursor_index > 0 => {
                if self.selection_end > 0 {
                    self.remove_selected_text();
                } else {
                    if self.cursor_index < self.text.len() {
                        self.text.remove(self.cursor_index-1);
                    } else {
                        self.text.pop();
                    }
                    self.cursor_index -= 1;
                }

                (self.on_change.clone())(self, self.text.clone());
            }
            Key::Delete if self.cursor_index < self.text.len() => {
                if self.selection_end > 0 {
                    self.remove_selected_text();
                } else {
                    self.text.remove(self.cursor_index);
                }
                
                (self.on_change.clone())(self, self.text.clone());
            }
            
            Key::V if mods.ctrl => {
                let ctx:Result<ClipboardContext, Box<dyn Error>> = ClipboardProvider::new();
                match ctx {
                    Ok(mut ctx) => 
                        match ctx.get_contents() {
                            Ok(text) => self.set_text(text),
                            Err(e) => println!("[Clipboard] Error: {:?}", e),
                        }
                    Err(e) => println!("[Clipboard] Error: {:?}", e),
                }
            }
            
            Key::A if mods.ctrl => {
                self.cursor_index = 0;
                self.selection_end = self.text.len();
            }
            _ => {}
        }

        true
    }
    fn update(&mut self) {
        let mut should_repeat = None;

        if let Some((key, mods, since_start, last_time)) = &mut self.key_hold {
            let elapsed = since_start.elapsed().as_secs_f32() * 1000.0;

            if elapsed >= KEY_REPEAT_DELAY {
                if elapsed - *last_time >= KEY_REPEAT_INTERVAL {
                    // perform a repeat
                    should_repeat = Some((*key, *mods));
                    *last_time = elapsed;
                }
            }
        }

        if let Some((k, mods)) = should_repeat {
            self.on_key_press(k, mods);
        }
    }
    
    fn on_key_release(&mut self, _key:Key) {
        if self.key_hold.is_some() {
            self.key_hold = None;
        }
    }
    fn on_text(&mut self, text:String) {
        if !self.selected { return }

        for c in text.chars() { self.add_letter(c) }
        if text.len() > 0 {
            (self.on_change.clone())(self, self.text.clone());
        }
    }
}
