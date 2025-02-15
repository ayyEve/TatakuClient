use crate::prelude::*;


#[derive(Clone, Debug)]
pub struct GamepadInfo {
    pub id: GamepadId,
    pub name: Arc<String>,
    pub power_info: gilrs::PowerInfo,
    pub connected: bool,
}
