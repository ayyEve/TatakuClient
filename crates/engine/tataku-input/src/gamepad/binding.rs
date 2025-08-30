use crate::prelude::*;

#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ControllerBinding {
    pub button: Option<GamepadButton>,
    pub axis: Option<AxisConfig>
}
impl ControllerBinding {
    pub fn new(button: Option<GamepadButton>, axis: Option<AxisConfig>) -> Self {
        Self {
            button, 
            axis
        }
    }

    pub fn check_button(&self, button: GamepadButton) -> bool {
        if let Some(b) = self.button {
            b == button
        } else {
            false
        }
    }
}

impl From<Axis> for ControllerBinding {
    fn from(value: Axis) -> Self {
        Self {
            button: None,
            axis: Some(AxisConfig { axis_id: value, threshhold: 0.0 })
        }
    }
}
impl From<GamepadButton> for ControllerBinding {
    fn from(value: GamepadButton) -> Self {
        Self {
            button: Some(value),
            axis: None,
        }
    }
}
