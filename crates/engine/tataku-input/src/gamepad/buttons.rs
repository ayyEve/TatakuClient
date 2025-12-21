use crate::*;
use common::reflect::*;
use common::macros::Reflect;

#[derive(Reflect)]
#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[serde(rename_all="camelCase")]
pub enum GamepadButton {
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
    Unknown(u32),
}

impl From<(gilrs::Button, gilrs::ev::Code)> for GamepadButton {
    fn from((value, code): (gilrs::Button, gilrs::ev::Code)) -> Self {
        use gilrs::Button;

        match value {
            Button::South => GamepadButton::South,
            Button::East => GamepadButton::East,
            Button::North => GamepadButton::North,
            Button::West => GamepadButton::West,
            Button::C => GamepadButton::C,
            Button::Z => GamepadButton::Z,

            // should these be swapped?
            Button::LeftTrigger => GamepadButton::LeftTrigger,
            Button::LeftTrigger2 => GamepadButton::LeftBumper,
            Button::RightTrigger => GamepadButton::RightTrigger,
            Button::RightTrigger2 => GamepadButton::RightBumper,
            Button::Select => GamepadButton::Select,
            Button::Start => GamepadButton::Start,
            Button::Mode => GamepadButton::Mode,
            Button::LeftThumb => GamepadButton::LeftThumb,
            Button::RightThumb => GamepadButton::RightThumb,
            Button::DPadUp => GamepadButton::DPadUp,
            Button::DPadDown => GamepadButton::DPadDown,
            Button::DPadLeft => GamepadButton::DPadLeft,
            Button::DPadRight => GamepadButton::DPadRight,
            Button::Unknown => GamepadButton::Unknown(code.into_u32()),
        }
    }
}

impl std::str::FromStr for GamepadButton {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        Ok(match &*s.to_lowercase() {
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

            s => Self::Unknown(s.parse().unwrap_or_default()),
        })
    }
}
