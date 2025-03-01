use crate::prelude::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct AxisState {
    pub value: f32,
    pub changed: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct AxisConfig {
    pub axis_id: Axis,
    pub threshhold: f64
}
