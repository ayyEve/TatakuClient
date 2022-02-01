use crate::prelude::*;

pub struct SteamController {
    id: u32,
    name: Arc<String>
}
impl SteamController {
    pub fn new(id: u32, name: Arc<String>) -> Self {Self {id, name}}
}
impl Controller for SteamController {
    fn get_id(&self) -> u32 {self.id}
    fn get_name(&self) -> Arc<String> {self.name.clone()}

    fn map_button(&self, button: u8) -> Option<ControllerButton> {
        match button {
            // 0  => Some(ControllerButton::),
            // 1  => Some(ControllerButton::),
            2  => Some(ControllerButton::A),
            3  => Some(ControllerButton::B),
            4  => Some(ControllerButton::X),
            5  => Some(ControllerButton::Y),
            6  => Some(ControllerButton::Left_Bumper),
            7  => Some(ControllerButton::Right_Bumper),
            // 8  => Some(ControllerButton::),
            // 27 => Some(ControllerButton::),
            // 23 => Some(ControllerButton::),
            // 9  => Some(ControllerButton::),
            // 26 => Some(ControllerButton::),
            // 22 => Some(ControllerButton::),
            10 => Some(ControllerButton::Select),
            11 => Some(ControllerButton::Start),
            12 => Some(ControllerButton::Home),
            // 13 => Some(ControllerButton::Left_Stick_Click),
            // 14 => Some(ControllerButton::Right_Stick_Click),
            // 15 => Some(ControllerButton::),
            // 16 => Some(ControllerButton::),
            17 => Some(ControllerButton::DPad_Up),
            18 => Some(ControllerButton::DPad_Down),
            19 => Some(ControllerButton::DPad_Right),
            20 => Some(ControllerButton::DPad_Left),

            _ => None
        }
    }
    
    fn label_button(&self, button: u8) -> Option<&'static str> {
        match button {
            0  => Some("Left Touchpad Touch"),
            1  => Some("Right Touchpad Touch"),
            2  => Some("A"),
            3  => Some("B"),
            4  => Some("X"),
            5  => Some("Y"),
            6  => Some("Left Bumper"),
            7  => Some("Right Bumper"),
            8  => Some("Left Trigger Hard"),
            27 => Some("Left Trigger Mid"), // should these
            23 => Some("Left Trigger Soft"), // be swapped?
            9  => Some("Right Trigger Hard"),
            26 => Some("Right Trigger Mid"), // should these
            22 => Some("Right Trigger Soft"), // be swapped?
            10 => Some("Back"),
            11 => Some("Forward"),
            12 => Some("Steam Button"),
            13 => Some("Analog Click"),
            14 => Some("Right Touchpad Click"),
            15 => Some("Left Rear"),
            16 => Some("Right Rear"),
            17 => Some("D-Pad Up"),
            18 => Some("D-Pad Down"),
            19 => Some("D-Pad Right"),
            20 => Some("D-Pad Left"),

            _ => None
        }
    }
}