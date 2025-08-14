use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct GamepadInfo {
    pub id: GamepadId,
    pub name: ArcStr,
    pub power_info: gilrs::PowerInfo,
    pub connected: bool,
}
