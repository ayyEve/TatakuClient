use crate::prelude::*;

pub struct XboxController {
    id: u32,
    name: Arc<String>
}
impl XboxController {
    pub fn new(id: u32, name: Arc<String>) -> Self {Self {id, name}}
}
impl Controller for XboxController {
    fn get_id(&self) -> u32 {self.id}
    fn get_name(&self) -> Arc<String> {self.name.clone()}

    
    fn map_axis(&self, axis: u8) -> Option<ControllerAxis> {
        match axis {
            0 => Some(ControllerAxis::Left_X),
            1 => Some(ControllerAxis::Left_Y),
            2 => Some(ControllerAxis::Right_X),
            3 => Some(ControllerAxis::Right_Y),
            4 => Some(ControllerAxis::Left_Trigger),
            5 => Some(ControllerAxis::Right_Trigger),

            _ => None
        }
    }

    fn map_button(&self, button: u8) -> Option<ControllerButton> {
        match button {
            0  => Some(ControllerButton::A),
            1  => Some(ControllerButton::B),
            2  => Some(ControllerButton::X),
            3  => Some(ControllerButton::Y),
            4  => Some(ControllerButton::Left_Bumper),
            5  => Some(ControllerButton::Right_Bumper),
            6  => Some(ControllerButton::Select),
            7  => Some(ControllerButton::Start),
            // 8  => Some(ControllerButton::LeftStickDown),
            // 9  => Some(ControllerButton::RightStickDown),
            10 => Some(ControllerButton::DPad_Up),
            11 => Some(ControllerButton::DPad_Right),
            12 => Some(ControllerButton::DPad_Down),
            13 => Some(ControllerButton::DPad_Left),

            _ => None
        }
    }
    
    fn label_button(&self, button: u8) -> Option<&'static str> {
        match button {
            0  => Some("A"),
            1  => Some("B"),
            2  => Some("X"),
            3  => Some("Y"),
            4  => Some("LB"),
            5  => Some("RB"),
            6  => Some("Select/Options"),
            7  => Some("Start"),
            8  => Some("Left Stick Down"),
            9  => Some("Right Stick Down"),
            10 => Some("D-Pad Up"),
            11 => Some("D-Pad Right"),
            12 => Some("D-Pad Down"),
            13 => Some("D-Pad Left"),

            _ => None
        }
    }
}