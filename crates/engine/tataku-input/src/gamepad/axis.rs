use crate::prelude::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct AxisState {
    pub value: f32,
    pub changed: bool,
}

#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AxisConfig {
    pub axis_id: Axis,
    pub threshhold: f64
}
