use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct GamepadState {
    /// gamepad info
    pub info: GamepadInfo,

    /// list of currently-pressed buttons
    pub buttons: HashSet<ControllerButton>,

    /// list of buttons that have been released since the last update
    pub buttons_up: HashSet<ControllerButton>,

    /// list of buttons that have been pressed since the last update
    pub buttons_down: HashSet<ControllerButton>,

    /// current axes states
    pub axis: HashMap<gilrs::Axis, AxisState>,


    pub power_info: gilrs::PowerInfo,
}
impl GamepadState {
    pub fn new(info: GamepadInfo) -> Self {
        Self {
            info,
            buttons: HashSet::new(),
            buttons_up: HashSet::new(),
            buttons_down: HashSet::new(),
            axis: [
                Axis::LeftStickX, Axis::LeftStickY, Axis::LeftZ,
                Axis::RightStickX, Axis::RightStickY, Axis::RightZ,
                Axis::DPadX, Axis::DPadY
            ].into_iter().map(|a| (a, AxisState::default())).collect(),
            power_info: gilrs::PowerInfo::Unknown,
        }
    }
}
