use std::any::Any;
use crate::prelude::*;

pub trait ScrollableItem: ScrollableItemGettersSetters + Any {
    fn window_size_changed(&mut self, _new_window_size: Vector2) {}
    fn ui_scale_changed(&mut self, _scale: Vector2) {}

    fn get_style(&self) -> Style;
    fn apply_layout(&mut self, layout: &LayoutManager, parent_pos: Vector2);
    fn update_layout(&self, _layout: &LayoutManager) {}

    /// fallback for when the item is not in a ScrollableArea
    fn check_hover(&mut self, p:Vector2) {
        let pos = self.get_pos();
        let size = self.size();
        self.set_hover(p.x > pos.x && p.x < pos.x + size.x && p.y > pos.y && p.y < pos.y + size.y)
    }

    /// returns none if not hovered or selected
    fn get_border_none(&self, border_radius: f32) -> Option<Border> {
        if self.get_hover() {
            Some(Border::new(Color::RED, border_radius))
        } else if self.get_selected() {
            Some(Border::new(Color::BLUE, border_radius))
        } else {
            None
        }
    }
    /// returns black border if not hovered or selected
    fn get_border_black(&self, border_radius: f32) -> Option<Border> {
        Some(self.get_border_none(border_radius).unwrap_or(Border::new(Color::BLACK, border_radius)))
    }

    fn update(&mut self) {}
    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection);

    /// check this object against the query to see if it should be included (true) or filtered (false)
    fn check_filter(&self, query: &Vec<String>, query_type: QueryType) -> bool { 
        let keywords = self.get_keywords();
        match query_type {
            QueryType::All => query.iter().all(|query_str|keywords.contains(query_str)),
            QueryType::Any => query.iter().any(|query_str|keywords.iter().any(|k|k.starts_with(query_str))),
        }
    }

    // input handlers

    /// when the mouse is clicked, returns the tag of the item clicked (or child of the item clicked)
    fn on_click(&mut self, _pos:Vector2, _button:MouseButton, _mods:KeyModifiers) -> bool { 
        self.get_hover()
        // self.on_click_tagged(pos, button, mods).is_some() 
    }
    
    /// when the mouse is clicked, returns the tag of the item clicked (or child of the item clicked)
    fn on_click_tagged(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers) -> Option<String> { 
        if self.on_click(pos, button, mods) {Some(self.get_tag())} else {None} 
        // if self.get_hover() {Some(self.get_tag())} else {None} 
    }

    /// when the mouse click is released
    fn on_click_release(&mut self, _pos:Vector2, _button:MouseButton) {}

    /// when the mouse is moved
    fn on_mouse_move(&mut self, p:Vector2) {self.check_hover(p)}

    /// when text is input
    fn on_text(&mut self, _text:String) {}

    /// when a key is pressed
    fn on_key_press(&mut self, _key:Key, _mods:KeyModifiers) -> bool {false}
    
    /// when a key is released TODO!
    fn on_key_release(&mut self, _key:Key) {}

    // when the mouse is scrolled
    fn on_scroll(&mut self, _delta:f32) -> bool {false}

    /// get the inner value
    fn get_value(&self) -> Box<dyn Any> {Box::new(0)}

    /// get a list of keywords for this object, primarily used for filtering
    fn get_keywords(&self) -> Vec<String> { Vec::new() }

    fn get_inner_tagged(&self, _tag: &String) -> Option<Vec<&Box<dyn ScrollableItem>>> { None }

}

/// helper trait for auto code generation
pub trait ScrollableItemGettersSetters: Send + Sync {
    fn size(&self) -> Vector2;
    fn set_size(&mut self, _new_size: Vector2) {}

    fn get_tag(&self) -> String { String::new() }
    fn set_tag(&mut self, _tag:&str) {}
    fn with_tag(mut self, tag:impl AsRef<str>) -> Self where Self:Sized {
        self.set_tag(tag.as_ref());
        self
    }

    fn get_pos(&self) -> Vector2 { Vector2::ZERO }
    fn set_pos(&mut self, _pos:Vector2) {}

    fn get_selected(&self) -> bool { false }
    fn set_selected(&mut self, _selected:bool) {}
    fn get_selectable(&self) -> bool { true }
    fn get_multi_selectable(&self) -> bool { false }

    fn get_hover(&self) -> bool { false }
    fn set_hover(&mut self, _hover:bool) {}
}

#[derive(Copy, Clone, Debug)]
pub enum QueryType {
    Any,
    All,
}
