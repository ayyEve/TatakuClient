use crate::prelude::*;

pub struct TaikoController {
    id: u32,
    name: Arc<String>
}
impl TaikoController {
    pub fn new(id: u32, name: Arc<String>) -> Self {Self{id, name}}
}
impl Controller for TaikoController {
    fn get_id(&self) -> u32 {self.id}
    fn get_name(&self) -> Arc<String> {self.name.clone()}

    fn map_button(&self, button: u8) -> Option<ControllerButton> {
        match button {
            0  => Some(ControllerButton::X),
            1  => Some(ControllerButton::A),
            2  => Some(ControllerButton::B),
            3  => Some(ControllerButton::Y),
            4  => Some(ControllerButton::Left_Bumper),
            5  => Some(ControllerButton::Right_Bumper),
            // 6  => Some(ControllerButton::),
            // 7  => Some(ControllerButton::),
            8  => Some(ControllerButton::Select),
            9  => Some(ControllerButton::Start),
            // 10 => Some(ControllerButton::),
            // 11 => Some(ControllerButton::),
            12 => Some(ControllerButton::Home),

            // 13 => Some(ControllerButton::),
            14 => Some(ControllerButton::DPad_Up),
            15 => Some(ControllerButton::DPad_Right),
            16 => Some(ControllerButton::DPad_Down),
            17 => Some(ControllerButton::DPad_Left),

            _ => None
        }
    }
    
    fn label_button(&self, button: u8) -> Option<&'static str> {
        match button {
            0  => Some("Y"),
            1  => Some("B"),
            2  => Some("A"),
            3  => Some("X"),
            4  => Some("L"),
            5  => Some("R"),
            6  => Some("Outer Left"),
            7  => Some("Outer Right"),
            8  => Some("Minus"),
            9  => Some("Plus"),
            10 => Some("Inner Left"),
            11 => Some("Inner Right"),
            12 => Some("Home"),
            13 => Some("Share"),
            14 => Some("D-Pad Up"),
            15 => Some("D-Pad Right"),
            16 => Some("D-Pad Down"),
            17 => Some("D-Pad Left"),
            _ => None
        }
    }
}