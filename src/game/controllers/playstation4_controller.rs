use crate::prelude::*;

pub struct Playstation4Controller {
    id: u32,
    name: Arc<String>
}
impl Playstation4Controller {
    pub fn new(id: u32, name: Arc<String>) -> Self {Self {id, name}}
}
impl Controller for Playstation4Controller {
    fn get_id(&self) -> u32 {self.id}
    fn get_name(&self) -> Arc<String> {self.name.clone()}

    fn map_button(&self, button: u8) -> Option<ControllerButton> {
        match button {
            0 =>  Some(ControllerButton::X),
            1 =>  Some(ControllerButton::A),
            2 =>  Some(ControllerButton::B),
            3 =>  Some(ControllerButton::Y),
            4 =>  Some(ControllerButton::Left_Bumper),
            5 =>  Some(ControllerButton::Right_Bumper),
            // 6 =>  Some(ControllerButton::Left_Trigger),
            // 7 =>  Some(ControllerButton::Right_Trigger),
            8 =>  Some(ControllerButton::Select),
            9 =>  Some(ControllerButton::Start),
            // 10 => Some(ControllerButton::Left_Stick_Down),
            // 11 => Some(ControllerButton::Right_Stick_Down),
            12 => Some(ControllerButton::Home),
            // 13 => Some(ControllerButton::Touchpad),
            14 => Some(ControllerButton::DPad_Up),
            15 => Some(ControllerButton::DPad_Down),
            16 => Some(ControllerButton::DPad_Left),
            17 => Some(ControllerButton::DPad_Right),

            _ => None
        }
    }
    
    fn label_button(&self, button: u8) -> Option<&'static str> {
        match button {
            0 =>  Some("Square"),
            1 =>  Some("Cross"),
            2 =>  Some("Circle"),
            3 =>  Some("Triangle"),
            4 =>  Some("L1"),
            5 =>  Some("R1"),
            6 =>  Some("L2"),
            7 =>  Some("R2"),
            8 =>  Some("Share"),
            9 =>  Some("Start"),

            10 => Some("L3"),
            11 => Some("R3"),
            12 => Some("Home"),
            13 => Some("Touchpad Click"),
            14 => Some("D-Pad Up"),
            15 => Some("D-Pad Down"),
            16 => Some("D-Pad Left"),
            17 => Some("D-Pad Right"),

            _ => None
        }
    }
}