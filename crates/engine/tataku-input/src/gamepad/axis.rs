use crate::*;
use common::reflect::*;
use common::macros::Reflect;

#[derive(Copy, Clone, Debug, Default)]
pub struct AxisState {
    pub value: f32,
    pub changed: bool,
}

#[derive(Reflect)]
#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AxisConfig {
    pub axis_id: Axis,
    pub threshhold: f64
}

#[derive(Reflect)]
#[derive(Serialize, Deserialize)]
#[reflect(display="display")]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Axis {
    LeftStickX,
    LeftStickY,
    LeftTrigger,
    RightStickX,
    RightStickY,
    RightTrigger,
    DPadX,
    DPadY,
    Unknown,
}
impl From<gilrs::Axis> for Axis {
    fn from(value: gilrs::Axis) -> Self {
        match value {
            gilrs::Axis::LeftStickX => Self::LeftStickX,
            gilrs::Axis::LeftStickY => Self::LeftStickY,
            gilrs::Axis::LeftZ => Self::LeftTrigger,
            gilrs::Axis::RightStickX => Self::RightStickX,
            gilrs::Axis::RightStickY => Self::RightStickY,
            gilrs::Axis::RightZ => Self::RightTrigger,
            gilrs::Axis::DPadX => Self::DPadX,
            gilrs::Axis::DPadY => Self::DPadY,
            gilrs::Axis::Unknown => Self::Unknown,
        }
    }
}

impl std::fmt::Display for Axis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeftStickX => "Left Stick X".fmt(f),
            Self::LeftStickY => "Left Stick Y".fmt(f),
            Self::LeftTrigger => "Left Trigger".fmt(f),
            Self::RightStickX => "Right Stick X".fmt(f),
            Self::RightStickY => "Right Stick Y".fmt(f),
            Self::RightTrigger => "Right Trigger".fmt(f),
            Self::DPadX => "DPad X".fmt(f),
            Self::DPadY => "DPad Y".fmt(f),
            Self::Unknown => "Unknown".fmt(f),
        }
    }
}
