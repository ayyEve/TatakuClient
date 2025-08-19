use crate::prelude::*;

// you might be wondering why i dont just use gilrs::Button
#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[serde(rename_all="camelCase")]
pub enum ControllerButton {
    // action buttons
    North,
    South,
    East, 
    West,
    C,
    Z,

    // menu buttons
    Start,
    Select,
    Mode,

    // dpad buttons
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    
    // triggers and bumpers
    LeftTrigger,
    LeftBumper,
    RightTrigger,
    RightBumper,

    // thumbs
    LeftThumb,
    RightThumb,

    // dunno
    Unknown,
}

impl ControllerButton {
    pub fn from_string(s: &str) -> Self {
        match &*s.to_lowercase() {
            "north" => Self::North,
            "south" => Self::South,
            "east" => Self::East, 
            "west" => Self::West,
            "c" => Self::C,
            "z" => Self::Z,

            "start" => Self::Start,
            "select" => Self::Select,
            "mode" => Self::Mode,

            "up" | "dpad_up" | "dpadup" => Self::DPadUp,
            "down" | "dpad_down" | "dpaddown" => Self::DPadDown,
            "left" | "dpad_left" | "dpadleft" => Self::DPadLeft,
            "right" | "dpad_right" | "dpadright" => Self::DPadRight,
            
            "left_trigger" | "lefttrigger" | "ltrigger" => Self::LeftTrigger,
            "left_bumper" | "leftbumper" | "lbumper" => Self::LeftBumper,
            "right_trigger" | "righttrigger" | "rtrigger" => Self::RightTrigger,
            "right_bumper" | "rightbumper" | "rbumper" => Self::RightBumper,

            "left_thumb" | "leftthumb" | "lthumb" => Self::LeftThumb,
            "right_thumb" | "rightthumb" | "rthumb" => Self::RightThumb,

            _ => Self::Unknown,
        }
    }

}

impl From<gilrs::Button> for ControllerButton {
    fn from(value: gilrs::Button) -> Self {
        use gilrs::Button;

        match value {
            Button::South => ControllerButton::South,
            Button::East => ControllerButton::East,
            Button::North => ControllerButton::North,
            Button::West => ControllerButton::West,
            Button::C => ControllerButton::C,
            Button::Z => ControllerButton::Z,

            // should these be swapped?
            Button::LeftTrigger => ControllerButton::LeftTrigger,
            Button::LeftTrigger2 => ControllerButton::LeftBumper,
            Button::RightTrigger => ControllerButton::RightTrigger,
            Button::RightTrigger2 => ControllerButton::RightBumper,
            Button::Select => ControllerButton::Select,
            Button::Start => ControllerButton::Start,
            Button::Mode => ControllerButton::Mode,
            Button::LeftThumb => ControllerButton::LeftThumb,
            Button::RightThumb => ControllerButton::RightThumb,
            Button::DPadUp => ControllerButton::DPadUp,
            Button::DPadDown => ControllerButton::DPadDown,
            Button::DPadLeft => ControllerButton::DPadLeft,
            Button::DPadRight => ControllerButton::DPadRight,
            Button::Unknown => ControllerButton::Unknown,
        }
    }
}
