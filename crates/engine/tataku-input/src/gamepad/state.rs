use crate::*;

const AXES: &[Axis] = &[
    Axis::LeftStickX, Axis::LeftStickY, Axis::LeftTrigger,
    Axis::RightStickX, Axis::RightStickY, Axis::RightTrigger,
    Axis::DPadX, Axis::DPadY
];

#[derive(Clone, Debug)]
pub struct GamepadState {
    /// gamepad info
    pub info: GamepadInfo,

    /// list of currently-pressed buttons
    pub buttons: HashSet<GamepadButton>,

    /// list of buttons that have been released since the last update
    pub buttons_up: HashSet<GamepadButton>,

    /// list of buttons that have been pressed since the last update
    pub buttons_down: HashSet<GamepadButton>,

    /// current axes states
    pub axis: HashMap<Axis, AxisState>,


    pub power_info: gilrs::PowerInfo,
}
impl GamepadState {
    pub fn new(info: GamepadInfo) -> Self {
        Self {
            info,
            buttons: HashSet::new(),
            buttons_up: HashSet::new(),
            buttons_down: HashSet::new(),
            axis: AXES.iter()
                .map(|a| (*a, AxisState::default()))
                .collect(),
            power_info: gilrs::PowerInfo::Unknown,
        }
    }
}

