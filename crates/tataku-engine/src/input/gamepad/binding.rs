use crate::prelude::*;


#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct ControllerBinding {
    pub button: Option<ControllerButton>,
    pub axis: Option<AxisConfig>
}
impl ControllerBinding {
    pub fn new(button: Option<ControllerButton>, axis: Option<AxisConfig>) -> Self {
        Self {
            button, 
            axis
        }
    }

    pub fn check_button(&self, button: ControllerButton) -> bool {
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
impl From<ControllerButton> for ControllerBinding {
    fn from(value: ControllerButton) -> Self {
        Self {
            button: Some(value),
            axis: None,
        }
    }
}
