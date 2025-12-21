use crate::*;
use input::{
    Key,
    KeyModifiers,
    GamepadButton,
    InputEvent,
    InputType,
};

#[derive(Debug)]
pub struct CurrentInputState {
    pub mouse_pos: Vector2,
    pub mouse_moved: bool,
    pub mods: KeyModifiers,
    pub window_focus_changed: Option<bool>,
    pub controller_pause: bool,

    pub events: Vec<InputType>,
}
impl CurrentInputState {
    pub fn keys_down(&self) -> impl Iterator<Item=Key> {
        self.events
            .iter()
            .filter_map(|e| if let InputType::KeyPress(k) = e {
                k.as_key()
            } else { None })
    }
    pub fn controller_down(&self) -> impl Iterator<Item=&GamepadButton> {
        self.events
            .iter()
            .filter_map(|e| if let InputType::ControllerPress(b, _, _) = e {
                Some(b)
            } else { None })
    }

    pub fn into_events(self) -> Vec<InputEvent> {
        [
            self.mouse_moved.then_some(InputType::MouseMove(self.mouse_pos)),
            // (self.scroll_delta.x.abs() > f32::EPSILON || self.scroll_delta.y.abs() > f32::EPSILON).then_some(InputType::MouseScroll(self.scroll_delta))
        ]
            .into_iter()
            .flatten()
            .chain(self.events)
            .map(|event| InputEvent { 
                event, 
                mouse_pos: self.mouse_pos, 
                key_mods: self.mods 
            })
            .collect()
    }
}
