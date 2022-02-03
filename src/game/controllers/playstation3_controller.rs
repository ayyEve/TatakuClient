use crate::prelude::*;

pub struct Playstation3Controller {
    id: u32,
    name: Arc<String>
}
impl Playstation3Controller {
    pub fn new(id: u32, name: Arc<String>) -> Self {Self {id, name}}
}
impl Controller for Playstation3Controller {
    fn get_id(&self) -> u32 {self.id}
    fn get_name(&self) -> Arc<String> {self.name.clone()}

    
    fn map_axis(&self, _axis: u8) -> Option<ControllerAxis> {
        // TODO:!
        None
    }

    fn map_button(&self, button: u8) -> Option<ControllerButton> {
        match button {
            0 =>  Some(ControllerButton::A),
            1 =>  Some(ControllerButton::B),
            2 =>  Some(ControllerButton::X),
            3 =>  Some(ControllerButton::Y),
            4 =>  Some(ControllerButton::Left_Bumper),
            5 =>  Some(ControllerButton::Right_Bumper),
            // 6 =>  Some(ControllerButton::Left_Trigger),
            // 7 =>  Some(ControllerButton::Right_Trigger),
            8 =>  Some(ControllerButton::Select),
            9 =>  Some(ControllerButton::Start),
            10 => Some(ControllerButton::Home),
            // 11 => Some(ControllerButton::Left_Stick_Down),
            // 12 => Some(ControllerButton::Right_Stick_Down),
            13 => Some(ControllerButton::DPad_Up),
            14 => Some(ControllerButton::DPad_Right),
            15 => Some(ControllerButton::DPad_Down),
            16 => Some(ControllerButton::DPad_Left),

            _ => None
        }
    }
    
    fn label_button(&self, button: u8) -> Option<&'static str> {
        match button {
            0 =>  Some("Cross"),
            1 =>  Some("Circle"),
            2 =>  Some("Triangle"),
            3 =>  Some("Square"),
            4 =>  Some("L1"),
            5 =>  Some("R1"),
            6 =>  Some("L2"),
            7 =>  Some("R2"),
            8 =>  Some("Select"),
            9 =>  Some("Start"),
            10 => Some("Home"),
            11 => Some("L3"),
            12 => Some("R3"),
            13 => Some("D-Pad Up"),
            14 => Some("D-Pad Down"),
            15 => Some("D-Pad Left"),
            16 => Some("D-Pad Right"),

            _ => None
        }
    }
}