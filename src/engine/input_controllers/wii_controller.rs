#![allow(unused)]
use crate::prelude::*;

pub struct WiiController {
    id: u32,
    name: Arc<String>
}
impl WiiController {
    pub fn new(id: u32, name: Arc<String>) -> Self {Self {id, name}}
}
impl Controller for WiiController {
    fn get_id(&self) -> u32 {self.id}
    fn get_name(&self) -> Arc<String> {self.name.clone()}

    
    fn map_axis(&self, _axis: u8) -> Option<ControllerAxis> {
        // TODO:!
        None
    }

    fn map_button(&self, button: u8) -> Option<ControllerButton> {
        match button {
            // 0  => Some("1"),
            // 1  => Some("2"),
            2  => Some(ControllerButton::A),
            3  => Some(ControllerButton::B),
            4  => Some(ControllerButton::Select),
            5  => Some(ControllerButton::Start),
            // 6  => Some("Z"),
            // 7  => Some("C"),
            11 => Some(ControllerButton::Home),
            12 => Some(ControllerButton::DPad_Up),
            13 => Some(ControllerButton::DPad_Right),
            14 => Some(ControllerButton::DPad_Down),
            15 => Some(ControllerButton::DPad_Left),

            _ => None
        }
    }
    
    fn label_button(&self, button: u8) -> Option<&'static str> {
        match button {
            0  => Some("1"),
            1  => Some("2"),
            2  => Some("A"),
            3  => Some("B"),
            4  => Some("-"),
            5  => Some("+"),
            6  => Some("Z"),
            7  => Some("C"),
            // 8  => "?",
            // 9  => "?",
            // 10 => "?",
            11 => Some("Home"),
            12 => Some("D-Pad Up"),
            13 => Some("D-Pad Right"),
            14 => Some("D-Pad Down"),
            15 => Some("D-Pad Left"),

            _ => None
        }
    }
}

