use crate::prelude::*;

pub trait Controller {
    fn get_id(&self) -> u32;
    fn get_name(&self) -> Arc<String>;

    fn map_axis(&self, axis: u8) -> Option<ControllerAxis>;
    fn map_button(&self, button: u8) -> Option<ControllerButton>;

    fn label_button(&self, _button: u8) -> Option<&'static str> {None}
}

#[allow(non_camel_case_types, dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
/// these buttons follow xbox notation
/// because thats what im used to
pub enum ControllerButton {
    /// Cross for Sony
    A,
    /// Circle for Sony
    B,
    /// Square for Sony
    X,
    /// Triangle for Sony
    Y,

    Start,
    Select,
    /// xbox button, ps button, etc
    Home,

    /// L1 for Sony
    Left_Bumper,
    /// R1 for Sony
    Right_Bumper,

    DPad_Up,
    DPad_Down,
    DPad_Left,
    DPad_Right,
}


#[allow(non_camel_case_types, dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum ControllerAxis {
    Left_X,
    Left_Y,
    Right_X,
    Right_Y,
    Left_Trigger,
    Right_Trigger
}

/// this is a very sad controller indeed
pub struct GenericController {
    id: u32,
    name: Arc<String>
} 
impl GenericController {
    pub fn new(id: u32, name: Arc<String>) -> Self {Self{id, name}}
}
impl Controller for GenericController {
    fn get_id(&self) -> u32 {self.id}
    fn get_name(&self) -> Arc<String> {self.name.clone()}
    fn map_axis(&self, _axis: u8) -> Option<ControllerAxis> {None}
    fn map_button(&self, _button: u8) -> Option<ControllerButton> {None}
}


pub fn make_controller(id: u32, name: Arc<String>) -> Box<dyn Controller> {

    match &**name {
        // taiko drum (ps4, switch) (windows | linux)
        "Taiko Controller"|"HORI CO.,LTD. Taiko Controller" => Box::new(TaikoController::new(id, name)),

        // xbox one controller (windows|linux)
        "Xbox Controller"|"Microsoft X-Box One S pad"|"Microsoft X-Box One pad"|"Microsoft XBox One X pad" => Box::new(XboxController::new(id, name)),

        // ps3 controller (linux only, windows apparently is pain)
        "Sony PLAYSTATION(R)3 Controller" => Box::new(Playstation3Controller::new(id, name)),

        // ps4 controller (windows | linux)
        "Wireless Controller"|"Sony Interactive Entertainment Wireless Controller" => Box::new(Playstation4Controller::new(id, name)),

        // steam controller
        "Steam Controller" => Box::new(SteamController::new(id, name)),

        // unknown controller
        _ => Box::new(GenericController::new(id, name))
    }
}